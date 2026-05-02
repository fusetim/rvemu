#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod data;
pub mod instr;
pub mod reg;
pub mod executor;