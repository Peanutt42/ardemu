; decrements a 32 bit value using four 8-bit registers

ldi r16, 255
ldi r17, 255
ldi r18, 255
ldi r19, 255

loop_r16:
	dec r16
	brne loop_r16
	jmp loop_r17

loop_r17:
	dec r17
	brne loop_r16
	jmp loop_r18

loop_r18:
	dec r18
	brne loop_r17
	jmp loop_r19

loop_r19:
	dec r19
	jmp loop_r18