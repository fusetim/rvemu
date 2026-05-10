#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod data;
pub mod executor;
pub mod instr;
pub mod reg;
