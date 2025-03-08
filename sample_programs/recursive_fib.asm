; n = r16
; result = r17

call Fib
jmp exit

Fib:
    cpi r16, 0             ; Check if n == 0
    brne not_zero
    ldi r17, 0             ; Return 0
    ret
not_zero:
    cpi r16, 1             ; Check if n == 1
    brne not_one
    ldi r17, 1             ; Return 1
    ret
not_one:
    push r16               ; Save original n
    dec r16                ; n-1
    call Fib              ; Compute Fib(n-1)
    push r17               ; Save Fib(n-1)
    pop r0                 ; Move Fib(n-1) to r0
    pop r16                ; Restore original n
    push r0                ; Save Fib(n-1) on stack
    subi r16, 2            ; n-2
    call Fib              ; Compute Fib(n-2)
    pop r0                 ; Retrieve Fib(n-1)
    add r17, r0            ; Fib(n-1) + Fib(n-2)
    ret

exit: