//! R-type instruction format and related utilities.
use super::execute_one;
use super::utils::instr_field;
use core::ops::Deref;
use paste::paste;

use crate::instr::{Execute, Instr, InstrStep};

/// R-type instruction format
///
/// ```text
/// 31          25 24   20 19   15 14   12 11    7 6     0
/// funct7         rs2     rs1     funct3  rd      opcode
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RType(u32);

// - R-type instruction field offsets, widths, and masks -
instr_field!(RType, opcode, 0, 7);
instr_field!(RType, rd, 7, 5);
instr_field!(RType, funct3, 12, 3);
instr_field!(RType, rs1, 15, 5);
instr_field!(RType, rs2, 20, 5);
instr_field!(RType, funct7, 25, 7);

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
pub const RTYPE_OPCODE: u32 = 0b0110011;

/// Mask for the differentiating fields of R-type instructions (opcode, funct3, and funct7).
const R_TYPE_PATTERN_MASK: u32 = RTYPE_OPCODE_MASK | RTYPE_FUNCT3_MASK | RTYPE_FUNCT7_MASK;

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
            #[repr(transparent)]
            pub struct [<$mnemonic:camel RInstr>] (RType);

            impl [<$mnemonic:camel RInstr>] {
                /// Creates a new instance of the instruction from a 32-bit word.
                pub const fn new(word: u32) -> Self {
                    Self(RType::new(word))
                }

                pub const fn expected_pattern() -> u32 {
                    (RTYPE_OPCODE << RTYPE_OPCODE_OFFSET) | ([<RINSTR_ $mnemonic:upper _FUNCT3>] << RTYPE_FUNCT3_OFFSET) | ([<RINSTR_ $mnemonic:upper _FUNCT7>] << RTYPE_FUNCT7_OFFSET)
                }

                /// Checks if the instruction matches the expected opcode, funct3, and funct7 for this mnemonic.
                pub fn is_valid(&self) -> bool {
                    // Optimized check for opcode, funct3, and funct7 in one go using masks and shifts
                    const INSTR_EXPECTED : u32 = [<$mnemonic:camel RInstr>]::expected_pattern();
                    let instr_func = self.0.raw() & R_TYPE_PATTERN_MASK;
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

rtype_instr!(add, 0b000, 0b0000000);
rtype_instr!(sub, 0b000, 0b0100000);
rtype_instr!(sll, 0b001, 0b0000000);
rtype_instr!(slt, 0b010, 0b0000000);
rtype_instr!(sltu, 0b011, 0b0000000);
rtype_instr!(xor, 0b100, 0b0000000);
rtype_instr!(srl, 0b101, 0b0000000);
rtype_instr!(sra, 0b101, 0b0100000);
rtype_instr!(or, 0b110, 0b0000000);
rtype_instr!(and, 0b111, 0b0000000);

/// Union of all R-type instructions for easy matching and decoding.
#[derive(Clone, Copy)]
#[repr(C)]
pub union InstrR {
    pub raw: u32,
    pub add: AddRInstr,
    pub sub: SubRInstr,
    pub sll: SllRInstr,
    pub slt: SltRInstr,
    pub sltu: SltuRInstr,
    pub xor: XorRInstr,
    pub srl: SrlRInstr,
    pub sra: SraRInstr,
    pub or: OrRInstr,
    pub and: AndRInstr,
}

impl PartialEq for InstrR {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.raw == other.raw }
    }
}

impl Eq for InstrR {}

macro_rules! execute_rinstr {
    ($instr:ident, $instr_type:ty, |$rs1_val:ident, $rs2_val:ident| $body:expr) => {
        execute_one!($instr, $instr_type, |instr, regs, _state| {
            let instr = unsafe { instr.rtype.$instr };
            let $rs1_val = regs.read(instr.rs1() as usize);
            let $rs2_val = regs.read(instr.rs2() as usize);
            let rd_val = $body;
            regs.write(instr.rd() as usize, rd_val);
            regs.inc_pc(4);
        });
    };
}

impl InstrR {
    pub const fn to_instr(self) -> Instr {
        Instr { rtype: self }
    }
}

// add performs the arethmetic addition of the values in rs1 and rs2, and stores the result in rd.
// Sign and overflow are ignored, and the addition is performed modulo 2^32 (wrapping around on overflow).
execute_rinstr!(add, AddRInstr, |rs1_val, rs2_val| rs1_val
    .wrapping_add(rs2_val));
// sub performs the arethmetic subtraction of the value in rs2 from the value in rs1, and stores the result in rd.
// Sign and overflow are ignored, and the subtraction is performed modulo 2^32 (wrapping around on overflow).
execute_rinstr!(sub, SubRInstr, |rs1_val, rs2_val| rs1_val
    .wrapping_sub(rs2_val));
// sll performs a logical left shift of the value in rs1 by the number of bits specified in the lower 5 bits of rs2, and stores the result in rd.
// The shift amount is masked to 5 bits to ensure it is between 0 and 31, as shifting by more than the word size would be undefined behavior.
// Sign is ignored
execute_rinstr!(sll, SllRInstr, |rs1_val, rs2_val| rs1_val
    .wrapping_shl(rs2_val & 0x1F));
