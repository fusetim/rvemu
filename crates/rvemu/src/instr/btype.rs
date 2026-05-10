//! B-type instruction format and related utilities.
use super::utils::{instr_field, sign_extend};
use crate::{
    instr::{Execute, Instr, InstrState, InstrStep},
    reg::Regs32,
};
use core::ops::Deref;
use paste::paste;

/// B-type instruction format
///
/// ```text
/// 31        25   24   20 19   15 14    12 11        7  6    0
/// imm[12|10:5]   rs2     rs1     funct3   imm[4:1|11]  opcode
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct BType(u32);

// - B-type instruction field offsets, widths, and masks -
instr_field!(BType, opcode, 0, 7);
instr_field!(BType, imml, 7, 5);
instr_field!(BType, funct3, 12, 3);
instr_field!(BType, rs1, 15, 5);
instr_field!(BType, rs2, 20, 5);
instr_field!(BType, immh, 25, 7);

impl BType {
    /// Creates a new B-type instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(word)
    }

    /// Get the raw 32-bit word of the instruction.
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Get the immediate value of the B-type instruction, which is a 13-bit signed integer formed
    /// by concatenating imm[12], imm[10:5], imm[4:1], and imm[11].
    #[inline(always)]
    pub fn imm(&self) -> i32 {
        // While this function does a lot of work, thanks to compiler optimization, this function becomes quite small
        // in instructions in practice.
        let imm12 = self.immh() >> 6; // imm[12] is the highest bit of immh
        let imm10_5 = self.immh() & 0x3F; // imm[10:5] are the lower 6 bits of immh
        let imm4_1 = (self.imml() >> 1) & 0x1F; // imm[4:1] are the upper 5 bits of imml
        let imm11 = self.imml() & 0x1; // imm[11] is the lowest bit of imml

        // Reconstruct the 13-bit immediate value by concatenating the fields in the correct order
        // The bit 0 is always 0 (aligned to 16 bit)
        let imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
        sign_extend(imm as i32, 13)
    }
}

/// Opcode for B-type instructions.
/// This is a constant value that can be used to identify B-type instructions when decoding.
pub const BTYPE_OPCODE: u32 = 0b1100011;

/// Mask for the differentiating fields of B-type instructions (opcode, funct3).
const B_TYPE_PATTERN_MASK: u32 = BTYPE_OPCODE_MASK | BTYPE_FUNCT3_MASK;

/// B-type instruction function codes for different operations.
macro_rules! btype_instr {
    ($mnemonic:ident, $func3:expr) => {
        paste! {
            /// Function code discriminent (funct3) for the $mnemonic B-type instruction.
            pub const [<BTYPE_INSTR_ $mnemonic:upper _FUNCT3>]: u32 = $func3;

            /// [<$mnemonic:camel BInstr>] is a representation of the B-type instruction $mnemonic,
            /// which includes methods for validating and matching the instruction based on
            /// its opcode, funct3.
            #[derive(Debug, Clone, Copy)]
            #[repr(transparent)]
            pub struct [<$mnemonic:camel BInstr>] (BType);

            impl [<$mnemonic:camel BInstr>] {
                /// Creates a new instance of the instruction from a 32-bit word.
                pub const fn new(word: u32) -> Self {
                    Self(BType::new(word))
                }

                /// Checks if the instruction matches the expected opcode, funct3, and funct7 for this mnemonic.
                pub fn is_valid(&self) -> bool {
                    // Optimized check for opcode, funct3, and funct7 in one go using masks and shifts
                    const INSTR_EXPECTED : u32 = [<$mnemonic:camel BInstr>]::expected_pattern();
                    let instr_func = self.0.raw() & B_TYPE_PATTERN_MASK;
                    instr_func == INSTR_EXPECTED
                }

                /// Attempts to match a 32-bit word to this instruction type. Returns Some(instruction) if it matches, or None if it doesn't.
                pub fn match_instr(word: u32) -> Option<Self> {
                    let instr = Self::new(word);
                    if instr.is_valid() {
                        Some(instr)
                    } else {
                        None
                    }
                }

                /// Converts this instruction into the union type InstrB for easier handling in the emulator.
                pub const fn to_instr_b(self) -> InstrB {
                    InstrB { $mnemonic: self }
                }

                /// Converts this instruction into the general Instr type, which can be used for execution.
                pub const fn to_instr(self) -> Instr {
                    Instr { btype: self.to_instr_b() }
                }
            }

            impl Deref for [<$mnemonic:camel BInstr>] {
                type Target = BType;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl [<$mnemonic:camel BInstr>] {
                pub const fn expected_pattern() -> u32 {
                    (BTYPE_OPCODE << BTYPE_OPCODE_OFFSET) | ([<BTYPE_INSTR_ $mnemonic:upper _FUNCT3>] << BTYPE_FUNCT3_OFFSET)
                }
            }
        }
    };
}

btype_instr!(beq, 0b000);
btype_instr!(bne, 0b001);
btype_instr!(blt, 0b100);
btype_instr!(bge, 0b101);
btype_instr!(bltu, 0b110);
btype_instr!(bgeu, 0b111);

/// Union of all B-type instructions for easy matching and decoding.
#[derive(Clone, Copy)]
#[repr(C)]
pub union InstrB {
    pub raw: u32,
    pub beq: BeqBInstr,
    pub bne: BneBInstr,
    pub blt: BltBInstr,
    pub bge: BgeBInstr,
    pub bltu: BltuBInstr,
    pub bgeu: BgeuBInstr,
}

impl PartialEq for InstrB {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.raw == other.raw }
    }
}

