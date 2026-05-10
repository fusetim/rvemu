//! J-type instructions and related utilities.

use crate::instr::{Execute, Instr, InstrStep, execute_one, utils::sign_extend};

use super::utils::instr_field;

/// JAL instruction format
///
/// ```text
/// 31                     12 11   7 6    0
/// imm[20|10:1|11|19:12]     rd     opcode
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JalJInstr(u32);

instr_field!(JalJInstr, opcode, 0, 7);
instr_field!(JalJInstr, rd, 7, 5);
instr_field!(JalJInstr, imm10_1, 21, 10);
instr_field!(JalJInstr, imm11, 20, 1);
instr_field!(JalJInstr, imm19_12, 12, 8);
instr_field!(JalJInstr, imm20, 31, 1);

/// Opcode for JAL instruction.
pub const JAL_OPCODE: u32 = 0b1101111;

/// Mask for the differentiating fields of JAL instruction (opcode).
pub const JAL_PATTERN_MASK: u32 = JALJINSTR_OPCODE_MASK;

impl JalJInstr {
    /// Creates a new JAL instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(word)
    }

    /// Get the raw 32-bit word of the instruction.
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Get the immediate value of the JAL instruction, which is a 21-bit signed integer formed
    /// by concatenating imm[20], imm[10:1], imm[11], and imm[19:12].
    #[inline(always)]
    pub fn imm(&self) -> i32 {
        let imm20 = self.imm20() << 20; // imm[20]
        let imm10_1 = self.imm10_1() << 1; // imm[10:1]
        let imm11 = self.imm11() << 11; // imm[11]
        let imm19_12 = self.imm19_12() << 12; // imm[19:12]
        (imm20 | imm19_12 | imm11 | imm10_1) as i32
    }

    pub const fn to_instr_j(self) -> InstrJ {
        InstrJ { jal: self }
    }

    pub const fn to_instr(self) -> Instr {
        Instr {
            jtype: self.to_instr_j(),
        }
    }

    pub const fn expected_pattern(&self) -> u32 {
        JAL_OPCODE
    }
}

execute_one!(jal, JalJInstr, |instr, regs, _state| {
    let instr = unsafe { instr.jtype.jal }; // Safe because we are sure that the instruction is indeed a JAL instruction when this function is called.
    let rd = instr.rd() as usize;
    let imm = instr.imm();
    // Write the return address (address of the next instruction) to rd
    regs.write(rd, regs.read_pc().wrapping_add(4));
    // Write the target address (address of the instruction to jump to) to the program counter
    regs.write_pc(regs.read_pc().wrapping_add(imm as u32));
});

/// JALR instruction format
///
/// ```text
/// 31       20 19   15 14    12 11   7 6    0
/// imm[11:0]   rs1     funct3   rd     opcode
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct JalrJInstr(u32);

instr_field!(JalrJInstr, opcode, 0, 7);
instr_field!(JalrJInstr, rd, 7, 5);
instr_field!(JalrJInstr, funct3, 12, 3);
instr_field!(JalrJInstr, rs1, 15, 5);
instr_field!(JalrJInstr, imm11_0, 20, 12);

/// Opcode for JALR instruction.
pub const JALR_OPCODE: u32 = 0b1100111;

/// Mask for the differentiating fields of JALR instruction (opcode + funct3).
pub const JALR_PATTERN_MASK: u32 = JALRJINSTR_OPCODE_MASK | JALRJINSTR_FUNCT3_MASK;

/// Funct3 for JALR instruction.
pub const JALR_FUNCT3: u32 = 0b000;

impl JalrJInstr {
    /// Creates a new JALR instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(word)
    }

    /// Get the raw 32-bit word of the instruction.
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Get the immediate value of the JALR instruction, which is a 12-bit signed integer from imm[11:0].
    #[inline(always)]
    pub fn imm(&self) -> i32 {
        sign_extend(self.imm11_0() as i32, 12)
    }

    pub const fn to_instr_j(self) -> InstrJ {
        InstrJ { jalr: self }
    }

    pub const fn to_instr(self) -> Instr {
        Instr {
            jtype: self.to_instr_j(),
        }
    }

    pub const fn expected_pattern(&self) -> u32 {
        JALR_OPCODE | (JALR_FUNCT3 << JALRJINSTR_FUNCT3_OFFSET)
    }
}

execute_one!(jalr, JalrJInstr, |instr, regs, _state| {
    let instr = unsafe { instr.jtype.jalr }; // Safe because we are sure that the instruction is indeed a JALR instruction when this function is called.
    let rd = instr.rd() as usize;
    let rs1 = instr.rs1() as usize;
    let imm = instr.imm();
    let jump_address = regs.read(rs1).wrapping_add(imm as u32);
    // Write the return address (address of the next instruction) to rd
    regs.write(rd, regs.read_pc().wrapping_add(4));
    // Write the target address (address of the instruction to jump to) to the program counter
    // Todo: theorically, the target address should be aligned to 16 bit, it might be a good idea to check this and
    // raise an exception if it's not the case, but for now we just ignore this detail.
    regs.write_pc(jump_address);
});

#[derive(Clone, Copy)]
#[repr(C)]
pub union InstrJ {
    pub raw: u32,
    pub jal: JalJInstr,
    pub jalr: JalrJInstr,
}

impl PartialEq for InstrJ {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.raw == other.raw }
    }
}

impl InstrJ {
    pub const fn new(word: u32) -> Self {
        Self { raw: word }
    }

    pub const fn to_instr(self) -> Instr {
        Instr { jtype: self }
    }
}

impl Eq for InstrJ {}

impl Execute for InstrJ {
    fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
        let opcode = unsafe { self.raw } & 0x7F;
        match opcode {
            JAL_OPCODE => unsafe { self.jal.execute(steps) },
            JALR_OPCODE => {
                let funct3 = unsafe { self.jalr.funct3() };
                if funct3 == JALR_FUNCT3 {
                    unsafe { self.jalr.execute(steps) }
                } else {
                    // If the funct3 doesn't match the expected value for JALR, we can treat it as an invalid instruction.
                    steps[0] = InstrStep::TrapInvalidInstruction;
                    1
                }
            }
            _ => {
                // If the instruction doesn't match any known I-type instruction pattern, we can treat it as an invalid instruction.
                steps[0] = InstrStep::TrapInvalidInstruction;
                1
            }
        }
    }
}
