use anyhow::Result;
use jsonrpc_v2::ResponseObjects;
use std::io::Write;
use tokio::io::{AsyncBufRead, AsyncBufReadExt};
use std::sync::Arc;

use jsonrpc_v2::Server;

pub async fn run_with_io<R, W>(
    server: Arc<Server<jsonrpc_v2::MapRouter>>,
    reader: R,
    mut writer: W,
) -> Result<()>
where
    R: AsyncBufRead + Unpin,
    W: Write,
{
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = server.handle(trimmed.as_bytes()).await;
        match response {
            ResponseObjects::Empty => {}
            other => {
                if let Ok(s) = serde_json::to_string(&other) {
                    let _ = writeln!(writer, "{}", s);
                    let _ = writer.flush();
                }
            }
        }
    }

    Ok(())
}