// slt (signed less than) performs a signed comparison between the values in rs1 and rs2, and stores 1 in rd if rs1 is less than rs2, or 0 otherwise.
execute_rinstr!(
    slt,
    SltRInstr,
    |rs1_val, rs2_val| if (rs1_val as i32) < (rs2_val as i32) {
        1
    } else {
        0
    }
);
// sltu (unsigned less than) performs an unsigned comparison between the values in rs1 and rs2, and stores 1 in rd if rs1 is less than rs2, or 0 otherwise.
execute_rinstr!(sltu, SltuRInstr, |rs1_val, rs2_val| if rs1_val < rs2_val {
    1
} else {
    0
});
// xor performs a bitwise exclusive OR operation between the values in rs1 and rs2, and stores the result in rd.
execute_rinstr!(xor, XorRInstr, |rs1_val, rs2_val| rs1_val ^ rs2_val);
// srl performs a logical right shift of the value in rs1 by the number of bits specified in the lower 5 bits of rs2, and stores the result in rd.
// The shift amount is masked to 5 bits to ensure it is between 0 and 31, as shifting by more than the word size would be undefined behavior.
// Shift is logical, meaning that zeros are shifted in from the left regardless of the sign of the value in rs1.
execute_rinstr!(srl, SrlRInstr, |rs1_val, rs2_val| rs1_val
    .wrapping_shr(rs2_val & 0x1F));
// sra performs an arithmetic right shift of the value in rs1 by the number of bits specified in the lower 5 bits of rs2, and stores the result in rd.
// The shift amount is masked to 5 bits to ensure it is between 0 and 31, as shifting by more than the word size would be undefined behavior.
// Shift is arithmetic, meaning that the sign bit (most significant bit) of rs1 is replicated and shifted in from the
// left to preserve the sign of the value in rs1.
execute_rinstr!(
    sra,
    SraRInstr,
    |rs1_val, rs2_val| ((rs1_val as i32).wrapping_shr(rs2_val & 0x1F)) as u32
);
// or performs a bitwise OR operation between the values in rs1 and rs2, and stores the result in rd.
execute_rinstr!(or, OrRInstr, |rs1_val, rs2_val| rs1_val | rs2_val);
// and performs a bitwise AND operation between the values in rs1 and rs2, and stores the result in rd.
execute_rinstr!(and, AndRInstr, |rs1_val, rs2_val| rs1_val & rs2_val);

