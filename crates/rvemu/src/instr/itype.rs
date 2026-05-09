//! I-type instruction format and related utilities.
use paste::paste;
use core::ops::Deref;
use super::utils::{sign_extend, instr_field};
use super::execute_one;


use crate::{instr::{Execute, Instr, InstrState, InstrStep}, reg::Regs32};

/// I-type instruction format
/// 
/// ```text
/// 31          20 19    15 14   12 11    7 6      0
/// imm[11:0]      rs1      funct3  rd      opcode 
/// 
/// 31       25 24    20 19    15 14   12 11    7 6      0
/// funct7      shamt    rs1      funct3  rd      opcode
///
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct IType(u32);

// - I-type instruction field offsets, widths, and masks -
instr_field!(IType, opcode, 0, 7);
instr_field!(IType, rd, 7, 5);
instr_field!(IType, funct3, 12, 3);
instr_field!(IType, rs1, 15, 5);
instr_field!(IType, imm, 20, 12);

// For shift instructions, the immediate field is split into shamt (shift amount) and funct7, 
// where shamt occupies the lower 5 bits of the imm field, and funct7 occupies the upper 7 bits of the imm field.
instr_field!(IType, funct7, 25, 7);
instr_field!(IType, shamt, 20, 5); 

impl IType {
    /// Creates a new I-type instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(word)
    }

    /// Get the raw 32-bit word of the instruction.
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// Opcode for I-type instructions.
/// This is a constant value that can be used to identify I-type instructions when decoding.
pub const ITYPE_OPCODE : u32 = 0b0010011;

/// Mask for the differentiating fields of I-type instructions (opcode, funct3).
const I_TYPE_PATTERN_MASK : u32 = ITYPE_OPCODE_MASK | ITYPE_FUNCT3_MASK;

macro_rules! __internal__itype_instr {
    ($mnemonic:ident) => {
        paste! {
            /// [<$mnemonic:camel IInstr>] is a representation of the I-type instruction $mnemonic,
            /// which includes methods for validating and matching the instruction based on 
            /// its opcode, funct3.
            #[derive(Debug, Clone, Copy)]
            #[repr(transparent)]
            pub struct [<$mnemonic:camel IInstr>] (IType);

            impl [<$mnemonic:camel IInstr>] {
                /// Creates a new instance of the instruction from a 32-bit word.
                pub const fn new(word: u32) -> Self {
                    Self(IType::new(word))
                }

                /// Checks if the instruction matches the expected opcode, funct3, and funct7 for this mnemonic.
                pub fn is_valid(&self) -> bool {
                    // Optimized check for opcode, funct3, and funct7 in one go using masks and shifts
                    const INSTR_EXPECTED : u32 = [<$mnemonic:camel IInstr>]::expected_pattern();
                    let instr_func = self.0.raw() & I_TYPE_PATTERN_MASK;
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

                /// Converts this instruction into the union type InstrI for easier handling in the emulator.
                pub const fn to_instr_i(self) -> InstrI {
                    InstrI { $mnemonic: self }
                }

                /// Converts this instruction into the general Instr type, which can be used for execution.
                pub const fn to_instr(self) -> Instr {
                    Instr { itype: self.to_instr_i() }
                }
            }

            impl Deref for [<$mnemonic:camel IInstr>] {
                type Target = IType;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        }
    };
}

/// I-type instruction function codes for different operations.
macro_rules! itype_instr {
    ($mnemonic:ident, $func3:expr) => {
        paste! {
            /// Function code discriminent (funct3) for the $mnemonic I-type instruction.
            pub const [<ITINSTR_ $mnemonic:upper _FUNCT3>]: u32 = $func3;

            __internal__itype_instr!($mnemonic);

            impl [<$mnemonic:camel IInstr>] {
                pub const fn expected_pattern() -> u32 {
                    (ITYPE_OPCODE << ITYPE_OPCODE_OFFSET) | ([<ITINSTR_ $mnemonic:upper _FUNCT3>] << ITYPE_FUNCT3_OFFSET)
                }
            }
        }
    };
    ($mnemonic:ident, $func3:expr, $funct7:expr) => {
        paste! {
            /// Function code discriminent (funct3) for the $mnemonic I-type instruction.
            pub const [<ITINSTR_ $mnemonic:upper _FUNCT3>]: u32 = $func3;

            /// Function code discriminent (funct7) for the $mnemonic I-type instruction.
            pub const [<ITINSTR_ $mnemonic:upper _FUNCT7>]: u32 = $funct7;

            __internal__itype_instr!($mnemonic);

            impl [<$mnemonic:camel IInstr>] {
                pub const fn expected_pattern() -> u32 {
                    (ITYPE_OPCODE << ITYPE_OPCODE_OFFSET) | ([<ITINSTR_ $mnemonic:upper _FUNCT3>] << ITYPE_FUNCT3_OFFSET) | ([<ITINSTR_ $mnemonic:upper _FUNCT7>] << ITYPE_FUNCT7_OFFSET)
                }
            }
        }
    };
}

/* The classic I-type instructions */
itype_instr!(addi,   0b000);
itype_instr!(slti,   0b010);
itype_instr!(sltui,  0b011);
itype_instr!(xori,   0b100);
itype_instr!(ori,    0b110);
itype_instr!(andi,   0b111);
/* The I-type instructions, with an additional funct7 to differentiate them */
itype_instr!(slli,  0b001, 0b0000000);
itype_instr!(srli,  0b101, 0b0000000);
itype_instr!(srai,  0b101, 0b0100000);


/// Union of all I-type instructions for easy matching and decoding.
#[derive(Clone, Copy)]
#[repr(C)]
pub union InstrI {
    pub raw:    u32,
    pub addi:   AddiIInstr,
    pub slti:   SltiIInstr,
    pub sltui:  SltuiIInstr,
    pub xori:   XoriIInstr,
    pub ori:    OriIInstr,
    pub andi:   AndiIInstr,
    pub slli:   SlliIInstr,
    pub srli:   SrliIInstr,
    pub srai:   SraiIInstr,
}

impl PartialEq for InstrI {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            self.raw == other.raw
        }
    }
}

