    ldi r0, 0x0     ; r0 = LOW
    ldi r1, 0x20    ; r1 = HIGH
    store r1, 0x24  ; addr 0x24 = DDRB, if 0x20 is set, used as OUTPUT

loop:
    store r1, 0x25  ; turn LED on
    store r0, 0x25  ; turn LED off
    jmp loop