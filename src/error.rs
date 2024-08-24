#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error("{0}")]
    Header(#[from] http::header::InvalidHeaderValue),
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    #[error("{0}")]
    Url(#[from] url::ParseError),
    #[error("failed to build subtitle request")]
    SubtitleRequestBuild,
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    DotEnv(#[from] dotenv::Error),
    #[error("{0}")]
    Env(#[from] std::env::VarError),
    #[error("not file extension")]
    NoExtension,
    #[error("{0}")]
    Io(#[from] std::io::Error)
}

pub type Result<T> = std::result::Result<T, Error>;