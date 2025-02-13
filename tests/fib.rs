use ardemu_asm_parse_macro::include_asm;
use ardemu_core::{
	Cpu,
	Register::{A, Z},
};

#[test]
fn fib() {
	let mut cpu = Cpu::new(include_asm!("src/fib.asm"));

	let n = 10;

	cpu.registers[A as usize] = n;

	while cpu.step().unwrap() {}

	assert_eq!(cpu.registers[Z as usize], 55);
}
