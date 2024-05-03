    .section .text.entry
    .globl _start
_start:
# 把默认0x8…和9的窗给关了，全开成0的窗，这样就相当于0的这个部分是地址恒等映射，直接继承原来的代码
    pcaddi      $t0,    0x0
    srli.d      $t0,    $t0,    0x30
    slli.d      $t0,    $t0,    0x30
    addi.d      $t0,    $t0,    0x11
    csrwr       $t0,    0x181   # Make sure the window remains the same after the switch.
    # 前5行是把当前PC所在段给保留下来,存到DMW1
    # 然后改DMW0
    # 使用sub生成0,因为有些版本的虚拟机上面zero会被赋值,避免使用zero
    sub.d       $t0,    $t0,    $t0
    addi.d      $t0,    $t0,    0x11
    csrwr       $t0,    0x180
    pcaddi      $t0,    0x0
    slli.d      $t0,    $t0,    0x10
    srli.d      $t0,    $t0,    0x10
    jirl        $t0,    $t0,    0x10    # 跳0段的下一条指令
    # The barrier
    sub.d       $t0,    $t0,    $t0
    csrwr       $t0,    0x181
    sub.d       $t0,    $t0,    $t0
    la.global $sp, boot_stack_top
    bl          rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
