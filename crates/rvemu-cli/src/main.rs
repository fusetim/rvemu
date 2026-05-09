use std::{fs::File, io::{Read as _, Write as _}};

use rvemu::{
    instr::{Execute, Instr, InstrState, InstrStep, rtype::*},
    reg::Regs32,
};

    const R_INSTRS: [Instr; 11] = [
        AddRInstr::new(0x00820133).to_instr(),    //  add x2, x4, x8  |  x2 <- x4 + x8            |  x2 <- 4 + 8 = 12
        SubRInstr::new(0x402200b3).to_instr(),    //  sub x1, x4, x2  |  x1 <- x4 - x2            |  x1 <- 4 - 12 = 0xFFFFFFF8 (wrapping around)
        SllRInstr::new(0x00111133).to_instr(),    //  sll x2, x2, x1  |  x2 <- x2 << (x1 & 0x1F)  |  x2 <- 12 << (0xFFFFFFF8 & 0x1F) = 12 << 24 = 0x0C000000
        SltRInstr::new(0x00212233).to_instr(),    //  slt x4, x2, x2  |  x4 <- (x2 < x2) ? 1 : 0  |  x4 <- (0x0C000000 < 0x0C000000) ? 1 : 0 = 0
        SltRInstr::new(0x00412233).to_instr(),    //  slt x4, x2, x4  |  x4 <- (x2 < x4) ? 1 : 0  |  x4 <- (0x0C000000 < 0) ? 1 : 0 = 0
        SltuRInstr::new(0x00213233).to_instr(),   //  sltu x4, x2, x2 |  x4 <- (x2 < x2) ? 1 : 0  |  x4 <- (0x0C000000 < 0x0C000000) ? 1 : 0 = 0
        XorRInstr::new(0x0025c533).to_instr(),    //  xor x10, x11, x2|  x10 <- x11 ^ x2          | x10 <- 11 ^ 0x0C000000 = 0x0BFFFFF5
        SrlRInstr::new(0x001111b3).to_instr(),    //  srl x3, x2, x1  |  x3 <- x2 >> (x1 & 0x1F)  |  x3 <- 0x0C000000 >> (0xFFFFFFF8 & 0x1F) = 0x0C000000 >> 24 = 12
        SraRInstr::new(0x401151b3).to_instr(),    //  sra x3, x2, x1  |  x3 <- x2 >> (x1 & 0x1F)  |  x3 <- 0x0C000000 >> (0xFFFFFFF8 & 0x1F) = 0x0C000000 >> 24 = 12 (same as srl since the value is positive and the sign bit is 0)
        OrRInstr::new(0x0025d533).to_instr(),     //  or x10, x11, x2 |  x10 <- x11 | x2          | x10 <- 11 | 0x0C000000 = 0x0C00000B 
        AndRInstr::new(0x0025e533).to_instr(),    //  and x10, x11, x2|  x10 <- x11 & x2          | x10 <- 11 & 0x0C000000 = 0
    ];


#[inline(never)]
fn run(instrs: &[Instr], regs: &mut Regs32) {
    let mut steps = [InstrStep::Noop; 8];
    let mut steps_filled ;
    while (regs.read_pc() as usize) < instrs.len() * 4 {
        // Fetch the instruction at the current PC
        let instr_index = (regs.read_pc() / 4) as usize;
        let instr = instrs[instr_index];
        let mut state = InstrState::new();

        steps_filled = instr.execute(&mut steps);
        for step in &steps[0..steps_filled] {
            match step {
                InstrStep::Call(func) => {
                    func(instr, regs, &mut state)
                }
                InstrStep::Jump => {
                    // JumpAddress is put in val_c of the InstrState by the instruction execution function, 
                    // so we need to read it from there and write it to the PC register to perform the jump.
                    println!("Jumping: from: 0x{:08x} to: 0x{:08x}", regs.read_pc(), state.val_c);
                    regs.write_pc(state.val_c);
                }
                InstrStep::TrapInvalidInstruction => panic!("Invalid instruction encountered during execution, pc: 0x{:08x}, instr: 0x{:08x}", regs.read_pc(), unsafe { instr.raw }),
                _ => panic!("Unexpected instruction step, pc: 0x{:08x}, instr: 0x{:08x}", regs.read_pc(), unsafe { instr.raw }),
            }
        }
    }
}

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
        let instr = Instr { raw: u32::from_le_bytes(buf) };
        instrs.push(instr);
    }

    println!("Loaded {} instructions from {}", instrs.len(), args[1]);

    let mut regs: Regs32 = Regs32::with([0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31], 0);

    run(&instrs, &mut regs);
    
    // Print all registers
    for i in 0..32 {
        println!("r{} -> {:08x}", i, regs.read(i));
    }
}
