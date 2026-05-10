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

    # Test JAL instruction
    jal x1, _test_jal_target     # Jump to _test_jal_target
    addi x10, x0, 0x01          # Should not be here, flag it in x10
    j _test_jal_end             # Jump to the end of the test
    addi x10, x0, 0x04          # Should not be here, flag it in x10
    j _test_jalr_end
    nop
_test_jal_target:
    # After the jump, x1 should contain the return address (address of the instruction after the JAL)
    # We can check this by comparing x1 to the expected return address
    addi x2, x0, 0x1c        # Expected address 
    bne  x1, x2, _test_jal_fail   # If x1 != expected address, jump to failure handler
    # If we are here, the JAL test passed, we can try to jump back to the caller using JALR
    # But we need to skip the first instruction, which is the one that flags failure in x10,
    # so we add 4 to x1 to point to the instruction after the one that flags failure.
    jalr x0, x1, 4              # Jump back to the address in x1 (the return address+4)
    addi x11, x0, 0x02          # Should not be here, flag it in x11
    j _test_jal_end
    nop
_test_jal_fail:
    addi x10, x0, 0x80          # Flag failure in x10
    j _test_jal_end
_test_jal_end:
    # If we reach here, the JAL test passed, x10 should be 0
    # We now want to test the JALR instruction, we will jump to the same target as before, but this time using JALR instead of JAL
    jalr x0, x0, 0x28              # Jump to the address in x1 (the target address)
    addi x11, x0, 0x01          # Should not be here, flag it in x11
    j _test_jalr_end
    nop
_test_jalr_end:
    # If we reach here, the JALR test passed, x11 should be 0
    # We can now test the conditional branch instructions, we will use the same target
    nop
    nop