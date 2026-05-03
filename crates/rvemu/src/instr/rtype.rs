//! Dedicated module for R-type instruction format and related utilities.
use paste::paste;
use core::ops::Deref;

use crate::{execute_one, instr::{Execute, Instr, InstrState, InstrStep}, reg::Regs32};

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
            /// Offset of the $field field in the R-type instruction.
            pub const [<RTYPE_ $field:upper _OFFSET>]: u32 = $offset;
            /// Width of the $field field in the R-type instruction.
            pub const [<RTYPE_ $field:upper _WIDTH>]: u32 = $width;
            /// Mask for the $field field in the R-type instruction, used to extract the field value from the raw instruction word.
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
            /// Function code discriminent (funct3) for the $mnemonic R-type instruction.
            pub const [<RINSTR_ $mnemonic:upper _FUNCT3>]: u32 = $func3;
            /// Function code discriminent (funct7) for the $mnemonic R-type instruction.
            pub const [<RINSTR_ $mnemonic:upper _FUNCT7>]: u32 = $func7;

            /// [<$mnemonic:camel RInstr>] is a representation of the R-type instruction $mnemonic,
            /// which includes methods for validating and matching the instruction based on 
            /// its opcode, funct3, and funct7 fields.
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

                /// Converts this instruction into the union type InstrR for easier handling in the emulator.
                pub const fn to_instr_r(self) -> InstrR {
                    InstrR { $mnemonic: self }
                }

                /// Converts this instruction into the general Instr type, which can be used for execution.
                pub const fn to_instr(self) -> Instr {
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

macro_rules! execute_rinstr {
    ($instr:ident, $instr_type:ty, |$rs1_val:ident, $rs2_val:ident, $rd_val:ident| $body:expr) => {
        execute_one!($instr, $instr_type, |instr, regs, _state| {
            let instr = unsafe { instr.rtype.$instr };
            let $rs1_val = regs.read(instr.rs1() as usize);
            let $rs2_val = regs.read(instr.rs2() as usize);
            let $rd_val = $body;
            regs.write(instr.rd() as usize, $rd_val);
            regs.inc_pc(4);
        });
    };
}

execute_rinstr!(add, AddRInstr, |rs1_val, rs2_val, rd_val| rs1_val.wrapping_add(rs2_val));
execute_rinstr!(sub, SubRInstr, |rs1_val, rs2_val, rd_val| rs1_val.wrapping_sub(rs2_val));
execute_rinstr!(sll, SllRInstr, |rs1_val, rs2_val, rd_val| rs1_val.wrapping_shl(rs2_val & 0x1F));
execute_rinstr!(slt, SltRInstr, |rs1_val, rs2_val, rd_val| if (rs1_val as i32) < (rs2_val as i32) { 1 } else { 0 });
execute_rinstr!(sltu, SltuRInstr, |rs1_val, rs2_val, rd_val| if rs1_val < rs2_val { 1 } else { 0 });
execute_rinstr!(xor, XorRInstr, |rs1_val, rs2_val, rd_val| rs1_val ^ rs2_val);
execute_rinstr!(srl, SrlRInstr, |rs1_val, rs2_val, rd_val| rs1_val.wrapping_shr(rs2_val & 0x1F));
execute_rinstr!(sra, SraRInstr, |rs1_val, rs2_val, rd_val| ((rs1_val as i32).wrapping_shr(rs2_val & 0x1F)) as u32);
execute_rinstr!(or, OrRInstr, |rs1_val, rs2_val, rd_val| rs1_val | rs2_val);
execute_rinstr!(and, AndRInstr, |rs1_val, rs2_val, rd_val| rs1_val & rs2_val);

#[cfg(test)]
mod tests {
    use super::*;

    const ADD_INSTRS: [AddRInstr; 3] = [
                                            //  add rd, rs1, rs2
        AddRInstr::new(0x004385b3),    //  add x0, x2, x4  |  x0 should stay zero
        AddRInstr::new(0x00200233),    //  add x4, x0, x2  |  x4 <- x0 + x2
        AddRInstr::new(0x004385b3),    //  add x11, x7, x4 |  x11 <- x7 + x4 
    ];

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
        let instrs = ADD_INSTRS;
        let mut regs = Regs32::new();
        regs.write(2, 2); // x2 = 2
        regs.write(4, 4); // x4 = 4
        regs.write(7, 7); // x7 = 7
        regs.write(11, 0); // x11 = 0

        // Execute the instructions in order
        let mut steps = [InstrStep::Noop; 8];
        let mut steps_filled = 0;
        while (regs.read_pc() as usize) < instrs.len() * 4 {
            // Fetch the instruction at the current PC
            let instr_index = (regs.read_pc() / 4) as usize;
            let instr = instrs[instr_index];

            steps_filled = instr.execute(&mut steps);
            for step in &steps[0..steps_filled] {
                match step {
                    InstrStep::Call(func) => func(instr.to_instr(), &mut regs, &mut InstrState::default()),
                    InstrStep::Jump(addr) => regs.write_pc(*addr),
                    InstrStep::Noop => {},
                    _ => panic!("Unexpected instruction step"),
                }
            }
        }
        assert_eq!(regs.read(0), 0); // x0 should stay at zero
        assert_eq!(regs.read(4), 2); // x4 should be 2 after executing the second instruction
        assert_eq!(regs.read(7), 7); // x7 should remain unchanged
        assert_eq!(regs.read(11), 9); // x11 should be 9 after executing the third instruction
    }

}