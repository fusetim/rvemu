use thiserror::Error;

use crate::{data::{MemoryController, MemoryException, Word}, executor::Executor, instr::{Execute as _, Instr, InstrState, InstrStep}, reg::Regs32};


#[derive(Debug, Clone, PartialEq)]
pub struct ExecutorRV32I<M: MemoryController<Word>> {
    memory: M,
    regs: Regs32,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ExecutorRV32IError<M: MemoryController<Word>> {
    #[error("Invalid instruction encountered")]
    InvalidInstruction,
    #[error("Memory access error")]
    MemoryAccessError(MemoryException<M::Error>),
    // Other execution errors can be added here
    #[error("Unexpected error: {0}")]
    Unexpected(&'static str),
}

impl<M> Executor for ExecutorRV32I<M> 
where M: MemoryController<Word> {
    type Address = Word;
    type MemoryController = M;
    type Regs = Regs32;
    type ExecutionError = ExecutorRV32IError<M>;

    fn new(memory: Self::MemoryController, regs: Self::Regs) -> Self {
        Self { memory, regs }
    }

    fn break_down(self) -> (Self::MemoryController, Self::Regs) {
        (self.memory, self.regs)
    }

    fn memory(&self) -> &Self::MemoryController {
        &self.memory
    }

    fn memory_mut(&mut self) -> &mut Self::MemoryController {
        &mut self.memory
    }

    fn regs(&self) -> &Self::Regs {
        &self.regs
    }

    fn regs_mut(&mut self) -> &mut Self::Regs {
        &mut self.regs
    }

    fn read_pc(&self) -> Self::Address {
        self.regs.read_pc()
    }

    fn jump_to(&mut self, addr: Self::Address) {
        self.regs.write_pc(addr);
    }

    fn step(&mut self) -> Result<(), Self::ExecutionError> {
        // Get the current PC and fetch the instruction from memory
        let pc: u32 = self.regs.read_pc();
        let instr = self.memory.read_word(pc)
            .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
        let instr = Instr { raw: instr };

        // Get the instruction execution steps and execute them
        let regs = &mut self.regs;
        let mut state = InstrState::new();
        let mut steps = [InstrStep::Noop; 8];
        let steps_filled = instr.execute(&mut steps);
        for step in &steps[0..steps_filled] {
            match step {
                InstrStep::Call(func) => {
                    func(instr, regs, &mut state);
                }
                InstrStep::Jump => {
                    // JumpAddress is put in val_c of the InstrState by the instruction execution function,
                    // so we need to read it from there and write it to the PC register to perform the jump.
                    regs.write_pc(state.val_c);
                }
                InstrStep::MemLoadWord => {
                    let addr = state.val_c as usize;
                    let rd = state.val_mem as usize;
                    let value = self.memory.read_word(addr as u32)
                        .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
                    regs.write(rd, value);
                },
                InstrStep::MemLoadByte => {
                    let addr = state.val_c as usize;
                    let rd = state.val_mem as usize;
                    let value = self.memory.read_byte(addr as u32)
                        .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
                    let value = value as i8 as i32 as u32; // Sign-extend the byte to 32 bits
                    regs.write(rd, value);
                },
                InstrStep::MemLoadHalf => {
                    let addr = state.val_c as usize;
                    let rd = state.val_mem as usize;
                    let value = self.memory.read_half(addr as u32)
                        .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
                    let value = value as i16 as i32 as u32; // Sign-extend the half-word to 32 bits
                    regs.write(rd, value);
                },
                InstrStep::MemLoadUnsignedByte => {
                    let addr = state.val_c as usize;
                    let rd = state.val_mem as usize;
                    let value = self.memory.read_byte(addr as u32)
                        .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
                    let value = value as u32; // Zero-extend the byte to 32 bits
                    regs.write(rd, value);
                },
                InstrStep::MemLoadUnsignedHalf => {
                    let addr = state.val_c as usize;
                    let rd = state.val_mem as usize;
                    let value = self.memory.read_half(addr as u32)
                        .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
                    let value = value as u32; // Zero-extend the half-word to 32 bits
                    regs.write(rd, value);
                },
                InstrStep::MemStoreWord => {
                    let addr = state.val_c as usize;
                    let rs = state.val_mem as usize;
                    let value = regs.read(rs);
                    self.memory.write_word(addr as u32, value)
                        .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
                },
                InstrStep::MemStoreByte => {
                    let addr = state.val_c as usize;
                    let rs = state.val_mem as usize;
                    let value = regs.read(rs) as u8; // Take only the least significant byte
                    self.memory.write_byte(addr as u32, value)
                        .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
                },
                InstrStep::MemStoreHalf => {
                    let addr = state.val_c as usize;
                    let rs = state.val_mem as usize;
                    let value = regs.read(rs) as u16; // Take only the least significant half-word
                    self.memory.write_half(addr as u32, value)
                        .map_err(|mem_err| Self::ExecutionError::MemoryAccessError(mem_err))?;
                },
                InstrStep::IncPc32 => regs.inc_pc(4),
                InstrStep::TrapInvalidInstruction => {
                    return Err(Self::ExecutionError::InvalidInstruction);
                }
                _ => {
                    return Err(Self::ExecutionError::Unexpected("Unsupported instruction step encountered"));
                }
            }
        }

        Ok(())
    }
}