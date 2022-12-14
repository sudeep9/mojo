//#![feature(write_all_vectored)]

pub mod index;
mod bucket;
mod error;
mod value;
mod state;
mod keymap;
mod utils;
mod store;
mod bmap;

pub use error::Error;
pub use bucket::Bucket;
pub use bmap::BucketMap;
pub use keymap::KeyMap;
pub use value::{Value, Slot};
pub use store::{Store, BucketOpenMode};


//TODO: Pass pps from single place