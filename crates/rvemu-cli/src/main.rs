use std::{
    fs::File, io::Read as _,
};

use rvemu::{
    data::{MemoryController as _, MinimalMemoryController}, executor::{Executor, rv32i::ExecutorRV32I}, instr::Instr, reg::Regs32
};

fn main() {
    // Get input executable file from command line arguments
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Usage: {} <executable_file>", args[0]);
        std::process::exit(1);
    }

    println!("Loading executable file: {}", args[1]);

    let mut exe = File::open(&args[1]).expect("Failed to open executable file");
    let mut instrs = Vec::new();
    let mut buf = [0u8; 4];
    while exe.read_exact(&mut buf).is_ok() {
        instrs.push(u32::from_le_bytes(buf));
    }

    println!("Loaded {} instructions from {}", instrs.len(), args[1]);

    let mut regs: Regs32 = Regs32::with(
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ],
        0,
    );

    let mut memory = MinimalMemoryController::<4096>::new();
    // Copy the program to the memory end so that it doesn't interfere with the initial register values
    let program_start = 4096 - (instrs.len() * 4) as u32;
    for (i, instr) in instrs.iter().enumerate() {
        let addr = program_start + (i as u32 * 4);
        memory.write_word(addr, *instr).expect("Failed to write instruction to memory");
    }

    let mut executor = ExecutorRV32I::new(memory, regs);
    executor.jump_to(program_start);

    while (executor.read_pc() as usize) < 4096 {
        if let Err(e) = executor.step() {
            eprintln!("Execution error at PC=0x{:08x}: {:?}", executor.read_pc(), e);
            break;
        }
    }

    // Print all registers
    for i in 0..32 {
        println!("r{} -> {:08x}", i, executor.regs().read(i));
    }

    // Print the first 64 bytes of memory
    for i in 0..16 {
        let addr = i * 4;
        let value = executor.memory().read_word(addr).unwrap_or(0);
        println!("0x{:08x}: {:08x}", addr, value);
    }
}
