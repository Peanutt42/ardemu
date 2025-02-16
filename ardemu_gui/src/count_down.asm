mw a, 255
mw b, 255
mw c, 255
mw d, 255

loop_a:
	dec a
	lda loop_a
	jnz a
	jmp loop_b

loop_b:
	dec b
	lda loop_a
	jnz b
	jmp loop_c

loop_c:
	dec c
	lda loop_b
	jnz c
	jmp loop_d

loop_d:
	dec d
	jmp loop_c