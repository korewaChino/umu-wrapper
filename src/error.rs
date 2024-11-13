use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("No Proton installation provided")]
    NoProton,

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde Error: {0}")]
    Serde(#[from] toml::de::Error),

    #[error("{0}")]
    Other(#[from] color_eyre::eyre::Report),
}
