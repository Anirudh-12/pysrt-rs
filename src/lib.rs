#![cfg_attr(not(feature = "python"), forbid(unsafe_code))]

pub mod error;
pub mod file;
pub mod item;
pub mod time;

#[cfg(feature = "python")]
pub mod python;

pub use error::{Result, SrtError};
pub use file::{ErrorHandling, SubRipFile};
pub use item::SubRipItem;
pub use time::SubRipTime;

pub fn open<P: AsRef<std::path::Path>>(path: P, encoding: Option<&str>) -> Result<SubRipFile> {
    SubRipFile::open(path, encoding)
}

pub fn from_string(source: &str) -> Result<SubRipFile> {
    SubRipFile::from_string(source)
}
