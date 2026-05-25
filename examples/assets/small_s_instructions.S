	.section .text
	.globl _start

_start:
    # Some initial 
    lui x1, 0xA1B2C          # x1 <- 0xA1B2C000
    addi x1, x1, 0x3E4       # x1 <- 0xA1B2C3E4
    lui x2, 0xFEDCB          # x2 <- 0xFEDBC000
    addi x2, x2, 0x123       # x2 <- 0xFEDBC123
    lui x3, 0x12345          # x3 <- 0x12345000
    
    # Put the flag registers to zero
    addi x10, x0, 0             # x10 will be used to flag LB test failures
    addi x11, x0, 0             # x11 will be used to flag LH test failures
    addi x12, x0, 0             # x12 will be used to flag LW test failures
    addi x13, x0, 0             # x13 will be used to flag LBU test failures
    addi x14, x0, 0             # x14 will be used to flag LHU test failures

    nop
    nop
    nop
    nop

    # Test word load-store instructions first to set up memory for the byte and half-word load tests
_test_word:
    sw x1, 0(x0)                # Store 0xA1B2C3E4 at address 0
    lw x4, 0(x0)                # Load from address 0 into x4, should get 0xA1B2C3E4
    bne x4, x1, _test_lw_fail   # If loaded value does not match original, flag it in x13

    sw x2, 4(x0)                # Store 0xFEDBC123 at address 4
    lw x5, 4(x0)                # Load from address 4 into x5, should get 0xFEDBC123
    bne x5, x2, _test_lw_fail   # If loaded value does not match original, flag it in x13

    sw x3, 8(x0)                # Store 0x12345678 at address 8
    lw x6, 8(x0)                # Load from address 8 into x6, should get 0x12345678
    bne x6, x3, _test_lw_fail   # If loaded value does not match original, flag it in x13

    # Zero them out again before the next tests
    sw x0, 0(x0)                # Store 0 at address 0
    sw x0, 4(x0)                # Store 0 at address 4
    sw x0, 8(x0)                # Store 0 at address 8
    # Check that the stores worked correctly by loading back and checking for zero
    lw x4, 0(x0)                # Load from address 0 into x4, should get 0
    bne x4, x0, _test_lw_fail   # If loaded value is not zero, flag it in x13
    lw x5, 4(x0)                # Load from address 4 into x5, should get 0
    bne x5, x0, _test_lw_fail   # If loaded value is not zero, flag it in x13
    lw x6, 8(x0)                # Load from address 8 into x6, should get 0
    bne x6, x0, _test_lw_fail   # If loaded value is not zero, flag it in x13
    j _test_byte                  # Jump to the byte load tests

_test_lw_fail:
    addi x13, x0, 0x80          # Flag LW test failure in x13
    j _test_lb                  # Jump to the byte load tests

_test_byte:
    # It is important to ensure endianness is correct for the byte load tests
    addi x3, x3, 0x678       # x3 <- 0x12345678
    addi x4, x0, 0x78        # x4 <- 0x78
    addi x5, x0, 0x56        # x5 <- 0x56
    addi x6, x0, 0x34        # x6 <- 0x34
    addi x7, x0, 0x12        # x7 <- 0x12
    sb x4, 0(x0)                # Store the least significant byte of x4 (0x78) at address 0
    sb x5, 1(x0)                # Store the least significant byte of x5 (0x56) at address 1
    sb x6, 2(x0)                # Store the least significant byte of x6 (0x34) at address 2
    sb x7, 3(x0)                # Store the least significant byte of x7 (0x12) at address 3
    lw x2, 0(x0)                # Load from address 0 into x2, should get 0x12345678
    bne x2, x3, _test_lb_fail   # If loaded value does not match original, flag it in x10
    lbu x8, 0(x0)               # Load unsigned byte from address 0 into x8, should get 0x78
    bne x8, x4, _test_lbu_fail  # If loaded value does not match original, flag it in x13
    lbu x8, 1(x0)               # Load unsigned byte from address 1 into x8, should get 0x56
    bne x8, x5, _test_lbu_fail  # If loaded value does not match original, flag it in x13
    lbu x8, 2(x0)               # Load unsigned byte from address 2 into x8, should get 0x34
    bne x8, x6, _test_lbu_fail  # If loaded value does not match original, flag it in x13
    lbu x8, 3(x0)               # Load unsigned byte from address 3 into x8, should get 0x12
    bne x8, x7, _test_lbu_fail  # If loaded value does not match original, flag it in x13
    # Test signed byte loads
    addi x1, x0, -1        # x1 <- 0xFFFFFFFF (signed -1)
    sw x1, 4(x0)                # Store 0xFFFFFFFF at address 4
    lb x8, 4(x0)                # Load signed byte from address 4 into x8, should get 0xFFFFFFFF (signed -1)
    bne x8, x1, _test_lb_fail   # If loaded value does not match original, flag it in x10
    j _test_half                  # Jump to the half-word load tests

_test_lb_fail:
    addi x10, x0, 0x40          # Flag LB test failure in x10
    j _test_half                  # Jump to the half-word load tests

_test_lbu_fail:
    addi x13, x0, 0x40          # Flag LBU test failure in x13
    j _test_half                  # Jump to the half-word load tests

_test_half:
    # Similar to the byte load tests, we need to ensure endianness is correct for the half-word load tests
    lui x3, 0x12345             # x3 <- 0x12345000
    addi x3, x3, 0x678          # x3 <- 0x12345678
    lui x4, 0x00005             # x4 <- 0x00005000
    addi x4, x4, 0x678          # x4 <- 0x5678
    lui x5, 0x00001             # x5 <- 0x00001000
    addi x5, x5, 0x234          # x5 <- 0x1234
    sh x4, 0(x0)                # Store the least significant half-word of x4 (0x5678) at address 0
    sh x5, 2(x0)                # Store the least significant half-word of x5 (0x1234) at address 2
    lw x2, 0(x0)                # Load from address 0 into x2, should get 0x12345678
    bne x2, x3, _test_lh_fail   # If loaded value does not match original, flag it in x11
    lhu x8, 0(x0)               # Load unsigned half-word from address 0 into x8, should get 0x5678
    bne x8, x4, _test_lhu_fail  # If loaded value does not match original, flag it in x14
    lhu x8, 2(x0)               # Load unsigned half-word from address 2 into x8, should get 0x1234
    bne x8, x5, _test_lhu_fail  # If loaded value does not match original, flag it in x14
    # Test signed half-word loads
    addi x1, x0, -1        # x1 <- 0xFFFFFFFF (signed -1)
    sw x1, 4(x0)                # Store 0xFFFFFFFF at address 4
    lh x8, 4(x0)                # Load signed half-word from address 4 into x8, should get 0xFFFFFFFF (signed -1)
    bne x8, x1, _test_lh_fail   # If loaded value does not match original, flag it in x11
    j _end

_test_lh_fail:
    addi x11, x0, 0x40          # Flag LH test failure in x11
    j _end

_test_lhu_fail:
    addi x14, x0, 0x40          # Flag LHU test failure in x14
    j _end  

_end:
    nop
