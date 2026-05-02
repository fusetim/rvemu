//! Dedicated module for R-type instruction format and related utilities.
use paste::paste;
use core::ops::Deref;

use crate::{instr::{Execute, Instr, InstrState, InstrStep}, reg::Regs32};

/// R-type instruction format
/// 
/// ```text
/// 31          25 24   20 19   15 14   12 11    7 6     0
/// funct7         rs2     rs1     funct3  rd      opcode
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RType(u32);

/// R-type instruction field offsets, widths, and masks 
/// These constants and methods are generated using the `rtype_instr_field` macro for each field in the R-type instruction format.
/// The macro generates constants for the offset, width, and mask of each field, as well as methods to extract the raw and shifted values of each field from an RType instruction.
macro_rules! rtype_instr_field {
    ($field:ident, $offset:expr, $width:expr) => {
        paste! {
            pub const [<RTYPE_ $field:upper _OFFSET>]: u32 = $offset;
            pub const [<RTYPE_ $field:upper _WIDTH>]: u32 = $width;
            pub const [<RTYPE_ $field:upper _MASK>]: u32 = ((1 << [<RTYPE_ $field:upper _WIDTH>]) - 1) << [<RTYPE_ $field:upper _OFFSET>];
            impl RType {
                /// Extracts the $field field from the instruction, just masking it but without shifting it.
                #[inline(always)]
                pub fn [<raw_ $field>](&self) -> u32 {
                    self.0 & [<RTYPE_ $field:upper _MASK>]
                }
                /// Extracts the $field field from the instruction.
                #[inline(always)]
                pub fn $field(&self) -> u32 {
                    self.[<raw_ $field>]() >> [<RTYPE_ $field:upper _OFFSET>]
                }
            }
        }
    };
}

// - R-type instruction field offsets, widths, and masks -
rtype_instr_field!(opcode, 0, 7);
rtype_instr_field!(rd, 7, 5);
rtype_instr_field!(funct3, 12, 3);
rtype_instr_field!(rs1, 15, 5);
rtype_instr_field!(rs2, 20, 5);
rtype_instr_field!(funct7, 25, 7);

impl RType {
    /// Creates a new R-type instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(word)
    }

    /// Get the raw 32-bit word of the instruction.
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// Opcode for R-type instructions.
/// This is a constant value that can be used to identify R-type instructions when decoding.
pub const RTYPE_OPCODE : u32 = 0b0110011;

/// R-type instruction function codes for different operations.
macro_rules! rtype_instr {
    ($mnemonic:ident, $func3:expr, $func7:expr) => {
        paste! {
            pub const [<RINSTR_ $mnemonic:upper _FUNCT3>]: u32 = $func3;
            pub const [<RINSTR_ $mnemonic:upper _FUNCT7>]: u32 = $func7;

            #[derive(Debug, Clone, Copy)]
            pub struct [<$mnemonic:camel RInstr>] (RType);

            impl [<$mnemonic:camel RInstr>] {
                /// Creates a new instance of the instruction from a 32-bit word.
                pub const fn new(word: u32) -> Self {
                    Self(RType::new(word))
                }

                /// Checks if the instruction matches the expected opcode, funct3, and funct7 for this mnemonic.
                pub fn is_valid(&self) -> bool {
                    // Optimized check for opcode, funct3, and funct7 in one go using masks and shifts
                    const INSTR_MASK : u32 = RTYPE_OPCODE_MASK | RTYPE_FUNCT3_MASK | RTYPE_FUNCT7_MASK;
                    const INSTR_EXPECTED : u32 = (RTYPE_OPCODE << RTYPE_OPCODE_OFFSET) | ([<RINSTR_ $mnemonic:upper _FUNCT3>] << RTYPE_FUNCT3_OFFSET) | ([<RINSTR_ $mnemonic:upper _FUNCT7>] << RTYPE_FUNCT7_OFFSET);
                    let instr_func = self.0.raw() & INSTR_MASK;
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

                pub fn to_instr_r(self) -> InstrR {
                    InstrR { $mnemonic: self }
                }

                pub fn to_instr(self) -> Instr {
                    Instr { rtype: self.to_instr_r() }
                }
            }

            impl Deref for [<$mnemonic:camel RInstr>] {
                type Target = RType;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        }
    };
}

rtype_instr!(add,   0b000, 0b0000000);
rtype_instr!(sub,   0b000, 0b0100000);
rtype_instr!(sll,   0b001, 0b0000000);
rtype_instr!(slt,   0b010, 0b0000000);
rtype_instr!(sltu,  0b011, 0b0000000);
rtype_instr!(xor,   0b100, 0b0000000);
rtype_instr!(srl,   0b101, 0b0000000);
rtype_instr!(sra,   0b101, 0b0100000);
rtype_instr!(or,    0b110, 0b0000000);
rtype_instr!(and,   0b111, 0b0000000);

/// Union of all R-type instructions for easy matching and decoding.
#[derive(Clone, Copy)]
pub union InstrR {
    pub raw:    u32,
    pub add:    AddRInstr,
    pub sub:    SubRInstr,
    pub sll:    SllRInstr,
    pub slt:    SltRInstr,
    pub sltu:   SltuRInstr,
    pub xor:    XorRInstr,
    pub srl:    SrlRInstr,
    pub sra:    SraRInstr,
    pub or:     OrRInstr,
    pub and:    AndRInstr,
}

impl PartialEq for InstrR {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            self.raw == other.raw
        }
    }
}

impl Eq for InstrR {}

impl Execute for AddRInstr {
    #[inline(always)]
    fn execute(&self) -> [InstrStep; 8] {
        fn execute_add(instr: Instr, regs: &mut Regs32, _state: &mut InstrState) {
            // Safety: instr is expected to be an AddRInstr
            let instr = unsafe { instr.rtype.add };
            let rs1_val = regs.read(instr.rs1() as usize);
            let rs2_val = regs.read(instr.rs2() as usize);
            let result = rs1_val.wrapping_add(rs2_val);
            regs.write(instr.rd() as usize, result);
            regs.inc_pc(4); // Increment PC by 4 to move to the next instruction
        }
        [
            InstrStep::Call(&execute_add),
            InstrStep::Noop,
            InstrStep::Noop,
            InstrStep::Noop,
            InstrStep::Noop,
            InstrStep::Noop,     
            InstrStep::Noop,
            InstrStep::Noop,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtype_fields() {
        let instr = RType::new(0x00000013);
        assert_eq!(instr.opcode(), 0b0010011);
        assert_eq!(instr.rd(), 0);
        assert_eq!(instr.funct3(), 0b000);
        assert_eq!(instr.rs1(), 0);
        assert_eq!(instr.rs2(), 0);
        assert_eq!(instr.funct7(), 0b0000000);
    }

    #[test]
    fn test_execute_add() {
        let add_instr = AddRInstr::new(0b0000000_00010_00001_000_00011_0110011); // add x3, x1, x2
        let mut regs = Regs32::new();
        regs.write(1, 5); // x1 = 5
        regs.write(2, 10); // x2 = 10

        let steps = add_instr.execute();
        let instr = add_instr.to_instr();
        let mut state = InstrState::new();
        for step in steps.iter() {
            match step {
                InstrStep::Call(func) => func(instr, &mut regs, &mut state),
                _ => {},
            }
        }

        assert_eq!(regs.read(3), 15); // x3 should be 15 after execution
    }
}