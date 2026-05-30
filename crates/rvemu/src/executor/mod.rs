pub mod rv32i;

pub trait Executor {
    type Address;
    type MemoryController;
    type Regs;
    type ExecutionError;

    fn new(memory: Self::MemoryController, regs: Self::Regs) -> Self;
    fn break_down(self) -> (Self::MemoryController, Self::Regs);

    fn memory(&self) -> &Self::MemoryController;
    fn memory_mut(&mut self) -> &mut Self::MemoryController;

    fn regs(&self) -> &Self::Regs;
    fn regs_mut(&mut self) -> &mut Self::Regs;

    fn read_pc(&self) -> Self::Address;
    fn jump_to(&mut self, addr: Self::Address);
    fn step(&mut self) -> Result<(), Self::ExecutionError>;
}