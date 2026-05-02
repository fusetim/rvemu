# rvemu

> Supposedly a RISC-V emulator in Rust, that can run in the browser using WebAssembly, and hopefully with a fun demo to show off.

## Backstory

After some work on implementing my own RISC-V RV32I core in Verilog for a previous project, and having a lot of fun
making a small "MP3 player" that runs fully on FPGA using a RISC-V core, some custom peripherals brought together in an home-assembled SoC, and 
Rust software running on top of it (see [rusty-soc](https://github.com/fusetim/rusty-soc)), I would like to demonstrate it. 
So what better way to do it than writing a RISC-V emulator in Rust, and probably some stub peripherals, to run the same software, directly in your browser?

This is the goal of this project, and the name "rvemu" is a simple contraction of "RISC-V emulator", and definitely the least original name I could come up with.

## Status

Nothing works and pretty much everything is to be done, but the plan is here:

- [ ] Implement a simple RISC-V RV32I emulator library, with `![no_std]` support (a lot easier to port then to WASM).
- [ ] Create a mini front-end TUI to run the emulator in a terminal, and test it with some simple RISC-V binaries.
- [ ] Create a WASM front-end to run the emulator in the browser, and test it with the same RISC-V binaries.
- [ ] Implement some peripherals (GPIO, SPI,...), probably matching first the ones implemented in [rusty-soc](https://github.com/fusetim/rusty-soc), as I know everything about them, and test them with the same RISC-V binaries.
- [ ] Would be nice to have a minimal GDB-server implementation (I'm betting on you, [gdbstub](https://github.com/daniel5151/gdbstub))
- [ ] Make at least a fun demo
- [ ] Extend to the common RISC-V extensions (M, A, C, F, D,...), and maybe even the 64-bit architecture (RV64I), if I have the time and motivation.
- [ ] Should we support a real hardware platform, such as the ESP32 RISC-V variant? 

## Build and run

Well, currently everything is to be defined, so I guess you are on your own for now.  
I hope to have something working in the next few weeks, and I will update this section with instructions on how to build and run the emulator.

## License

This project and all code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer! 

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, shall be dual licensed as above, without any additional terms or conditions.