impl Eq for InstrI {}

macro_rules! execute_iinstr {
    ($instr:ident, $instr_type:ty, |$rs1_val:ident, $sext_imm:ident, $shamt:ident| $body:expr) => {
        execute_one!($instr, $instr_type, |instr, regs, _state| {
            let instr = unsafe { instr.itype.$instr };
            let $rs1_val = regs.read(instr.rs1() as usize);
            // For the immediate value, we need to sign-extend it to 32 bits before using it in the execution of the instruction, 
            // since the immediate field in the I-type instruction is only 12 bits wide, and it can represent both positive and negative values.
            let $sext_imm = sign_extend(instr.imm() as i32, 12);
            let $shamt = instr.shamt();
            let _ = ($sext_imm, $shamt); // To avoid unused variable warning for instructions that don't use both of them
            let rd_val = $body;
            regs.write(instr.rd() as usize, rd_val);
            regs.inc_pc(4);
        });
    };
}

impl InstrI {
    pub const fn to_instr(self) -> Instr {
        Instr { itype: self }
    } 
}

// addi performs the arethmetic addition of the value in rs1 and the immediate value,
// and writes the result to rd. The addition is performed as a wrapping addition, meaning that if
// the result exceeds the maximum value that can be represented in a 32-bit unsigned integer, it will
// wrap around to the beginning of the range.
execute_iinstr!(addi, AddiIInstr, |rs1_val, sext_imm, shamt| (rs1_val as i32).wrapping_add(sext_imm) as u32);

// slti performs a signed comparison between the value in rs1 and the immediate value, and writes 1 to rd if rs1 is less than imm, or 0 otherwise.
// To do so, the immediate value must be sign-extended to 32 bits before the comparison, and the comparison itself must be performed as a 
// signed comparison, meaning that the values are interpreted as signed integers rather than unsigned integers.
execute_iinstr!(slti, SltiIInstr, |rs1_val, sext_imm, shamt| {
    if (rs1_val as i32) < sext_imm {
        1
    } else {
        0
    }
});

// sltui performs an unsigned comparison between the value in rs1 and the immediate value, and writes 1 to rd if rs1 is less than imm, or 0 otherwise.
// Even if is an unsigned comparison, the immediate value still needs to be sign-extended to 32 bits before the comparison.
execute_iinstr!(sltui, SltuiIInstr, |rs1_val, sext_imm, shamt| {
    if (rs1_val as u32) < (sext_imm as u32) {
        1
    } else {
        0
    }
});

