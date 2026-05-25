//! S-type instructions and related utilities.

use core::ops::Deref;

use crate::instr::{Execute, Instr, InstrStep, execute_one, utils::sign_extend};

use super::utils::instr_field;

/// Load-S instruction format
/// ```text
/// 31      20  19  15  14   12 11   7 6    0
/// imm[11:0]   rs1     funct3  rd     opcode
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct LoadSType(u32);

instr_field!(LoadSType, opcode, 0, 7);
instr_field!(LoadSType, rd, 7, 5);
instr_field!(LoadSType, funct3, 12, 3);
instr_field!(LoadSType, rs1, 15, 5);
instr_field!(LoadSType, imm11_0, 20, 12);

impl LoadSType {
    /// Creates a new Load-S type instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(word)
    }

    /// Get the raw 32-bit word of the instruction.
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Get the immediate value of the Load-S type instruction, which is a 12-bit signed integer
    /// formed by concatenating imm[11:0].
    #[inline(always)]
    pub fn imm(&self) -> i32 {
        sign_extend(self.imm11_0() as i32, 12)
    }
}

/// Opcode for Load-S type instructions.
/// This is a constant value that can be used to identify Load-S type instructions when decoding.
pub const LOADSTYPE_OPCODE: u32 = 0b0000011;

/// Mask for the differentiating fields of Load-S type instructions (opcode, funct3).
const LOADSTYPE_PATTERN_MASK: u32 = LOADSTYPE_OPCODE_MASK | LOADSTYPE_FUNCT3_MASK;

/// Load-S type instruction function codes for different operations.
macro_rules! loadstype_instr {
    ($mnemonic:ident, $func3:expr) => {
        paste::paste! {
            /// Function code discriminent (funct3) for the $mnemonic Load-S type instruction.
            pub const [<LOADSTYPE_INSTR_ $mnemonic:upper _FUNCT3>]: u32 = $func3;

            /// [<$mnemonic:camel LoadSType>] is a representation of the Load-S type instruction $mnemonic,
            /// which includes methods for validating and matching the instruction based on
            /// its opcode, funct3.
            #[derive(Debug, Clone, Copy)]
            #[repr(transparent)]
            pub struct [<$mnemonic:camel LoadSType>] (LoadSType);

            impl [<$mnemonic:camel LoadSType>] {
                /// Creates a new instance of the instruction from a 32-bit word.
                pub const fn new(word: u32) -> Self {
                    Self(LoadSType::new(word))
                }
            }

            impl Deref for [<$mnemonic:camel LoadSType>] {
                type Target = LoadSType;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl [<$mnemonic:camel LoadSType>] {
                pub const fn expected_pattern() -> u32 {
                    (LOADSTYPE_OPCODE << LOADSTYPE_OPCODE_OFFSET) | ([<LOADSTYPE_INSTR_ $mnemonic:upper _FUNCT3>] << LOADSTYPE_FUNCT3_OFFSET)
                }
            }
        }
    };
}

loadstype_instr!(lb, 0b000);
loadstype_instr!(lh, 0b001);
loadstype_instr!(lw, 0b010);
loadstype_instr!(lbu, 0b100);
loadstype_instr!(lhu, 0b101);

/// Execute function for Load-S type instructions, this will be called by the execute method of each 
/// specific Load-S instruction (e.g., LbLoadSType) to perform the actual execution steps.
fn execute_load(instr: Instr, regs: &mut crate::reg::Regs32, state: &mut crate::instr::InstrState) {
    // SAFETY: The caller must ensure that the provided `Instr` is indeed an `LoadSType` instruction.
    let instr = LoadSType::new(unsafe { instr.raw }); 
    let addr = regs.read(instr.rs1() as usize).wrapping_add(instr.imm() as u32);
    state.val_c = addr;
    state.val_mem = instr.rd() as u32; // Store the destination register index in val_mem for the memory load step to use.
}


macro_rules! load_execute {
    ($mnemonic:ident, $func3:expr, $mem_step:ident) => {
        paste::paste! {
            impl Execute for [<$mnemonic:camel LoadSType>] {
                fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
                    steps[0] = InstrStep::Call(execute_load);
                    steps[1] = InstrStep::$mem_step;
                    steps[2] = InstrStep::IncPc32;
                    3 // Number of steps filled in the steps array
                }
            }
        }
    }
}

load_execute!(lb, 0b000, MemLoadByte);
load_execute!(lh, 0b001, MemLoadHalf);
load_execute!(lw, 0b010, MemLoadWord);
load_execute!(lbu, 0b100, MemLoadUnsignedByte);
load_execute!(lhu, 0b101, MemLoadUnsignedHalf);

/// Store-S instruction format
/// ```text
/// 31        25 24  20 19  15 14   12 11     7 6    0
/// imm[11:5]    rs2    rs1    funct3  imm[4:0] opcode
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct StoreSType(u32);

instr_field!(StoreSType, opcode, 0, 7);
instr_field!(StoreSType, imm4_0, 7, 5);
instr_field!(StoreSType, funct3, 12, 3);
instr_field!(StoreSType, rs1, 15, 5);
instr_field!(StoreSType, rs2, 20, 5);
instr_field!(StoreSType, imm11_0, 25, 7);

impl StoreSType {
    /// Creates a new Store-S type instruction from a 32-bit word.
    pub const fn new(word: u32) -> Self {
        Self(word)
    }

