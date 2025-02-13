# ardemu

`fib.asm`:
```asm
; a is n
; b is n1
; c is n2
; d is n3
; z is result

mw b, 0                ; n1 = 0
mw c, 1                ; n2 = 1

add a, 1

loop:                  ; while n > 0
    mw z, b            ; result = n1
    mw d, b            ; n3 = n1
    add d, c           ; n3 += n2
    mw b, c            ; n1 = n2
    mw c, d            ; n2 = n3
    sub a, 1           ; n--
    lda loop
    jnz a              ; if n > 0, continue

; result is in z
```

`main.rs`:
```rust
fn main() {
	let mut cpu = Cpu::new(include_asm!("src/fib.asm"));

	let n = 10;

	cpu.write_register(A, n);

	while cpu.step().unwrap() {}

	let result = cpu.read_register(Z);

	assert_eq!(result, 55);
}
```