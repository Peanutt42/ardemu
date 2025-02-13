mw a 0x5
mw b 0x10

loop:
    add a, b
    lda loop
    jnz a