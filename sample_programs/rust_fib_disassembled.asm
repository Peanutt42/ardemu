
./template-bin.elf:     file format elf32-avr


Disassembly of section .text:

00000000 <__vectors>:
   0:	0c 94 34 00 	jmp	0x68	; 0x68 <__ctors_end>
   4:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
   8:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
   c:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  10:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  14:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  18:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  1c:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  20:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  24:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  28:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  2c:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  30:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  34:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  38:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  3c:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  40:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  44:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  48:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  4c:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  50:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  54:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  58:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  5c:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  60:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>
  64:	0c 94 3e 00 	jmp	0x7c	; 0x7c <__bad_interrupt>

00000068 <__ctors_end>:
  68:	11 24       	eor	r1, r1
  6a:	1f be       	out	0x3f, r1	; 63
  6c:	cf ef       	ldi	r28, 0xFF	; 255
  6e:	d8 e0       	ldi	r29, 0x08	; 8
  70:	de bf       	out	0x3e, r29	; 62
  72:	cd bf       	out	0x3d, r28	; 61
  74:	0e 94 4d 00 	call	0x9a	; 0x9a <main>
  78:	0c 94 5c 00 	jmp	0xb8	; 0xb8 <_exit>

0000007c <__bad_interrupt>:
  7c:	0c 94 00 00 	jmp	0	; 0x0 <__vectors>

00000080 <_ZN12template_bin3fib17h381bcd22270dae12E>:
  80:	83 95       	inc	r24
  82:	21 e0       	ldi	r18, 0x01	; 1
  84:	41 2d       	mov	r20, r1
  86:	31 e0       	ldi	r19, 0x01	; 1
  88:	94 2f       	mov	r25, r20
  8a:	39 0f       	add	r19, r25
  8c:	8a 95       	dec	r24
  8e:	80 30       	cpi	r24, 0x00	; 0
  90:	42 2f       	mov	r20, r18
  92:	23 2f       	mov	r18, r19
  94:	c9 f7       	brne	.-14     	; 0x88 <_ZN12template_bin3fib17h381bcd22270dae12E+0x8>
  96:	89 2f       	mov	r24, r25
  98:	08 95       	ret

0000009a <main>:
  9a:	0f 93       	push	r16
  9c:	1f 93       	push	r17
  9e:	11 2d       	mov	r17, r1
  a0:	01 2d       	mov	r16, r1
  a2:	80 2f       	mov	r24, r16
  a4:	0e 94 40 00 	call	0x80	; 0x80 <_ZN12template_bin3fib17h381bcd22270dae12E>
  a8:	18 0f       	add	r17, r24
  aa:	03 95       	inc	r16
  ac:	0f 3f       	cpi	r16, 0xFF	; 255
  ae:	c9 f7       	brne	.-14     	; 0xa2 <main+0x8>
  b0:	81 2f       	mov	r24, r17
  b2:	1f 91       	pop	r17
  b4:	0f 91       	pop	r16
  b6:	08 95       	ret

000000b8 <_exit>:
  b8:	f8 94       	cli

000000ba <__stop_program>:
  ba:	ff cf       	rjmp	.-2      	; 0xba <__stop_program>
