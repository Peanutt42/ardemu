use ardemu_asm_parse_macro::include_asm;
use ardemu_core::Cpu;

fn main() {
	let program = include_asm!("src/blink.asm");

	let mut cpu = Cpu::default();

	while let Some(instr) = cpu.get_current_instruction(&program) {
		if let Err(e) = cpu.execute(instr) {
			eprintln!("failed to execute instruction: {e}");
			return;
		}
		println!(
			"{}: {instr}\n\t-> r0={:#04x}, r1={:#04x}, LED={}",
			cpu.program_counter,
			cpu.registers[0],
			cpu.registers[1],
			if cpu.is_builtin_led_on() {
				"HIGH"
			} else {
				"LOW"
			}
		);
	}

	println!("Program finished");
}
