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
    fn write_double_word(&mut self, addr: Address, value: DoubleWord) -> MemoryResult<(), Self::Error>;
}

/// Memory exception type for handling errors during memory access.
/// 
/// This enum represents different types of memory exceptions that can occur during read 
/// and write operations, such as invalid addresses, read-only addresses, and write-only addresses. 
/// 
/// The generic parameter `E` allows for custom error types to be used in the exception handling, while still 
/// enabling the emulator to use the correct trapping mechanism.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryException<E>
where E: PartialEq + Eq + Debug + Clone  {
    InvalidAddress(E),
    ReadOnlyAddress(E),
    WriteOnlyAddress(E),
    // Other exceptions can be added as needed
}

/// Result type for memory operations, using the MemoryException for error handling.
pub type MemoryResult<T, E> = Result<T, MemoryException<E>>;

impl<E> Copy for MemoryException<E> where E: PartialEq + Eq + Debug + Clone + Copy {}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimalMemoryController<const SIZE: usize> {
    memory: [Byte; SIZE]
}

impl<const SIZE: usize> MinimalMemoryController<SIZE> {
    pub const fn new() -> Self {
        Self {
            memory: [0; SIZE]
        }
    }

    pub const fn from_array(memory: [Byte; SIZE]) -> Self {
        Self {
            memory
        }
    }

    pub fn copy_from_slice(memory: &[Byte]) -> Self {
        let mut mem = [0; SIZE];
        mem[0..memory.len()].copy_from_slice(memory);
        Self {
            memory: mem
        }
    }

    pub const fn as_array(&self) -> &[Byte; SIZE] {
        &self.memory
    }

    pub const fn consume(self) -> [Byte; SIZE] {
        self.memory
    }
}