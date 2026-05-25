#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod data;
pub mod executor;
pub mod instr;
pub mod reg;

#[cfg(feature = "std")]
pub(crate) use std::dbg;

#[cfg(not(feature = "std"))]
macro_rules! dbg {
    ($($arg:expr),*) => {};
}

#[cfg(not(feature = "std"))]
pub(crate) use dbg;