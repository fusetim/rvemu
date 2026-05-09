	.section .text
	.globl _start

_start:
    # Initializing some registers to compare
    addi x1, x0, 0x01           # x1 <- 1
    addi x2, x0, 0x02           # x2 <- 2
    addi x3, x0, 0x03           # x3 <- 3
    addi x4, x0, -1             # x4 <- -1  (0xFFFFFFFF in 32-bit)
    addi x5, x0, -16            # x5 <- -16 (0xFFFFFFF0 in 32-bit)
    addi x6, x0, 16             # x6 <- 16
    addi x7, x0, 32             # x7 <- 32
    # Put the flag registers to zero
    addi x10, x0, 0             # x10 will be used to flag BEQ test failures
    addi x11, x0, 0             # x11 will be used to flag BNE test failures
    addi x12, x0, 0             # x12 will be used to flag BLT test failures
    addi x13, x0, 0             # x13 will be used to flag BGE test failures
    addi x14, x0, 0             # x14 will be used to flag BLTU test failures
    addi x15, x0, 0             # x15 will be used to flag BGEU test failures

    nop
    nop
    
_test_beq:
    beq x1, x1, _test_beq_pass1     # Should branch (1 == 1)
    addi x10, x0, 0x01              # Should not be here, flag it in x10
_test_beq_pass1:
    beq x1, x2, _test_beq_fail      # Should not branch (1 != 2)
    beq x0, x0, _test_beq_pass2     # Should branch (0 == 0)
    addi x10, x0, 0x04              # Should not be here, flag it in x10
_test_beq_pass2:
    # Next test, BNE
    bne x1, x2, _test_bne_pass1     # Should branch (1 != 2)
    addi x11, x0, 0x01              # Should not be here, flag it in x11
_test_bne_pass1:
    bne x1, x1, _test_bne_fail      # Should not branch (1 == 1)
    bne x5, x7, _test_bne_pass2     # Should branch (-16 != 32)
    addi x11, x0, 0x02              # Should not be here
_test_bne_pass2:
    bne x0, x0, _test_bne_fail      # Should not branch (0 == 0)
    # Next test, BLT
    blt x1, x2, _test_blt_pass1     # Should branch (1 < 2)
    addi x12, x0, 0x01              # Should not be here, flag it in x12
_test_blt_pass1:
    blt x2, x1, _test_blt_fail      # Should not branch (2 < 1)
    blt x4, x1, _test_blt_pass2     # Should branch (-1 < 1)
    addi x12, x0, 0x02              # Should not be here, flag it in x12
_test_blt_pass2:
    blt x1, x4, _test_blt_fail      # Should not branch (1 < -1)
    # Next test, BGE
    bge x2, x1, _test_bge_pass1     # Should branch (2 >= 1)
    addi x13, x0, 0x01              # Should not be here, flag it in x13
_test_bge_pass1:
    bge x1, x2, _test_bge_fail      # Should not branch (1 >= 2)
    bge x1, x1, _test_bge_pass2     # Should branch (1 >= 1)
    addi x13, x0, 0x02              # Should not be here, flag it in x13
_test_bge_pass2:
    bge x4, x1, _test_bge_fail      # Should not branch (-1 >= 1)
    # Next test, BLTU
    bltu x1, x2, _test_bltu_pass1   # Should branch (1 < 2 unsigned)
    addi x14, x0, 0x01              # Should not be here, flag it in x14
_test_bltu_pass1:
    bltu x2, x1, _test_bltu_fail    # Should not branch (2 < 1 unsigned)
    bltu x4, x1, _test_bltu_fail    # Should not branch (0xFFFFFFFF < 1 unsigned)
    # Next test, BGEU
    bgeu x2, x1, _test_bgeu_pass1   # Should branch (2 >= 1 unsigned)
    addi x15, x0, 0x01              # Should not be here, flag it in x15
_test_bgeu_pass1:
    bgeu x1, x2, _test_bgeu_fail    # Should not branch (1 >= 2 unsigned)
    bgeu x4, x1, _test_bgeu_pass2   # Should branch (0xFFFFFFFF >= 1 unsigned)
    addi x15, x0, 0x02              # Should not be here, flag it in x15
_test_bgeu_pass2:
    bgeu x1, x4, _test_bgeu_fail    # Should not branch (1 >= 0xFFFFFFFF unsigned)
_just_some_space:
    nop
    nop
    nop
    nop
    beq x0, x0, _end                # Jump to end
    nop
    nop
_test_beq_fail:
    addi x10, x0, 0x02              # Flag fail in x10
    beq x0, x0, _end                # Early end
_test_bne_fail:
    addi x11, x0, 0x80              # Flag fail in x11
    beq x0, x0, _end                # Early end
_test_blt_fail:
    addi x12, x0, 0x80              # Flag fail in x12
    beq x0, x0, _end                # Early end
_test_bge_fail:
    addi x13, x0, 0x80              # Flag fail in x13
    beq x0, x0, _end                # Early end
_test_bltu_fail:
    addi x14, x0, 0x80              # Flag fail in x14
    beq x0, x0, _end                # Early end
_test_bgeu_fail:
    addi x15, x0, 0x80              # Flag fail in x15
    beq x0, x0, _end                # Early end
_end:
    # If we reach here, the test passed, x10 should be 0
    nop
