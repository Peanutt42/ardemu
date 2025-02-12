store 0x20, 0x24  ; addr 0x24 = DDRB, if 0x20 is set, used as OUTPUT

loop:
    store 0x20, 0x25  ; turn LED on
    store 0, 0x25  ; turn LED off
    jmp loop