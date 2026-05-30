use thiserror::Error;
use core::fmt::Debug;

pub type Byte = u8;
pub type Half = u16;
pub type Word = u32;
pub type DoubleWord = u64;

/// Memory controller trait for reading and writing data of various sizes.
///
/// This controller is used by the emulator to interact with memory, allowing it to read and write bytes, half-words, words, and double words.
/// This interface manages memory access for classic RAM and ROM access, but also MMIO peripherals.
pub trait MemoryController<Address> {
    /// Error type for memory operations, which must implement PartialEq, Eq, Debug, and Clone traits.
    type Error: PartialEq + Eq + Debug + Clone;

    fn read_byte(&self, addr: Address) -> MemoryResult<Byte, Self::Error>;
    fn read_half(&self, addr: Address) -> MemoryResult<Half, Self::Error>;
    fn read_word(&self, addr: Address) -> MemoryResult<Word, Self::Error>;
    fn read_double_word(&self, addr: Address) -> MemoryResult<DoubleWord, Self::Error>;

    fn write_byte(&mut self, addr: Address, value: Byte) -> MemoryResult<(), Self::Error>;
    fn write_half(&mut self, addr: Address, value: Half) -> MemoryResult<(), Self::Error>;
    fn write_word(&mut self, addr: Address, value: Word) -> MemoryResult<(), Self::Error>;
    fn write_double_word(
        &mut self,
        addr: Address,
        value: DoubleWord,
    ) -> MemoryResult<(), Self::Error>;
}

/// Memory exception type for handling errors during memory access.
///
/// This enum represents different types of memory exceptions that can occur during read
/// and write operations, such as invalid addresses, read-only addresses, and write-only addresses.
///
/// The generic parameter `E` allows for custom error types to be used in the exception handling, while still
/// enabling the emulator to use the correct trapping mechanism.
#[non_exhaustive]
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MemoryException<E>
where
    E: PartialEq + Eq + Debug + Clone,
{
    #[error("Invalid memory address: {0:?}")]
    InvalidAddress(E),
    #[error("Read-only memory address: {0:?}")]
    ReadOnlyAddress(E),
    #[error("Write-only memory address: {0:?}")]
    WriteOnlyAddress(E),
    #[error("Unsupported operation at memory address: {0:?}")]
    UnsupportedOperation(E),
    // Other exceptions can be added as needed
}

/// Result type for memory operations, using the MemoryException for error handling.
pub type MemoryResult<T, E> = Result<T, MemoryException<E>>;

impl<E> Copy for MemoryException<E> where E: PartialEq + Eq + Debug + Clone + Copy {}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimalMemoryController<const SIZE: usize> {
    memory: [Byte; SIZE],
}

impl<const SIZE: usize> MinimalMemoryController<SIZE> {
    pub const fn new() -> Self {
        Self { memory: [0; SIZE] }
    }

    pub const fn from_array(memory: [Byte; SIZE]) -> Self {
        Self { memory }
    }

    pub fn copy_from_slice(memory: &[Byte]) -> Self {
        let mut mem = [0; SIZE];
        mem[0..memory.len()].copy_from_slice(memory);
        Self { memory: mem }
    }

    pub const fn as_array(&self) -> &[Byte; SIZE] {
        &self.memory
    }

    pub const fn consume(self) -> [Byte; SIZE] {
        self.memory
    }
}

impl<const SIZE: usize> MemoryController<Word> for MinimalMemoryController<SIZE> {
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