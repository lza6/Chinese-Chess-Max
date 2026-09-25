use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::RwLock;
use std::thread;

use engine::Engine;
use tauri::Manager as _;
use worker::GameHistory;

mod chess;
mod common;
mod config;
mod engine;
mod listen;
mod logger;
mod worker;
mod yolo;

// 全局共享状态，用Arc和Mutex包装以实现线程安全共享
struct SharedState {
    config: Arc<RwLock<config::Config>>,
    engine: Arc<Mutex<Engine>>,
    listen_thread: Mutex<Option<thread::JoinHandle<()>>>,
    history: Mutex<GameHistory>,
}

static SHARED_STATE: OnceLock<SharedState> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            logger::init_tracer(tracing::Level::DEBUG, &app.path().app_data_dir().unwrap());

            let _ = SHARED_STATE.get_or_init(|| {
                let config = config::Config::load(&app.path().config_dir().unwrap());
                let lib_path = app
                    .path()
                    .resolve("../libs/pikafish", tauri::path::BaseDirectory::Resource)
                    .unwrap();
                let mut engine = engine::Engine::new(&lib_path)
                    .expect("引擎初始化失败（检查 libs/pikafish 资源与 pikafish.nnue）");
                engine
                    .set_show_wdl(config.engine.show_wdl)
                    .expect("引擎 setoption show_wdl 失败");
                engine
                    .set_hash(config.engine.hash)
                    .expect("引擎 setoption hash 失败");
                engine
                    .set_threads(config.engine.threads)
                    .expect("引擎 setoption threads 失败");

                SharedState {
                    config: Arc::new(RwLock::new(config)),
                    engine: Arc::new(Mutex::new(engine)),
                    listen_thread: Mutex::new(None),
                    history: Mutex::new(GameHistory::default()),
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            reload_engine,
            listen::list_windows,
            worker::start_listen,
            worker::stop_listen,
            config::get_engine_config,
            config::set_engine_depth,
            config::set_engine_time,
            config::set_engine_threads,
            config::set_engine_hash,
            config::set_chessdb,
            worker::get_current_fen,
            worker::export_game,
            worker::load_history,
            worker::clear_history,
            worker::review_step,
            worker::human_move,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn reload_engine(app: tauri::AppHandle) {
    let lib_path = app
        .path()
        .resolve("../libs/pikafish", tauri::path::BaseDirectory::Resource)
        .unwrap();
    let state = SHARED_STATE.get().unwrap();
    let engine_config = state.config.read().unwrap().engine;
    state
        .engine
        .lock()
        .unwrap()
        .reload(&lib_path, &engine_config)
        .expect("引擎重载失败");
}
