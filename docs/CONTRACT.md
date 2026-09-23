# 前后端契约（Frontend ↔ Backend Tauri Contract）

本文档是唯一契约事实来源。`scripts/ci/check-contract.mjs` 会自动校验其两套名义集合，
`tests/Contract.test.ts` 提供仓库级回归。

## 1. 命令（invoke）
| 命令 | 参数 | 返回 | 前端调用点 | 说明 |
|---|---|---|---|---|
| `list_windows` | - | `Vec<Window>{id,title,app_name,width,height}` | Toolbar | 枚举可截屏窗口 |
| `start_listen` | `{target: Window}` | `Result<(),String>` | Toolbar | 启动监听线程；重复启动返回“已经在监听中” |
| `stop_listen` | - | `()` | Toolbar | 停止后台监听线程 |
| `get_engine_config` | - | `EngineConfig{depth,time,threads,hash,show_wdl,chessdb_enabled,chessdb_timeout}` | Toolbar | 读配置（time 单位=**毫秒**,见下） |
| `set_engine_depth` | `{depth:usize}` | `()` | Toolbar | 持久化 |
| `set_engine_time` | `{time:f32}` | `()` | Toolbar | ⚠ 单位=**秒**；后端 `*1000` 存 ms |
| `set_engine_threads` | `{num:usize}` | `()` | Toolbar | 持久化 |
| `set_engine_hash` | `{size:usize}` | `()` | Toolbar | 持久化（MB） |
| `set_chessdb` | `{enabled:bool,timeout:Option<u64>}` | `()` | Toolbar | 象棋DB 开关/超时 |
| `reload_engine` | - | `()` | ❌未调用 | 保留命令 |

**单位规则**：`get_engine_config.time` = **毫秒**；前端展示/编辑为**秒**；
`set_engine_time` 传**秒**（前端换算 `value.time/1000` 显示，写回时直接传秒，后端再 `×1000`）。

## 2. 事件（listen / emit）
| 事件 | 方向 | payload | 消费方/发射方 |
|---|---|---|---|
| `analyse` | BE→FE | `QueryResult{depth,score,time,pvs[],moves[],state,source}` | Analyse.vue |
| `mirror` | BE→FE | `bool`（黑方=true） | Chessboard.vue |
| `position` | BE→FE / FE→BE | `Position{piece:char,pos}`[]（初始由 FE emit startpos） | Chessboard.vue |
| `move` | BE→FE | `Changed{piece:char,camp,from,to}` | Chessboard.vue |

`QueryState` 枚举序列化为字符串（`Success/NotResult/InvalidBoard/ServerInternalError`），与前端 `state:string` 一致。

## 3. 配置持久化
- 引擎/识别参数持久化：后端 `Config::load(base)` 读写管理目录 `xqlink/config.json`；
  解析失败会删除重建为默认值（潜在丢自定义配置，已列入 OPEN ITEM）。
- 前端不直接写配置，全部经命令。

## 4. 关键资源依赖
| 资源 | 路径 | 需求 |
|---|---|---|
| YOLOv8 模型 | `libs/large.onnx`（`server/src/yolo.rs:24` `include_bytes!`） | **编译期硬依赖，当前缺失** |
| 引擎 | `libs/pikafish/pikafish-{windows,os,linux}` + `pikafish.nnue` | 运行时（已入库） |
| onnxruntime | `libs/windows-{cpu,gpu}/*.dll`、`libs/linux/libonnxruntime*.so` | 打包资源（未入库） |

## 5. 已知缺口 / 待办
- [ ] `reload_engine` 前端未接（保留）。
- [ ] 配置删除策略（解析失败直接删文件）需改“增量迁移”。
- [ ] `large.onnx` 未入库 → Rust 编译/安装包暂不可构建。