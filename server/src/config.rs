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
            let reader = match File::open(&config_path) {
                Ok(f) => BufReader::new(f),
                Err(e) => {
                    tracing::warn!("读取配置文件失败（使用默认配置）: {e}");
                    let config = Config {
                        config_path: Some(config_path.clone()),
                        ..Default::default()
                    };
                    config.save();
                    return config;
                }
            };
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
        // 序列化/写盘失败不 panic：记录日志，避免 set_engine_* 命令内 panic 导致 RwLock 中毒
        let path = match self.config_path.as_ref() {
            Some(p) => p,
            None => return,
        };
        debug!("save config to '{}'", path.display());
        let json_string = match serde_json::to_string_pretty(self) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("序列化配置失败（跳过保存）: {e}");
                return;
            }
        };

        // 原子写：先写临时文件再替换，避免崩溃/中断导致配置损坏
        let tmp = path.with_extension("json.tmp");
        match File::create(&tmp).and_then(|mut f| f.write_all(json_string.as_bytes())) {
            Ok(()) => {}
            Err(e) => {
                tracing::error!("写入配置临时文件失败（跳过保存）: {e}");
                return;
            }
        }
        if let Err(e) = std::fs::rename(&tmp, path) {
            // 跨盘/权限等场景 rename 失败时回退直接写
            tracing::warn!("原子写失败({}), 回退直接写入: {}", e, path.display());
            match File::create(path).and_then(|mut f| f.write_all(json_string.as_bytes())) {
                Ok(()) => {}
                Err(e2) => {
                    tracing::error!("回退直接写入配置失败: {e2}");
                }
            }
            let _ = std::fs::remove_file(&tmp);
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
#[cfg(test)]
mod tests {
    use super::*;

    fn temp_base() -> std::path::PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "ccm-config-test-{pid}-{n}",
            pid = std::process::id()
        ))
    }

    #[test]
    fn load_creates_default_config_when_missing() {
        let base = temp_base();
        std::fs::create_dir_all(&base).unwrap();
        let config = Config::load(&base);
        // 默认引擎配置
        assert_eq!(config.engine.depth, 20);
        assert_eq!(config.engine.time, 5000);
        assert_eq!(config.engine.threads, 4);
        assert_eq!(config.engine.hash, 64);
        assert!(config.engine.chessdb_enabled);
        // 配置文件已创建
        assert!(base.join("xqlink/config.json").exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn save_then_load_roundtrip_preserves_config() {
        let base = temp_base();
        std::fs::create_dir_all(&base).unwrap();
        let mut config = Config::load(&base);
        config.engine.depth = 30;
        config.engine.time = 8000;
        config.engine.threads = 8;
        config.save();

        let reloaded = Config::load(&base);
        assert_eq!(reloaded.engine.depth, 30);
        assert_eq!(reloaded.engine.time, 8000);
        assert_eq!(reloaded.engine.threads, 8);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn load_backs_up_corrupted_config_and_rebuilds_default() {
        let base = temp_base();
        std::fs::create_dir_all(&base).unwrap();
        let dir = base.join("xqlink");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.json"), "{ corrupted json !!").unwrap();

        let config = Config::load(&base);
        // 备份存在 + 默认重建
        assert!(dir.join("config.json.bak").exists());
        assert_eq!(config.engine.depth, 20);
        // 新配置可解析
        let reloaded = Config::load(&base);
        assert_eq!(reloaded.engine.depth, 20);
        let _ = std::fs::remove_dir_all(&base);
    }
}
