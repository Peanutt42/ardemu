; decrements a 32 bit value using four 8-bit registers

ldi r16, 255
ldi r17, 255
ldi r18, 255
ldi r19, 255

loop_r16:
	subi r16, 1
	brne loop_r16
	jmp loop_r17

loop_r17:
	subi r17, 1
	brne loop_r16
	jmp loop_r18

loop_r18:
	subi r18, 1
	brne loop_r17
	jmp loop_r19

loop_r19:
	subi r19, 1
	jmp loop_r18