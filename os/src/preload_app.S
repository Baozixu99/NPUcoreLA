    .section .data
    .global sinitproc
    .global einitproc
    .align 12
sinitproc:
    .incbin "./initproc"
einitproc:
    .align 12

    .section .data
    .global sbash
    .global ebash
    .align 12
sbash:
    .incbin "./bash"
ebash:
    .align 12
