use tokio_retry2::RetryError;

pub trait AsRetryError<T> {
    fn into_retry_error(self) -> Result<T, RetryError<std::io::Error>>;
}

impl<T> AsRetryError<T> for anyhow::Result<T> {
    #[inline]
    fn into_retry_error(self) -> Result<T, RetryError<std::io::Error>> {
        self.map_err(|reason| {
            tracing::error!(?reason, "RPC error");
            let err = std::io::Error::new(std::io::ErrorKind::Other, reason.to_string());
            RetryError::Transient {
                err,
                retry_after: None,
            }
        })
    }
}
