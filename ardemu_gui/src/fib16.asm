; 16-bit Fibonacci:
; r16: n
; r21:r20: next
; r23:r22: current
; r25:r24: prev
; r27:r26: result

ldi r16, 24            ; change n here!

inc r16                ; n++

; prev = 0
ldi r24, 0x00
ldi r25, 0x00
; current = 1
ldi r22, 0x01
ldi r23, 0x00

loop:
    movw r26, r24      ; result = prev

    ; 16-bit add: next = prev + current
    mov r20, r24
    add r20, r22
    mov r21, r25
    adc r21, r23

    movw r24, r22     ; prev = current

    movw r22, r20     ; current = next

    dec r16           ; n--
    brne loop         ; while n != 0