impl Eq for InstrB {}

impl InstrB {
    pub const fn to_instr(self) -> Instr {
        Instr { btype: self }
    }
}

macro_rules! execute_binstr {
    ($instr:ident, $instr_type:ty, |$rs1_val:ident, $rs2_val:ident| $predicate:expr) => {
        paste! {
            impl Execute for $instr_type {
                #[inline(always)]
                fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
                    #[inline(always)]
                    fn [<execute_ $instr>](instr: Instr, regs: &mut Regs32, _state: &mut InstrState) {
                        let instr_b = unsafe { instr.btype.$instr };
                        let $rs1_val = regs.read(instr_b.rs1() as usize);
                        let $rs2_val = regs.read(instr_b.rs2() as usize);
                        let pc = regs.read_pc();
                        let jump : bool = $predicate;
                        if jump {
                            regs.write_pc((pc as i32).wrapping_add(instr_b.imm()) as u32);
                        } else {
                            regs.write_pc(pc.wrapping_add(4)); // Next instruction address if not jumping
                        }
                    }
                    steps[0] = InstrStep::Call([<execute_ $instr>]);
                    1 // Number of steps filled in the steps array
                }
            }
        }
    };
}

execute_binstr!(beq, BeqBInstr, |rs1_val, rs2_val| rs1_val == rs2_val);
execute_binstr!(bne, BneBInstr, |rs1_val, rs2_val| rs1_val != rs2_val);
execute_binstr!(blt, BltBInstr, |rs1_val, rs2_val| (rs1_val as i32)
    < (rs2_val as i32));
execute_binstr!(bge, BgeBInstr, |rs1_val, rs2_val| (rs1_val as i32)
    >= (rs2_val as i32));
execute_binstr!(bltu, BltuBInstr, |rs1_val, rs2_val| rs1_val < rs2_val);
execute_binstr!(bgeu, BgeuBInstr, |rs1_val, rs2_val| rs1_val >= rs2_val);

impl Execute for InstrB {
    fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
        let raw = unsafe { self.raw };
        let raw_masked = raw & B_TYPE_PATTERN_MASK;
        match raw_masked {
            x if x == (BeqBInstr::expected_pattern()) => unsafe { self.beq.execute(steps) },
            x if x == (BneBInstr::expected_pattern()) => unsafe { self.bne.execute(steps) },
            x if x == (BltBInstr::expected_pattern()) => unsafe { self.blt.execute(steps) },
            x if x == (BgeBInstr::expected_pattern()) => unsafe { self.bge.execute(steps) },
            x if x == (BltuBInstr::expected_pattern()) => unsafe { self.bltu.execute(steps) },
            x if x == (BgeuBInstr::expected_pattern()) => unsafe { self.bgeu.execute(steps) },
            _ => {
                // If the instruction doesn't match any known I-type instruction pattern, we can treat it as an invalid instruction.
                steps[0] = InstrStep::TrapInvalidInstruction;
                1
            }
        }
    }
}
