use ardemu_asm_parse_macro::include_asm;
use ardemu_core::{
	Cpu,
	Register::{A, Z},
};

#[test]
fn fib() {
	let program = include_asm!("src/fib.asm");

	let mut cpu = Cpu::default();

	let n = 10;

	cpu.registers[A as usize] = n;

	while let Some(instr) = cpu.get_current_instruction(&program) {
		cpu.execute(instr).expect("failed to execute instruction");
	}

	assert_eq!(cpu.registers[Z as usize], 55);
}
