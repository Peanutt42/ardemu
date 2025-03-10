; Disassembled instructions from rust_fib.elf
./rust_fib.elf:     file format elf32-avr


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
  74:	0e 94 53 00 	call	0xa6	; 0xa6 <main>
  78:	0c 94 5b 00 	jmp	0xb6	; 0xb6 <_exit>

0000007c <__bad_interrupt>:
  7c:	0c 94 00 00 	jmp	0	; 0x0 <__vectors>

00000080 <_ZN8rust_fib3fib17h52828e8768a34918E>:
  80:	0f 93       	push	r16
  82:	1f 93       	push	r17
  84:	18 2f       	mov	r17, r24
  86:	01 2d       	mov	r16, r1
  88:	12 30       	cpi	r17, 0x02	; 2
  8a:	40 f0       	brcs	.+16     	; 0x9c <_ZN8rust_fib3fib17h52828e8768a34918E+0x1c>
  8c:	81 2f       	mov	r24, r17
  8e:	8a 95       	dec	r24
  90:	0e 94 40 00 	call	0x80	; 0x80 <_ZN8rust_fib3fib17h52828e8768a34918E>
  94:	08 0f       	add	r16, r24
  96:	12 50       	subi	r17, 0x02	; 2
  98:	12 30       	cpi	r17, 0x02	; 2
  9a:	c0 f7       	brcc	.-16     	; 0x8c <_ZN8rust_fib3fib17h52828e8768a34918E+0xc>
  9c:	01 0f       	add	r16, r17
  9e:	80 2f       	mov	r24, r16
  a0:	1f 91       	pop	r17
  a2:	0f 91       	pop	r16
  a4:	08 95       	ret

000000a6 <main>:
  a6:	8a e0       	ldi	r24, 0x0A	; 10
  a8:	0e 94 40 00 	call	0x80	; 0x80 <_ZN8rust_fib3fib17h52828e8768a34918E>
  ac:	68 2f       	mov	r22, r24
  ae:	77 27       	eor	r23, r23
  b0:	80 e0       	ldi	r24, 0x00	; 0
  b2:	90 e0       	ldi	r25, 0x00	; 0
  b4:	08 95       	ret

000000b6 <_exit>:
  b6:	f8 94       	cli

000000b8 <__stop_program>:
  b8:	ff cf       	rjmp	.-2      	; 0xb8 <__stop_program>