    /// Get the raw 32-bit word of the instruction.
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Get the immediate value of the Store-S type instruction, which is a 12-bit signed integer
    /// formed by concatenating imm[11:5] and imm[4:0].
    #[inline(always)]
    pub fn imm(&self) -> i32 {
        let imm = ((self.imm11_0() << 5) | self.imm4_0()) as i32; // imm[11:5] and imm[4:0]
        sign_extend(imm, 12)
    }
}

/// Opcode for Store-S type instructions.
/// This is a constant value that can be used to identify Store-S type instructions when decoding.
pub const STORESTYPE_OPCODE: u32 = 0b0100011;

/// Mask for the differentiating fields of Store-S type instructions (opcode, funct3).
const STORESTYPE_PATTERN_MASK: u32 = STORESTYPE_OPCODE_MASK | STORESTYPE_FUNCT3_MASK;

/// Store-S type instruction function codes for different operations.
macro_rules! storestype_instr {
    ($mnemonic:ident, $func3:expr) => {
        paste::paste! {
            /// Function code discriminent (funct3) for the $mnemonic Store-S type instruction.
            pub const [<STORESTYPE_INSTR_ $mnemonic:upper _FUNCT3>]: u32 = $func3;

            /// [<$mnemonic:camel StoreSType>] is a representation of the Store-S type instruction $mnemonic,
            /// which includes methods for validating and matching the instruction based on
            /// its opcode, funct3.
            #[derive(Debug, Clone, Copy)]
            #[repr(transparent)]
            pub struct [<$mnemonic:camel StoreSType>] (StoreSType);

            impl [<$mnemonic:camel StoreSType>] {
                /// Creates a new instance of the instruction from a 32-bit word.
                pub const fn new(word: u32) -> Self {
                    Self(StoreSType::new(word))
                }
            }

            impl Deref for [<$mnemonic:camel StoreSType>] {
                type Target = StoreSType;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl [<$mnemonic:camel StoreSType>] {
                pub const fn expected_pattern() -> u32 {
                    (STORESTYPE_OPCODE << STORESTYPE_OPCODE_OFFSET) | ([<STORESTYPE_INSTR_ $mnemonic:upper _FUNCT3>] << STORESTYPE_FUNCT3_OFFSET)
                }
            }
        }
    };
}

storestype_instr!(sb, 0b000);
storestype_instr!(sh, 0b001);
storestype_instr!(sw, 0b010);

/// Execute function for Store-S type instructions, this will be called by the execute method of each 
/// specific Store-S instruction (e.g., SbStoreSType) to perform the actual execution steps.
fn execute_store(instr: Instr, regs: &mut crate::reg::Regs32, state: &mut crate::instr::InstrState) {
    // SAFETY: The caller must ensure that the provided `Instr` is indeed an `StoreSType` instruction.
    let instr = StoreSType::new(unsafe { instr.raw }); 
    let addr = regs.read(instr.rs1() as usize).wrapping_add(instr.imm() as u32);
    state.val_c = addr;
    state.val_mem = instr.rs2() as u32; // Store the source register index in val_mem for the memory store step to use.
}


macro_rules! store_execute {
    ($mnemonic:ident, $func3:expr, $mem_step:ident) => {
        paste::paste! {
            impl Execute for [<$mnemonic:camel StoreSType>] {
                fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
                    steps[0] = InstrStep::Call(execute_store);
                    steps[1] = InstrStep::$mem_step;
                    steps[2] = InstrStep::IncPc32;
                    3 // Number of steps filled in the steps array
                }
            }
        }
    }
}

store_execute!(sb, 0b000, MemStoreByte);
store_execute!(sh, 0b001, MemStoreHalf);
store_execute!(sw, 0b010, MemStoreWord);

/// Union of all S-type instructions for easy matching and decoding.
#[derive(Clone, Copy)]
#[repr(C)]
pub union InstrS {
    pub raw: u32,
    pub lb: LbLoadSType,
    pub lh: LhLoadSType,
    pub lw: LwLoadSType,
    pub lbu: LbuLoadSType,
    pub lhu: LhuLoadSType,
    pub sb: SbStoreSType,
    pub sh: ShStoreSType,
    pub sw: SwStoreSType,
}

impl PartialEq for InstrS {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.raw == other.raw }
    }
}

impl Eq for InstrS {}

impl Execute for InstrS {
    fn execute(&self, steps: &mut [InstrStep; 8]) -> usize {
        let ityp = unsafe { self.raw } & LOADSTYPE_PATTERN_MASK;
        match ityp {
            x if x == LbLoadSType::expected_pattern() => unsafe { self.lb.execute(steps) },
            x if x == LhLoadSType::expected_pattern() => unsafe { self.lh.execute(steps) },
            x if x == LwLoadSType::expected_pattern() => unsafe { self.lw.execute(steps) },
            x if x == LbuLoadSType::expected_pattern() => unsafe { self.lbu.execute(steps) },
            x if x == LhuLoadSType::expected_pattern() => unsafe { self.lhu.execute(steps) },
            x if x == SbStoreSType::expected_pattern() => unsafe { self.sb.execute(steps) },
            x if x == ShStoreSType::expected_pattern() => unsafe { self.sh.execute(steps) },
            x if x == SwStoreSType::expected_pattern() => unsafe { self.sw.execute(steps) },
            _ => {
                // If the instruction does not match any known S type instruction, we can treat it as an invalid instruction.
                steps[0] = InstrStep::TrapInvalidInstruction;
                1 // Number of steps filled in the steps array
            }
        }
    }
}   