OUTPUT_ARCH(riscv)
ENTRY(_start)

/* Qemu, fu740 */
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* Load the kernel at this address: "." means the current address */
    . = BASE_ADDRESS;
    start = .;

    .text : {
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        . = ALIGN(4K);
        etext = .;
    }

    .rodata : {
        srodata = .;
        *(.rodata .rodata.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data : {
        sdata = .;
        *(.data .data.*)
        edata = .;
    }

    .stack : {
        *(.bss.stack)
        . = ALIGN(4K);
    }

    /* .sbss 会存放为0的static变量，系统启动时需要清0 */
    .bss : {
        sbss = .;
        *(.bss .bss.* .sbss .sbss.*)
        ebss = .;
    }

    . = ALIGN(4K);
    PROVIDE(end = .);
}
