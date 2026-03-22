use example::{async_example, sync_example};
use ironsaga::anyhow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async_example().await;
    sync_example();
    Ok(())
}
