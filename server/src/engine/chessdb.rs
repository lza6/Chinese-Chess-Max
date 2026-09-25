use std::time::Duration;

use super::QueryResult;
use super::QueryState;

const URL: &str = "https://www.chessdb.cn/chessdb.php";
const REFER: &str = "https://www.chessdb.cn/query/";
const AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.113 Safari/537.36";
const SOURCE_CHESSDB: &str = "云库";
async fn query_once(fen: &str, timeout: u64) -> QueryResult {
    let mut records = super::QueryResult::default();
    let resp = reqwest::Client::new()
        .get(URL)
        .header("User-Agent", AGENT)
        .header("Referer", REFER)
        .query(&[("action", "querypv"), ("board", fen)])
        .timeout(Duration::from_secs(timeout))
        .send()
        .await;
    match resp {
        Ok(resp) => {
            let text = match resp.text().await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("chessdb read body error: {}", e);
                    records.state = QueryState::ServerInternalError;
                    return records;
                }
            };
            let text = text.strip_suffix('\0').unwrap_or(&text);
            match text {
                "" | "unknown" => records.state = QueryState::NotResult,
                "invalid board" | "checkmate" | "stalemate" => {
                    records.state = QueryState::InvalidBoard
                }
                text => {
                    tracing::debug!("chessdb resp: {}", text);
                    for pair in text.split(',') {
                        let mut parts = pair.split(':');
                        match parts.next().unwrap_or("") {
                            "score" => {
                                if let Some(v) = parts.next() {
                                    records.score = v.parse().unwrap_or(0);
                                }
                            }
                            "depth" => {
                                if let Some(v) = parts.next() {
                                    records.depth = v.parse().unwrap_or(0);
                                }
                            }
                            "pv" => {
                                if let Some(pv_text) = parts.next() {
                                    for pv in pv_text.split('|') {
                                        // 过滤空着法/尾分隔符产生的空串，避免进入 analyse
                                        if !pv.is_empty() {
                                            records.pvs.push(pv.to_string());
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    records.state = QueryState::Success;
                    records.source = SOURCE_CHESSDB.to_string();
                }
            }
        }
        Err(e) => {
            tracing::warn!("chessdb query error: {}", e);
            records.state = QueryState::ServerInternalError
        }
    };
    records
}

/// 查询云库（querypv）。瞬态失败（网络/连接异常）重试 1 次，避免偶发抖动直接降级引擎。
pub async fn query(fen: &str, timeout: u64) -> QueryResult {
    let first = query_once(fen, timeout).await;
    if first.state == QueryState::ServerInternalError {
        tracing::debug!("chessdb 首查失败，200ms 后重试 1 次");
        tokio::time::sleep(Duration::from_millis(200)).await;
        return query_once(fen, timeout).await;
    }
    first
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_query() {
    let result = query(
        "2baka3/9/n3b4/r1p1nr2p/4R4/2PR5/4P3P/4B1N2/4A4/2B1KA3 w",
        10,
    )
    .await;
    println!("{:?}", result);
}
