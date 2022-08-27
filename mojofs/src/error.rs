
pub const MOJOFS_ERR_NOT_IMPL: i32 = 1;
pub const MOJOFS_ERR_IO: i32 = 2;
pub const MOJOFS_ERR_NIX: i32 = 3;
pub const MOJOFS_ERR_UTF8: i32 = 4;
pub const MOJOFS_ERR_MOJOKV: i32 = 5;
pub const MOJOFS_ERR_URL_PARSE: i32 = 6;
pub const MOJOFS_ERR_INT_PARSE: i32 = 7;
pub const MOJOFS_ERR_LARGE_PAGE: i32 = 8;
pub const MOJOFS_ERR_ARG_VER_MISSING: i32 = 9;
pub const MOJOFS_ERR_ARG_PAGESZ_MISSING: i32 = 10;
pub const MOJOFS_ERR_ARG_PPS_MISSING: i32 = 11;

#[derive(thiserror::Error, Debug)]
pub struct Error {
    pub code: i32,
    pub msg: String,
}

impl Error {
    pub fn new(code: i32, msg: String) -> Self {
        Error {
            code,
            msg,
        }
    }

    pub fn not_impl() -> Self {
        Error::new(MOJOFS_ERR_NOT_IMPL, "Not implemented".to_owned())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error { 
            code: MOJOFS_ERR_IO,
            msg: format!("{:?}", err),
        }
    }
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Self {
        Error { 
            code: MOJOFS_ERR_NIX,
            msg: err.to_string(),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error { 
            code: MOJOFS_ERR_UTF8,
            msg: err.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error { 
            code: MOJOFS_ERR_INT_PARSE,
            msg: err.to_string(),
        }
    }
}

impl From<mojokv::Error> for Error {
    fn from(err: mojokv::Error) -> Self {
        Error { 
            code: MOJOFS_ERR_MOJOKV,
            msg: err.to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.code, self.msg)
    }
}
