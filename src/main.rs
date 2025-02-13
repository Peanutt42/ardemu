use ardemu_asm_parse_macro::include_asm;
use ardemu_core::{
	Cpu,
	Register::{A, B, C, D, Z},
};

fn main() {
	let mut cpu = Cpu::new(include_asm!("src/fib.asm"));

	let n = 10;

	cpu.write_register(A, n);

	loop {
		let current_instruction = cpu.get_current_instruction();

		match cpu.step() {
			Ok(false) => break,
			Ok(true) => {
				println!(
					"{}: {}\n\t-> a={:#04x}, b={:#04x}, c={:#04x}, d={:#04x}, z={:#04x}",
					cpu.get_program_counter(),
					current_instruction.unwrap(),
					cpu.read_register(A),
					cpu.read_register(B),
					cpu.read_register(C),
					cpu.read_register(D),
					cpu.read_register(Z)
				);
			}
			Err(e) => {
				eprintln!("failed to execute instruction: {e}");
				return;
			}
		}
	}

	let output = cpu.read_register(Z);
	assert_eq!(output, 55);
	println!("Fib({n}) = {output}");
}
