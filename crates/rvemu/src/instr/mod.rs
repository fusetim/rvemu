mod utils;

use crate::{
    data::Word,
    instr::{
        btype::InstrB,
        itype::InstrI,
        jtype::{JalJInstr, JalrJInstr},
        rtype::InstrR,
    },
    reg::Regs32,
};

pub mod btype;
pub mod itype;
pub mod jtype;
pub mod rtype;
pub mod stype;
pub mod utype;

#[derive(Clone, Copy)]
#[repr(C)]
pub union Instr {
    pub raw: u32,
    pub rtype: InstrR,
    pub itype: InstrI,
    pub btype: InstrB,
    pub jal: JalJInstr,
    pub jalr: JalrJInstr,
}

impl PartialEq for Instr {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.raw == other.raw }
    }
}

impl Eq for Instr {}

pub trait Execute {
    /// Execute will produce a number of steps to complete the execution of the instruction,
    /// these steps will be executed one by one by the emulator, and each step is sure to complete
    /// in a finite amount of time, which enable nice features on the memory-side.
    ///
    /// # Arguments
    ///
    /// * `steps` - A mutable reference to an array of 8 instruction steps, which will be filled with the
    ///   steps to execute the instruction.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of steps that are filled in the steps array, this should be less than or equal to 8.
    fn execute(&self, steps: &mut [InstrStep; 8]) -> usize;
}

macro_rules! execute_one {
    ($instr:ident, $instr_type:ty, |$instr_arg:ident, $regs_arg:ident, $state_arg:ident| $body:expr) => {
        paste::paste! {
            impl crate::instr::Execute for $instr_type {
                #[inline(always)]
                fn execute(&self, steps: &mut [crate::instr::InstrStep; 8]) -> usize {
                    #[inline(always)]
                    fn [<execute_ $instr>]($instr_arg: Instr, $regs_arg: &mut crate::reg::Regs32, $state_arg: &mut crate::instr::InstrState) {
                        $body
                    }
                    steps[0] = crate::instr::InstrStep::Call([<execute_ $instr>]);
                    1 // Number of steps filled in the steps array
                }
            }
        }
    };
}

pub(crate) use execute_one;

impl Execute for Instr {
    #[inline(always)]
    fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
        let opcode = unsafe { self.raw } & 0x7F;
        match opcode {
            0x33 => unsafe { self.rtype.execute(steps) },
            0x13 => unsafe { self.itype.execute(steps) },
            0x63 => unsafe { self.btype.execute(steps) },
            0x6F => unsafe { self.jal.execute(steps) },
            0x67 => unsafe { self.jalr.execute(steps) },
            _ => {
                // For unrecognized opcodes, we can choose to either ignore them (treat them as no-ops) or treat them as invalid instructions.
                // Here, we will treat them as invalid instructions and return a step that indicates an invalid
                steps[0] = InstrStep::TrapInvalidInstruction;
                1
            }
        }
    }
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
#[derive(Clone, Copy)]
pub enum InstrStep {
    /// The simpler one is a call to an handle function, which is a static function that takes the
    /// current state of the instruction and perform a finite-time operation on the virtual machine, such as performing
    /// arithmetic operations, or wrtiting to a register.
    Call(fn(Instr, &mut Regs32, &mut InstrState) -> ()),
    /// No-op, this step does nothing, and can be used to represent the end of an instruction.
    Noop,
    /// Jump to address, stored in the val_c field of the InstrState, this is used for jump instructions to indicate the target address to jump to.
    Jump,
    /// Invalid instruction encountered, this is used to represent an error state when an instruction is not recognized or cannot be executed.
    TrapInvalidInstruction,
}

impl Default for InstrStep {
    fn default() -> Self {
        InstrStep::Noop
    }
}
