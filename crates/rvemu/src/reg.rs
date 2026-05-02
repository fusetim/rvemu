use crate::data::Word;

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

    #[inline(always)]
    pub fn read(&self, index: usize) -> Word {
        if index == 0 {
            0
        } else {
            self.regs[index]
        }
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

