use thiserror::Error;
use tokio_retry2::RetryError;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MainError {
    #[error("No action error")]
    NoAction,
    #[error("RPC error")]
    RpcError,
}

pub trait AsRetryError<T> {
    fn into_retry_error(self) -> Result<T, RetryError<std::io::Error>>;
}

impl<T> AsRetryError<T> for anyhow::Result<T> {
    #[inline]
    fn into_retry_error(self) -> Result<T, RetryError<std::io::Error>> {
        self.map_err(|reason| {
            let err = std::io::Error::new(std::io::ErrorKind::Other, reason.to_string());
            RetryError::Transient {
                err,
                retry_after: None,
            }
        })
    }
}
