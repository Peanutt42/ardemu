# ardemu

![ardemu_gui](ardemu_gui/Screenshot.png)

`fib.asm`:
```asm
; a is n
; b is n1
; c is n2
; d is n3
; z is result

mw b, 0                ; r1 = 0
mw c, 1                ; r2 = 1

add a, 1

loop:                  ; while n > 0
    mw z, b            ; result = n1
    mw d, b            ; n3 = n1
    add d, c           ; n3 += n2
    mw b, c            ; n1 = n2
    mw c, d            ; n2 = n3
    dec a              ; n--
    lda loop
    jnz a              ; if n > 0, continue

; result is in z
```

`main.rs`:
```rust
use ardemu_asm_parse_macro::include_asm;
use ardemu_core::{
	Cpu, CpuStatus,
	Register::{A, Z},
};

#[test]
fn fib() {
	let mut cpu = Cpu::new(include_asm!("src/fib.asm"));

	let n = 10;

	cpu.write_register(A, n);

	while matches!(cpu.step(), Ok(CpuStatus::Normal)) {}

	let result = cpu.read_register(Z);

	assert_eq!(result, 55);
}
```