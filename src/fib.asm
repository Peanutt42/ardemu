; r16 is n
; r17 is n1
; r18 is n2
; r19 is n3
; r20 is result

ldi r17, 0                  ; n1 = 0
ldi r18, 1                  ; n2 = 1

inc r16                     ; n++

loop:                       ; while n > 0
    mov r20, r17            ; result = n1
    mov r19, r17            ; n3 = n1
    add r19, r18            ; n3 += n2
    mov r17, r18            ; n1 = n2
    mov r18, r19            ; n2 = n3
    dec r16                 ; n--
    brne loop               ; if n > 0, continue

; result is in r20