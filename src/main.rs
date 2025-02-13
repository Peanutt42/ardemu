use ardemu_asm_parse_macro::include_asm;
use ardemu_core::Cpu;

fn main() {
	let program = include_asm!("src/main.asm");

	let mut cpu = Cpu::default();

	while let Some(instr) = cpu.get_current_instruction(&program) {
		if let Err(e) = cpu.execute(instr) {
			eprintln!("failed to execute instruction: {e}");
			return;
		}
		println!(
			"{}: {instr}\n\t-> a={:#04x}, b={:#04x}",
			cpu.program_counter, cpu.registers[0], cpu.registers[1]
		);
	}

	println!("Program finished");
}
