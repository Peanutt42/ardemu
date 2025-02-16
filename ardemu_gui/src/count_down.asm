mw a, 255
mw b, 255
mw c, 255
mw d, 255

loop_a:
	sub a, 1
	lda loop_a
	jnz a
	lda loop_b
	jnz 1

loop_b:
	sub b, 1
	lda loop_a
	jnz b
	lda loop_c
	jnz 1

loop_c:
	sub c, 1
	lda loop_b
	jnz c
	lda loop_d
	jnz 1

loop_d:
	sub d, 1
	lda loop_c
	jnz 1