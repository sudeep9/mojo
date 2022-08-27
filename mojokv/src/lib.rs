//#![feature(write_all_vectored)]

mod index;
mod bucket;
mod error;
mod file;
mod value;
mod state;
mod keymap;
mod utils;
mod store;
mod bmap;

pub use error::Error;
pub use bucket::Bucket;
pub use bmap::BucketMap;
pub use index::*;
pub use keymap::KeyMap;
pub use value::{Value, Slot};
pub use store::{KVStore, BucketOpenMode};


//TODO: Pass pps from single place