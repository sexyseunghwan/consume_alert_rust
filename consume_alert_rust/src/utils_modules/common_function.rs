use crate::common::*;

/// Awaits `fut`, logging `ctx` alongside the error before propagating it.
pub async fn log_ctx<T>(
    ctx: &str,
    fut: impl std::future::Future<Output = anyhow::Result<T>>,
) -> anyhow::Result<T> {
    fut.await.inspect_err(|e| error!("{}: {:#}", ctx, e))
}
