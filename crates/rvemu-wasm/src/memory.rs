use alloc::vec::Vec;
use rvemu::data::{Byte, DoubleWord, Half, MemoryController, MemoryException, MemoryResult, Word};


#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMemoryController {
    memory: Vec<Byte>,
}

impl SimpleMemoryController {
    pub const fn new() -> Self {
        Self { memory: Vec::new() }
    }

    pub const fn from_vec(memory: Vec<Byte>) -> Self {
        Self { memory }
    }
}

impl MemoryController<Word> for SimpleMemoryController {
    type Error = ();

    fn read_byte(&self, addr: Word) -> MemoryResult<Byte, Self::Error> {
        let addr = addr as usize;
        if addr < self.memory.len() {
            Ok(self.memory[addr])
        } else {
            Err(MemoryException::InvalidAddress(()))
        }
    }

    fn read_half(&self, addr: Word) -> MemoryResult<Half, Self::Error> {
        let addr = addr as usize;
        if addr + 1 < self.memory.len() {
            let low = self.memory[addr] as Half;
            let high = self.memory[addr + 1] as Half;
            Ok((high << 8) | low)
        } else {
            Err(MemoryException::InvalidAddress(()))
        }
    }

    fn read_word(&self, addr: Word) -> MemoryResult<Word, Self::Error> {
        let addr = addr as usize;
        if addr + 3 < self.memory.len() {
            let b0 = self.memory[addr] as Word;
            let b1 = self.memory[addr + 1] as Word;
            let b2 = self.memory[addr + 2] as Word;
            let b3 = self.memory[addr + 3] as Word;
            Ok((b3 << 24) | (b2 << 16) | (b1 << 8) | b0)
        } else {
            Err(MemoryException::InvalidAddress(()))
        }
    }

    fn read_double_word(&self, _addr: Word) -> MemoryResult<DoubleWord, Self::Error> {
        Err(MemoryException::UnsupportedOperation(()))
    }

    fn write_byte(&mut self, addr: Word, value: Byte) -> MemoryResult<(), Self::Error> {
        let addr = addr as usize;
        if addr < self.memory.len() {
            self.memory[addr] = value;
            Ok(())
        } else {
            Err(MemoryException::InvalidAddress(()))
        }
    }

    fn write_half(&mut self, addr: Word, value: Half) -> MemoryResult<(), Self::Error> {
        let addr = addr as usize;
        if addr + 1 < self.memory.len() {
            self.memory[addr] = (value & 0xFF) as u8;
            self.memory[addr + 1] = ((value >> 8) & 0xFF) as u8;
            Ok(())
        } else {
            Err(MemoryException::InvalidAddress(()))
        }
    }

    fn write_word(&mut self, addr: Word, value: Word) -> MemoryResult<(), Self::Error> {
        let addr = addr as usize;
        if addr + 3 < self.memory.len() {
            self.memory[addr] = (value & 0xFF) as u8;
            self.memory[addr + 1] = ((value >> 8) & 0xFF) as u8;
            self.memory[addr + 2] = ((value >> 16) & 0xFF) as u8;
            self.memory[addr + 3] = ((value >> 24) & 0xFF) as u8;
            Ok(())
        } else {
            Err(MemoryException::InvalidAddress(()))
        }
    }

    fn write_double_word(&mut self, _addr: Word, _value: DoubleWord) -> MemoryResult<(), Self::Error> {
        return Err(MemoryException::UnsupportedOperation(()));
    }
}