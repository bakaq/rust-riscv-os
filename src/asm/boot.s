# vim: ft=asm

.option norvc
.section .data

.section .text.init
.global _start
_start:
    # Park all harts except the one with hartid = 0
    csrr t0, mhartid
    bnez t0, 3f

    # Zero the satp (TODO: understand why)
    csrw satp, zero

.option push
.option norelax
    la gp, _global_pointer
.option pop

    # Clear the bss
    la a0, _bss_start
    la a1, _bss_end
    bgeu a0, a1, 2f
1:
    sd zero, (a0)
    addi a0, a0, 8
    bltu a0, a1, 1b

2:
    # Setup the stack
    la sp, _stack_end

    # Set needed CSRs
    li t0, (0b11 << 11) | (1 << 7) | (1 << 3)
    csrw mstatus, t0

    la t1, kmain
    csrw mepc, t1

    la t2, asm_trap_vector
    csrw mtvec, t2

    # No timer interrupts for now because they are evil
    li t3, (1 << 3) | (1 << 11)
    csrw mie, t3

    la ra, 3f
    mret

3:
    wfi
    j 3b
