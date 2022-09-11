pub mod nix;
mod error;

pub use error::Error;

pub const BUFFER_MAGIC: &[u8] = b"mojo";
pub const PAGE_HEADER_LEN: usize = 8;


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
