BASE_ADDRESS = 0x0000000090000000;
OUTPUT_ARCH( "loongarch" )
ENTRY(_start)
SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;
    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline);
        . = ALIGN(4K);
        ssignaltrampoline = .;
        KEEP(*(.text.signaltrampoline));
        . = ALIGN(4K);
        srfill = .;
        KEEP(*(.text.__rfill));
        . = ALIGN(4K);
        strap_handler = .;
        KEEP(*(.text.trap_handler));
        . = ALIGN(4K);
        skern_trap = .;
        KEEP(*(.text.kern_trap));
        . = ALIGN(4K);
        *(.text .text.*)
    }
    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    erodata = .;
    seh_frame_hdr = .;
    .eh_frame_hdr : {
        KEEP(*(.eh_frame_hdr .eh_frame_hdr.*))
    }
    . = ALIGN(4K);
    eeh_frame_hdr = .;
    seh_frame = .;
    .eh_frame : {
        KEEP(*(.eh_frame .eh_frame.*))
    }
    . = ALIGN(4K);
    eeh_frame =.;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    edata = .;
    sbss_with_stack = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    ebss = .;
    ekernel = .;
}