// xori performs a bitwise exclusive OR operation between the value in rs1 and the immediate value, and writes the result to rd. 
execute_iinstr!(xori, XoriIInstr, |rs1_val, sext_imm, shamt| rs1_val ^ (sext_imm as u32)); 

// ori performs a bitwise OR operation between the value in rs1 and the immediate value, and writes the result to rd.
execute_iinstr!(ori, OriIInstr, |rs1_val, sext_imm, shamt| rs1_val | (sext_imm as u32));

// andi performs a bitwise AND operation between the value in rs1 and the immediate value, and writes the result to rd.
execute_iinstr!(andi, AndiIInstr, |rs1_val, sext_imm, shamt| rs1_val & (sext_imm as u32));

// slli performs a logical left shift operation on the value in rs1 by the number of positions specified in the shamt field (which is the lower 5 bits of the imm field),
// and writes the result to rd.
execute_iinstr!(slli, SlliIInstr, |rs1_val, sext_imm, shamt| rs1_val << shamt);

// srli performs a logical right shift operation on the value in rs1 by the number of positions specified in the shamt field (which is the lower 5 bits of the imm field),
// and writes the result to rd. The logical right shift operation shifts in zeros from the left, regardless of the sign of the value in rs1.
execute_iinstr!(srli, SrliIInstr, |rs1_val, sext_imm, shamt| rs1_val >> shamt);

// srai performs an arithmetic right shift operation on the value in rs1 by the number of positions specified in the shamt field (which is the lower 5 bits of the imm field),
// and writes the result to rd. The arithmetic right shift operation shifts in the sign bit (the most significant bit) from the left, which means that if the value in rs1 is negative
// (i.e., has its most significant bit set to 1), the shifted value will also be negative, and if the value in rs1 is positive (i.e., has its most significant bit set to 0), the shifted value will also be positive.
execute_iinstr!(srai, SraiIInstr, |rs1_val, sext_imm, shamt| ((rs1_val as i32) >> shamt) as u32);


impl Execute for InstrI {
    fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
        let raw = unsafe { self.raw };
        let raw_masked = raw & I_TYPE_PATTERN_MASK;
        let funct7 = (raw & ITYPE_FUNCT7_MASK) >> ITYPE_FUNCT7_OFFSET;
        match raw_masked {
            x if x == (AddiIInstr::expected_pattern()  & I_TYPE_PATTERN_MASK) => AddiIInstr::new(raw).execute(steps),
            x if x == (SltiIInstr::expected_pattern()  & I_TYPE_PATTERN_MASK) => SltiIInstr::new(raw).execute(steps),
            x if x == (SltuiIInstr::expected_pattern() & I_TYPE_PATTERN_MASK) => SltuiIInstr::new(raw).execute(steps),
            x if x == (XoriIInstr::expected_pattern()  & I_TYPE_PATTERN_MASK) => XoriIInstr::new(raw).execute(steps),
            x if x == (OriIInstr::expected_pattern()   & I_TYPE_PATTERN_MASK) => OriIInstr::new(raw).execute(steps),
            x if x == (AndiIInstr::expected_pattern()  & I_TYPE_PATTERN_MASK) => AndiIInstr::new(raw).execute(steps),
            x if x == (SlliIInstr::expected_pattern()  & I_TYPE_PATTERN_MASK) && funct7 == ITINSTR_SLLI_FUNCT7 => SlliIInstr::new(raw).execute(steps),
            x if x == (SrliIInstr::expected_pattern()  & I_TYPE_PATTERN_MASK) && funct7 == ITINSTR_SRLI_FUNCT7 => SrliIInstr::new(raw).execute(steps),
            x if x == (SraiIInstr::expected_pattern()  & I_TYPE_PATTERN_MASK) && funct7 == ITINSTR_SRAI_FUNCT7 => SraiIInstr::new(raw).execute(steps),
            _ => {
                // If the instruction doesn't match any known I-type instruction pattern, we can treat it as an invalid instruction.
                steps[0] = InstrStep::TrapInvalidInstruction;
                1
            }
        }
    }
}

