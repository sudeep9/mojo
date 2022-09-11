
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    IoErr(#[from] std::io::Error),

    #[error("Bucket {0} not found at ver={1}")]
    BucketNotAtVerErr(String, u32),

    #[error("Bucket not writable")]
    BucketNotWritableErr,

    #[error("Version no longer writable bucket ver={0} active ver={1}")]
    VerNotWritable(u32, u32),

    #[error("Store not found")]
    StoreNotFoundErr,

    #[error("Store not writable")]
    StoreNotWritableErr,

    #[error("Missing arguments")]
    MissingArgsErr,

    #[error("Commit lock could not be acquired")]
    CommitLockedErr,

    #[error("Only single version exists")]
    SingleVersionErr,

    #[error("Json serialization error")]
    SerdeJsonErr(#[from] serde_json::Error),

    #[error("rmp encode error")]
    RmpEncodeErr(#[from] rmp_serde::encode::Error),

    #[error("rmp decode error")]
    RmpDecodeErr(#[from] rmp_serde::decode::Error),

    #[error("Key {0} not found")]
    KeyNotFoundErr(u32),

    #[error("Key {0} not multiple of page size")]
    KeyNotMultipleErr(u32),

    #[error("Version {0} not found")]
    VersionNotFoundErr(u32),

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
