use ardemu_asm_parse_macro::include_asm;
use ardemu_core::{
	Cpu,
	Register::{A, B, C, D, Z},
};

fn main() {
	let program = include_asm!("src/fib.asm");

	let mut cpu = Cpu::default();

	let n = 10;

	cpu.registers[0] = n;

	while let Some(instr) = cpu.get_current_instruction(&program) {
		if let Err(e) = cpu.execute(instr) {
			eprintln!("failed to execute instruction: {e}");
			return;
		}
		println!(
			"{}: {instr}\n\t-> a={:#04x}, b={:#04x}, c={:#04x}, d={:#04x}, z={:#04x}",
			cpu.program_counter,
			cpu.registers[A as usize],
			cpu.registers[B as usize],
			cpu.registers[C as usize],
			cpu.registers[D as usize],
			cpu.registers[Z as usize]
		);
	}

	let output = cpu.registers[Z as usize];
	assert_eq!(output, 55);
	println!("Fib({n}) = {output}");
}
