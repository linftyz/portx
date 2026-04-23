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
    #[error("port {port} has no killable PID")]
    NoPidForPort { port: u16 },
    #[error("port {port} was not found")]
    PortNotFound { port: u16 },
    #[error("port {port} has multiple PIDs; use --pid to select one")]
    MultiplePidsForPort { port: u16 },
    #[error("PID {pid} is not listening on port {port}")]
    PidNotOnPort { port: u16, pid: u32 },
    #[error("refusing to kill without confirmation in a non-interactive terminal; pass --yes")]
    ConfirmationRequired,
    #[error("kill signal is not supported on this platform")]
    UnsupportedSignal,
    #[error("failed to signal PID {pid}")]
    KillFailed { pid: u32 },
    #[error("portx data collection is not implemented yet for this milestone")]
    NotImplemented,
}

impl PortxError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Cli(error) => error.exit_code().try_into().unwrap_or(2),
            Self::Json(_)
            | Self::Socket(_)
            | Self::NoPidForPort { .. }
            | Self::PortNotFound { .. }
            | Self::MultiplePidsForPort { .. }
            | Self::PidNotOnPort { .. }
            | Self::ConfirmationRequired
            | Self::UnsupportedSignal
            | Self::KillFailed { .. }
            | Self::NotImplemented => 1,
        }
    }
}
