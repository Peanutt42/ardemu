use ardemu_core::{
	assemble, Cpu, CpuStatus,
	Register::{self, R26},
	RegisterPair16,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate(source_code: &str) -> String {
	let (instructions_str, mut cpu) = match assemble(source_code) {
		Ok(program) => (
			program
				.iter()
				.map(|instruction| format!("{instruction}"))
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

	let result = cpu.read_register_pair16(RegisterPair16::new(R26).unwrap());
	format!(
		"{instructions_str}\n\n{cpu_register_str}\n\nEvaluated to {}",
		result
	)
}
