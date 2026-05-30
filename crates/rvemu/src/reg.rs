use crate::data::Word;

#[derive(Debug, Clone, PartialEq)]
pub struct Regs32 {
    regs: [Word; 32],
    pc: Word,
}

impl Regs32 {
    pub const fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
        }
    }

    pub const fn with_pc(pc: Word) -> Self {
        Self { regs: [0; 32], pc }
    }

    /// Create a new Regs32 with the given register values and program counter.
    ///
    /// # Arguments
    ///
    /// * `regs` - An array of 32 Word values representing the initial state of the registers (x0 to x31).
    /// * `pc` - A Word value representing the initial state of the program counter.
    ///
    /// # Returns
    ///
    /// A new instance of Regs32 initialized with the provided register values and program counter.
    ///
    /// # Important
    ///
    /// The x0 register is hardwired to zero in RISC-V, so any value provided for x0 in the `regs` array will be ignored
    /// and treated as zero. The `write` method will also ensure that x0 remains zero regardless of any attempts to write to it.
    pub const fn with(mut regs: [Word; 32], pc: Word) -> Self {
        regs[0] = 0;
        Self { regs, pc }
    }

    #[inline(always)]
    pub fn read(&self, index: usize) -> Word {
        self.regs[index]
    }

    #[inline(always)]
    pub fn write(&mut self, index: usize, value: Word) {
        if index != 0 {
            self.regs[index] = value;
        }
    }

    #[inline(always)]
    pub fn read_pc(&self) -> Word {
        self.pc
    }

    #[inline(always)]
    pub fn write_pc(&mut self, value: Word) {
        self.pc = value;
    }

    #[inline(always)]
    pub fn inc_pc(&mut self, value: Word) {
        self.pc = self.pc.wrapping_add(value);
    }
}
