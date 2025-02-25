use ardemu_assemble_macro::include_asm;
use ardemu_core::{
	Cpu, CpuStatus,
	Register::{R16, R17, R18, R19, R20},
};

fn main() {
	let mut cpu = Cpu::new(include_asm!("src/fib.asm"));

	let n = 10;

	cpu.write_register(R16, n);

	loop {
		let current_instruction = cpu.get_current_instruction();

		match cpu.step() {
			Ok(cpu_status) => match cpu_status {
				CpuStatus::Normal => {
					println!(
						"{}: {}\n\t-> r16={:#04x}, r17={:#04x}, r18={:#04x}, r19={:#04x}, r20={:#04x}",
						cpu.get_program_counter(),
						current_instruction.unwrap(),
						cpu.read_register(R16),
						cpu.read_register(R17),
						cpu.read_register(R18),
						cpu.read_register(R19),
						cpu.read_register(R20),
					);
				}
				CpuStatus::BreakpointHit => {
					println!("breakpoint hit");
					break;
				}
				CpuStatus::BreakHit => {
					println!("break hit");
					break;
				}
				CpuStatus::ProgramFinished => {
					break;
				}
			},
			Err(e) => {
				eprintln!("failed to execute instruction: {e}");
				return;
			}
		}
	}

	let output = cpu.read_register(R20);
	assert_eq!(output, 55);
	println!("Fib({n}) = {output}");
}
