use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use tracing::debug;

use crate::SHARED_STATE;
use crate::engine::EngineConfig;

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    config_path: Option<PathBuf>,
    // trace, debug, info, wran, silent
    pub loglevel: String,
    pub timer_interval: u64,
    pub confirm_interval: u64,

    pub engine: EngineConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: None,
            loglevel: "INFO".to_string(),
            timer_interval: 100,
            confirm_interval: 200,
            engine: Default::default(),
        }
    }
}

impl Config {
    pub fn load(base: &Path) -> Self {
        let dir = base.join("xqlink");
        if !dir.exists() {
            let _ = fs::create_dir(&dir);
        };

        let config_path = dir.join("config.json");
        debug!("load config from '{}'", config_path.display());

        if config_path.exists() {
            let reader = BufReader::new(File::open(&config_path).unwrap());
            if let Ok(mut config) = serde_json::from_reader::<_, Config>(reader) {
                match config.config_path {
                    Some(ref path) => {
                        if path != &config_path {
                            config.config_path = Some(config_path);
                            config.save();
                        }
                    }
                    None => {
                        config.config_path = Some(config_path);
                        config.save();
                    }
                };
                return config;
            };

            // 解析失败代表配置不兼容：备份原文件（保留用户数据），再用默认配置
            let backup = dir.join("config.json.bak");
            let _ = std::fs::copy(&config_path, &backup);
            let _ = std::fs::remove_file(&config_path);
            tracing::warn!(
                "配置文件解析失败，已备份至 {} 并重建默认配置",
                backup.display()
            )
        }

        let config = Config {
            config_path: Some(config_path),
            ..Default::default()
        };
        config.save();
        config
    }

    pub fn save(&self) {
        let path = self.config_path.as_ref().unwrap();
        debug!("save config to '{}'", path.display());
        // 将对象序列化为格式化的JSON字符串
        let json_string = serde_json::to_string_pretty(self).unwrap();

        // 原子写：先写临时文件再替换，避免崩溃/中断导致配置损坏
        let tmp = path.with_extension("json.tmp");
        {
            let mut file = File::create(&tmp).unwrap();
            file.write_all(json_string.as_bytes()).unwrap();
            file.sync_all().ok();
        }
        match std::fs::rename(&tmp, path) {
            Ok(()) => {}
            Err(e) => {
                // 跨盘/权限等场景 rename 失败时回退直接写
                tracing::warn!("原子写失败({}), 回退直接写入: {}", e, path.display());
                let mut file = File::create(path).unwrap();
                file.write_all(json_string.as_bytes()).unwrap();
                let _ = std::fs::remove_file(&tmp);
            }
        }
    }
}

#[tauri::command]
pub async fn get_engine_config() -> EngineConfig {
    SHARED_STATE.get().unwrap().config.read().unwrap().engine
}

#[tauri::command]
pub async fn set_engine_depth(depth: usize) {
    let state = SHARED_STATE.get().unwrap();
    let mut config = state.config.write().unwrap();
    config.engine.depth = depth;
    config.save();
    debug!("set_engine_depth: {}", depth);
}

#[tauri::command]
pub async fn set_engine_time(time: f32) {
    let state = SHARED_STATE.get().unwrap();
    let mut config = state.config.write().unwrap();
    config.engine.time = (time * 1000.0) as usize;
    config.save();
    debug!("set_engine_time: {}", time);
}

#[tauri::command]
pub async fn set_engine_threads(num: usize) {
    let state = SHARED_STATE.get().unwrap();
    let mut config = state.config.write().unwrap();
    config.engine.threads = num;
    config.save();
    debug!("set_engine_threads: {}", num);
}

#[tauri::command]
pub async fn set_engine_hash(size: usize) {
    let state = SHARED_STATE.get().unwrap();
    let mut config = state.config.write().unwrap();
    config.engine.hash = size;
    config.save();
    debug!("set_engine_hash: {}", size);
}

#[tauri::command]
pub async fn set_chessdb(enabled: bool, timeout: Option<u64>) {
    let state = SHARED_STATE.get().unwrap();
    let mut config = state.config.write().unwrap();
    let timeout = timeout.unwrap_or_else(|| config.engine.chessdb_timeout.min(1));
    config.engine.chessdb_enabled = enabled;
    config.engine.chessdb_timeout = timeout;
    config.save();
    debug!("set_chessdb: {} -> {}", enabled, timeout);
}
