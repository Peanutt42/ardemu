use ardemu_asm_parse_macro::include_asm;
use ardemu_core::{
	Cpu,
	Register::{A, B, C, D, Z},
};

fn main() {
	let mut cpu = Cpu::new(include_asm!("src/fib.asm"));

	let n = 10;

	cpu.registers[0] = n;

	loop {
		match cpu.step() {
			Ok(false) => break,
			Ok(true) => {
				println!(
					"{}: {}\n\t-> a={:#04x}, b={:#04x}, c={:#04x}, d={:#04x}, z={:#04x}",
					cpu.program_counter,
					cpu.get_current_instruction().unwrap(),
					cpu.registers[A as usize],
					cpu.registers[B as usize],
					cpu.registers[C as usize],
					cpu.registers[D as usize],
					cpu.registers[Z as usize]
				);
			}
			Err(e) => {
				eprintln!("failed to execute instruction: {e}");
				return;
			}
		}
	}

	let output = cpu.registers[Z as usize];
	assert_eq!(output, 55);
	println!("Fib({n}) = {output}");
}
