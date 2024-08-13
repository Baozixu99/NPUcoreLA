	.section .text.entry
	.globl _start
_start:
	#OpenSBI将DTB地址保存在a1寄存器

	#关中断
	csrw sie, zero

	#关闭mmu
        csrw satp, zero

	#BSS节清零
	la t0, sbss
	la t1, ebss
	bgeu t0, t1, 2f

1:
	# sd: store double word (64 bits)
	sd zero, (t0)
	addi t0, t0, 8
	bltu t0, t1, 1b

2:
	la sp, bootstacktop
	call rust_main

4:
    wfi
	j 4b

# stack栈空间
	.section .bss.stack
	.align 12
	.global bootstack
bootstack:
	.space 1024 * 64
	.global bootstacktop
bootstacktop:
