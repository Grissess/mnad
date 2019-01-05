#![feature(libc)]

extern crate rlapack;
extern crate libc;

pub mod types;
pub use self::types::*;
pub mod util;
pub use self::util::*;
pub mod solver;

#[cfg(test)]
mod test;