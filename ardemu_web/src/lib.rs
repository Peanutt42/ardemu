use ardemu_core::{assemble, Cpu, CpuStatus, LowerEvenRegister, Register};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate(source_code: &str) -> String {
	let (instructions_str, mut cpu) = match assemble(source_code) {
		Ok(program) => (
			program
				.iter()
				.map(|(program_address, instruction)| format!("{program_address}: {instruction}"))
				.collect::<Vec<String>>()
				.join("\n"),
			Cpu::new(program),
		),
		Err(e) => return format!("{e:?}"),
	};

	loop {
		match cpu.step() {
			Ok(status) => match status {
				CpuStatus::Normal => {}
				CpuStatus::BreakpointHit => return "Breakpoint hit!".to_string(),
				CpuStatus::BreakHit => return "Break hit!".to_string(),
				CpuStatus::ProgramFinished => break,
			},
			Err(e) => return format!("{e:?}"),
		}
	}

	let cpu_register_str = Register::ALL
		.iter()
		.map(|reg| format!("{reg} = {}", cpu.read_register(*reg)))
		.collect::<Vec<String>>()
		.join(", ");

	let result = cpu.read_register_pair16(LowerEvenRegister::R26);
	format!(
		"{instructions_str}\n\n{cpu_register_str}\n\nEvaluated to {}",
		result
	)
}
