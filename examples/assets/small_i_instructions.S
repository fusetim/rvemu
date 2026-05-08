# Compile using `riscv32-elf-as -o examples/assets/small_i_instructions.elf examples/assets/small_i_instructions.S`
# and then convert to binary using `riscv32-elf-objcopy -O binary -o examples/assets/small_i_instructions.bin examples/assets/small_i_instructions.elf`

	.section .text
	.globl _start

_start:
	# addi: x1 = 5
	addi    x1, x0, 5
	# slti: x2 = (5 < 8) ? 1 : 0 = 1
	slti    x2, x1, 8
	# sltiu: x3 = (5 < 8) unsigned ? 1 : 0 = 1
	sltiu   x3, x1, 8
	# xori: x4 = 5 ^ 0xF = 0xA
	xori    x4, x1, 0xF
	# ori: x5 = 5 | 0xA = 0xF
	ori     x5, x1, 0xA
	# andi: x6 = 5 & 0x3 = 0x1
	andi    x6, x1, 0x3

	# slli: x7 = 5 << 2 = 20
	slli    x7, x1, 2
	# srli: x8 = 20 >> 1 = 10
	srli    x8, x7, 1
	# srai: x9 = 0xFFFFFFF0 >> 2 = 0xFFFFFFFC
	addi    x9, x0, -16
	srai    x9, x9, 2

	# addi x10 = 32 
	# addi x11 = x10 - 16 = 16 
	addi    x10, x0, 32
	addi    x11, x10, -16
