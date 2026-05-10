	.section .text
	.globl _start

_start:
    # Zero out the "flag registers" to indicate no failures yet (x10-x15)
    addi x10, x0, 0
    addi x11, x0, 0
    addi x12, x0, 0
    addi x13, x0, 0
    addi x14, x0, 0
    addi x15, x0, 0

    # Test LUI instruction
    lui x1, 0x12345                     # Load upper immediate, x1 should be 0x12345000
    addi x2, x0, 0x12                   # Construct the expected value in x2
    slli x2, x2, 8                      # x2 <- 0x1200
    addi x2, x2, 0x34                   # x2 <- 0x1234
    slli x2, x2, 8                      # x2 <- 0x123400
    addi x2, x2, 0x50                    # x2 <- 0x123450
    slli x2, x2, 8                      # x2 <- 0x12345000
    beq  x1, x2, _test_lui_success      # Check if x1 == expected value in x2
    addi x10, x10, 0x01                  # If not, flag failure in x10
_test_lui_success:
    # Test LUI with a negative immediate
    lui x3, 0xFFFFF                     # Load upper immediate, x3 should be 0xFFFFF000
    addi x4, x0, -1                     # Construct the expected value in x4 (0xFFFFFFFF)
    slli x4, x4, 12                     # x4 <- 0xFFFFF000 (sign-extended)
    beq  x3, x4, _test_lui_neg_success  # Check if x3 == expected value in x4
    addi x10, x10, 0x02                  # If not, flag failure in x10
_test_lui_neg_success:
    nop
    # Test AUIPC instruction
    auipc x5, 0x00000                       # AUIPC with 0 should just give the current PC
    addi x6, x0, 0x54                       # Construct the expected value in x6
    bne x5, x6, _test_auipc_fail1           # Check if x5 == expected value in x6
    auipc x7, 0x00001                       # AUIPC with 1 should give current PC + 0x1000
    lui   x8, 0x00001                       # Construct the expected value in x8 (current PC + 0x1000)
    addi x8, x8, 0x60                       # Construct the expected value in x8
    bne x7, x8, _test_auipc_fail2           # Check if x7 == expected value in x8
    nop
    j _end
_test_auipc_fail1:
    addi x11, x11, 0x04                      # Flag failure in x11
    j _end
_test_auipc_fail2:
    addi x11, x11, 0x08                      # Flag failure in x11
_end:
    nop


