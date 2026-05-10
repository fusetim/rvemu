//! J-type instructions and related utilities.

use core::ops::Deref;

use crate::instr::{Execute, Instr, InstrStep, execute_one};

use super::utils::instr_field;

/// U-instruction format
///
/// ```text
/// 31           12 11   7 6    0
/// imm[31:12]     rd     opcode
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct UType(u32);

instr_field!(UType, opcode, 0, 7);
instr_field!(UType, rd, 7, 5);
instr_field!(UType, imm31_12, 12, 20);

impl UType {
    /// Creates a new U-type instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(word)
    }

    /// Get the raw 32-bit word of the instruction.
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Get the immediate value of the U-type instruction, which is a 20-bit signed integer formed
    /// by concatenating imm[31:12].
    #[inline(always)]
    pub fn imm(&self) -> i32 {
        let imm31_12 = self.imm31_12() << 12; // imm[31:12]
        imm31_12 as i32
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct AuipcUInstr(UType);

/// Opcode for AUIPC instruction.
pub const AUIPC_OPCODE: u32 = 0b0010111;

/// Mask for the differentiating fields of AUIPC instruction (opcode).
pub const AUIPC_PATTERN_MASK: u32 = UTYPE_OPCODE_MASK;

impl AuipcUInstr {
    /// Creates a new AUIPC instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(UType::new(word))
    }

    pub const fn to_instr_u(self) -> InstrU {
        InstrU { auipc: self }
    }

    pub const fn to_instr(self) -> Instr {
        Instr {
            utype: self.to_instr_u(),
        }
    }

    pub const fn expected_pattern(&self) -> u32 {
        AUIPC_OPCODE
    }
}

impl Deref for AuipcUInstr {
    type Target = UType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

execute_one!(auipc, AuipcUInstr, |instr, regs, _state| {
    let instr = unsafe { instr.utype.auipc }; // Safe because we are sure that the instruction is indeed a AUIPC instruction when this function is called.
    let rd = instr.rd() as usize;
    let imm = instr.imm();
    // Write the return address (address of the next instruction) to rd
    regs.write(rd, regs.read_pc().wrapping_add(imm as u32));
    // Increment the program counter to point to the next instruction
    regs.inc_pc(4);
});

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct LuiUInstr(UType);

/// Opcode for LUI instruction.
pub const LUI_OPCODE: u32 = 0b0110111;

/// Mask for the differentiating fields of LUI instruction (opcode).
pub const LUI_PATTERN_MASK: u32 = UTYPE_OPCODE_MASK;

impl LuiUInstr {
    /// Creates a new LUI instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(UType::new(word))
    }

    pub const fn to_instr_u(self) -> InstrU {
        InstrU { lui: self }
    }

    pub const fn to_instr(self) -> Instr {
        Instr {
            utype: self.to_instr_u(),
        }
    }

    pub const fn expected_pattern(&self) -> u32 {
        LUI_OPCODE
    }
}

impl Deref for LuiUInstr {
    type Target = UType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

execute_one!(lui, LuiUInstr, |instr, regs, _state| {
    let instr = unsafe { instr.utype.lui }; // Safe because we are sure that the instruction is indeed a LUI instruction when this function is called.
    let rd = instr.rd() as usize;
    let imm = instr.imm();
    regs.write(rd, imm as u32);
    // Increment the program counter to point to the next instruction
    regs.inc_pc(4);
});

#[derive(Clone, Copy)]
#[repr(C)]
pub union InstrU {
    pub raw: u32,
    pub lui: LuiUInstr,
    pub auipc: AuipcUInstr,
}

impl PartialEq for InstrU {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.raw == other.raw }
    }
}

impl InstrU {
    pub const fn new(word: u32) -> Self {
        Self { raw: word }
    }

    pub const fn to_instr(self) -> Instr {
        Instr { utype: self }
    }
}

impl Eq for InstrU {}

impl Execute for InstrU {
    fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
        let opcode = unsafe { self.raw } & UTYPE_OPCODE_MASK;
        match opcode {
            LUI_OPCODE => unsafe { self.lui.execute(steps) },
            AUIPC_OPCODE => unsafe { self.auipc.execute(steps) },
            _ => {
                // If the instruction doesn't match any known I-type instruction pattern, we can treat it as an invalid instruction.
                steps[0] = InstrStep::TrapInvalidInstruction;
                1
            }
        }
    }
}
