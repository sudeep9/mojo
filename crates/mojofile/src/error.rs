
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    IoErr(#[from] std::io::Error),

    #[error("Unix error")]
    NixErr(#[from] nix::Error),

    #[error("Parse int error")]
    ParseIntErr(#[from] std::num::ParseIntError),

    #[error("Unknown error `{0}`")]
    UnknownStr(String),

    #[error("UTF8 error")]
    UTF8Err(#[from] std::str::Utf8Error),

    #[error("Unknown error")]
    Unknown,
}