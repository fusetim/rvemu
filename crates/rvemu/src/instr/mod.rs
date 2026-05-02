use core::any::Any;

use crate::{data::{DoubleWord, MemoryController, Word}, instr::rtype::InstrR, reg::Regs32};

pub mod rtype;
pub mod itype;
pub mod stype;
pub mod utype;
pub mod btype;
pub mod jtype;

#[derive(Clone, Copy)]
pub union Instr {
    pub raw: u32,
    pub rtype: InstrR,
}

impl PartialEq for Instr {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            self.raw == other.raw
        }
    }
}

impl Eq for Instr {}

pub trait Execute {
    /// Execute will produce a number of steps to complete the execution of the instruction, 
    /// these steps will be executed one by one by the emulator, and each step is sure to complete
    /// in a finite amount of time, which enable nice features on the memory-side.
    fn execute(&self) -> [InstrStep; 8];
}

#[derive(PartialEq, Eq, Default)]
pub struct InstrState {
    pub val_a: Word,
    pub val_b: Word,
    pub val_c: Word,
    pub val_mem: Word,
}

impl InstrState {
    pub fn new() -> Self {
        Self {
            val_a: 0,
            val_b: 0,
            val_c: 0,
            val_mem: 0,
        }
    }
}

/// Instruction step is a way to decompose an instruction into atomic steps that can be executed by the emulator, 
/// one by one. Each step is sure to complete in a finite amount of time, which enable nice features on the memory-side.
pub enum InstrStep {
    /// The simpler one is a call to an handle function, which is a static function that takes the 
    /// current state of the instruction and perform a finite-time operation on the virtual machine, such as performing
    /// arithmetic operations, or wrtiting to a register.
    Call(&'static dyn Fn(Instr, &mut Regs32, &mut InstrState)),
    /// No-op, this step does nothing, and can be used to represent the end of an instruction.
    Noop,
    /// Memory load byte from memory (RAM / ROM / MMIO peripherals), it will store the result inside
    /// the val_mem value of the instruction state.
    MemoryLoadByte(Word),
    /// Memory load half-word from memory (RAM / ROM / MMIO peripherals), it will store the result inside
    /// the val_mem value of the instruction state.
    MemoryLoadHalf(Word),
    /// Memory load word from memory (RAM / ROM / MMIO peripherals), it will store the result inside
    /// the val_mem value of the instruction state.
    MemoryLoadWord(Word),
    /// Memory load double word from memory (RAM / ROM / MMIO peripherals), it will store the result inside
    /// the val_mem value of the instruction state.
    MemoryLoadDoubleWord(Word),
    /// Memory store byte to memory (RAM / ROM / MMIO peripherals), it will store the value to be stored inside the val_mem value of the instruction state.
    MemoryStoreByte(Word),
    /// Memory store half-word to memory (RAM / ROM / MMIO peripherals), it will store the value to be stored inside the val_mem value of the instruction state.
    MemoryStoreHalf(Word),
    /// Memory store word to memory (RAM / ROM / MMIO peripherals), it will store the value to be stored inside the val_mem value of the instruction state.
    MemoryStoreWord(Word),
    /// Memory store double word to memory (RAM / ROM / MMIO peripherals), it will store the value to be stored inside the val_mem value of the instruction state.
    MemoryStoreDoubleWord(Word),
    /// Increment the program counter by a certain value, this is used to move to the next instruction after the current one is executed.
    IncrementPC(Word),
    /// Jump to a certain address, this is used for control flow instructions such as jumps and branches.
    Jump(Word),
}