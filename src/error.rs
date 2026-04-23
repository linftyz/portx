use thiserror::Error;

pub type Result<T> = std::result::Result<T, PortxError>;

#[derive(Debug, Error)]
pub enum PortxError {
    #[error(transparent)]
    Cli(#[from] clap::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Socket(#[from] netstat2::error::Error),
    #[error("portx data collection is not implemented yet for this milestone")]
    NotImplemented,
}

impl PortxError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Cli(error) => error.exit_code().try_into().unwrap_or(2),
            Self::Json(_) | Self::Socket(_) | Self::NotImplemented => 1,
        }
    }
}