impl Execute for InstrR {
    fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
        let raw = unsafe { self.raw };
        let raw_masked = raw & R_TYPE_PATTERN_MASK;
        match raw_masked {
            x if x == AddRInstr::expected_pattern() => unsafe { self.add.execute(steps) },
            x if x == SubRInstr::expected_pattern() => unsafe { self.sub.execute(steps) },
            x if x == SllRInstr::expected_pattern() => unsafe { self.sll.execute(steps) },
            x if x == SltRInstr::expected_pattern() => unsafe { self.slt.execute(steps) },
            x if x == SltuRInstr::expected_pattern() => unsafe { self.sltu.execute(steps) },
            x if x == XorRInstr::expected_pattern() => unsafe { self.xor.execute(steps) },
            x if x == SrlRInstr::expected_pattern() => unsafe { self.srl.execute(steps) },
            x if x == SraRInstr::expected_pattern() => unsafe { self.sra.execute(steps) },
            x if x == OrRInstr::expected_pattern() => unsafe { self.or.execute(steps) },
            x if x == AndRInstr::expected_pattern() => unsafe { self.and.execute(steps) },
            _ => {
                // If the instruction doesn't match any known R-type instruction pattern, we can treat it as an invalid instruction.
                steps[0] = InstrStep::TrapInvalidInstruction;
                1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use crate::{instr::InstrState, reg::Regs32};

use super::*;

    const ADD_INSTRS: [AddRInstr; 3] = [
        //  add rd, rs1, rs2
        AddRInstr::new(0x004385b3), //  add x0, x2, x4  |  x0 should stay zero
        AddRInstr::new(0x00200233), //  add x4, x0, x2  |  x4 <- x0 + x2
        AddRInstr::new(0x004385b3), //  add x11, x7, x4 |  x11 <- x7 + x4
    ];

    /// A mixed of all R-type instructions for testing the union type InstrR and the execution of multiple instructions in sequence.
    const R_INSTRS: [InstrR; 11] = [
        AddRInstr::new(0x00820133).to_instr_r(), //  add x2, x4, x8  |  x2 <- x4 + x8            |  x2 <- 4 + 8 = 12
        SubRInstr::new(0x402200b3).to_instr_r(), //  sub x1, x4, x2  |  x1 <- x4 - x2            |  x1 <- 4 - 12 = 0xFFFFFFF8 (wrapping around)
        SllRInstr::new(0x00111133).to_instr_r(), //  sll x2, x2, x1  |  x2 <- x2 << (x1 & 0x1F)  |  x2 <- 12 << (0xFFFFFFF8 & 0x1F) = 12 << 24 = 0x0C000000
        SltRInstr::new(0x00212233).to_instr_r(), //  slt x4, x2, x2  |  x4 <- (x2 < x2) ? 1 : 0  |  x4 <- (0x0C000000 < 0x0C000000) ? 1 : 0 = 0
        SltRInstr::new(0x00412233).to_instr_r(), //  slt x4, x2, x4  |  x4 <- (x2 < x4) ? 1 : 0  |  x4 <- (0x0C000000 < 0) ? 1 : 0 = 0
        SltuRInstr::new(0x00213233).to_instr_r(), //  sltu x4, x2, x2 |  x4 <- (x2 < x2) ? 1 : 0  |  x4 <- (0x0C000000 < 0x0C000000) ? 1 : 0 = 0
        XorRInstr::new(0x0025c533).to_instr_r(), //  xor x10, x11, x2|  x10 <- x11 ^ x2          | x10 <- 11 ^ 0x0C000000 = 0x0BFFFFF5
        SrlRInstr::new(0x001111b3).to_instr_r(), //  srl x3, x2, x1  |  x3 <- x2 >> (x1 & 0x1F)  |  x3 <- 0x0C000000 >> (0xFFFFFFF8 & 0x1F) = 0x0C000000 >> 24 = 12
        SraRInstr::new(0x401151b3).to_instr_r(), //  sra x3, x2, x1  |  x3 <- x2 >> (x1 & 0x1F)  |  x3 <- 0x0C000000 >> (0xFFFFFFF8 & 0x1F) = 0x0C000000 >> 24 = 12 (same as srl since the value is positive and the sign bit is 0)
        OrRInstr::new(0x0025d533).to_instr_r(), //  or x10, x11, x2 |  x10 <- x11 | x2          | x10 <- 11 | 0x0C000000 = 0x0C00000B
        AndRInstr::new(0x0025e533).to_instr_r(), //  and x10, x11, x2|  x10 <- x11 & x2          | x10 <- 11 & 0x0C000000 = 0
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
        let mut steps_filled;
        while (regs.read_pc() as usize) < instrs.len() * 4 {
            // Fetch the instruction at the current PC
            let instr_index = (regs.read_pc() / 4) as usize;
            let instr = instrs[instr_index];

            steps_filled = instr.execute(&mut steps);
            for step in &steps[0..steps_filled] {
                match step {
                    InstrStep::Call(func) => {
                        func(instr.to_instr(), &mut regs, &mut InstrState::default())
                    }
                    InstrStep::Noop => {}
                    _ => panic!("Unexpected instruction step"),
                }
            }
        }
        assert_eq!(regs.read(0), 0); // x0 should stay at zero
        assert_eq!(regs.read(4), 2); // x4 should be 2 after executing the second instruction
        assert_eq!(regs.read(7), 7); // x7 should remain unchanged
        assert_eq!(regs.read(11), 9); // x11 should be 9 after executing the third instruction
    }

    const DEBUG_REGS: Regs32 = Regs32::with(
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ],
        0,
    );

    #[test]
    fn test_execute_all_rinstrs() {
        let instrs = R_INSTRS;
        let mut regs = DEBUG_REGS;

        // Execute the instructions in order
        let mut steps = [InstrStep::Noop; 8];
        let mut steps_filled;
        while (regs.read_pc() as usize) < instrs.len() * 4 {
            // Fetch the instruction at the current PC
            let instr_index = (regs.read_pc() / 4) as usize;
            let instr = instrs[instr_index];

            steps_filled = instr.execute(&mut steps);
            for step in &steps[0..steps_filled] {
                match step {
                    InstrStep::Call(func) => {
                        func(instr.to_instr(), &mut regs, &mut InstrState::default())
                    }
                    InstrStep::Noop => {}
                    InstrStep::TrapInvalidInstruction => panic!(
                        "Invalid instruction encountered during execution, pc: 0x{:08x}, instr: 0x{:08x}",
                        regs.read_pc(),
                        unsafe { instr.raw }
                    ),
                    _ => panic!(
                        "Unexpected instruction step, pc: 0x{:08x}, instr: 0x{:08x}",
                        regs.read_pc(),
                        unsafe { instr.raw }
                    ),
                }
            }

            {
                // Debug print the register state after each instruction execution for easier debugging
                std::println!(
                    "After executing instruction at pc: 0x{:08x}, instr: 0x{:08x}",
                    regs.read_pc(),
                    unsafe { instr.raw }
                );
                for i in 0..32 {
                    std::println!("x{}: 0x{:08x}", i, regs.read(i));
                }
                std::println!("-----------------------------");
            }
        }
        assert_eq!(regs.read_pc(), (instrs.len() * 4) as u32); // PC should be at the end of the instruction sequence after execution
        let expected_regs = [
            0, 0xfffffff8, 0x0c000000, 0x0c, 0, 5, 6, 7, 8, 9, 0x0c00000b, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        ];
        for i in 0..32 {
            assert_eq!(
                regs.read(i),
                expected_regs[i],
                "Register x{} has unexpected value after execution, got: {:08x}, expected: {:08x}",
                i,
                regs.read(i),
                expected_regs[i]
            );
        }
    }
}
