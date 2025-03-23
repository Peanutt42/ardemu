use std::ops::RangeInclusive;

use ardemu_assemble_macro::include_asm;
use ardemu_core::{
	Cpu, CpuStatus, LowerEvenRegister, Program,
	Register::{self, R16, R17, R20},
};

fn testing_fib(n: usize) -> usize {
	if n == 0 || n == 1 {
		return n;
	}

	testing_fib(n - 1) + testing_fib(n - 2)
}

fn test_fib_program(
	name: &'static str,
	program: Program,
	test_range: RangeInclusive<u8>,
	n_register: Register,
	read_result_callback: impl Fn(&mut Cpu) -> usize,
) {
	for n in test_range {
		let mut cpu = Cpu::new(program.clone());
		cpu.write_register(n_register, n);

		while matches!(
			cpu.step()
				.unwrap_or_else(|e| panic!("{name} failed to step simulation: {e}")),
			CpuStatus::Normal
		) {}

		let result = read_result_callback(&mut cpu);
		let expected = testing_fib(n as usize);
		assert_eq!(
			result, expected,
			"{name} at n = {n} should be {expected}, but got {result}"
		);
	}
}

#[test]
fn test_different_fib_programs() {
	test_fib_program(
		"fib.asm",
		include_asm!("sample_programs/fib.asm"),
		0..=10,
		R16,
		|cpu| cpu.read_register(R20) as usize,
	);
	test_fib_program(
		"fib16.asm",
		include_asm!("sample_programs/fib16.asm"),
		0..=24,
		R16,
		|cpu| cpu.read_register_pair16(LowerEvenRegister::R26) as usize,
	);
	test_fib_program(
		"recursive_fib.asm",
		include_asm!("sample_programs/recursive_fib.asm"),
		0..=10,
		R16,
		|cpu| cpu.read_register(R17) as usize,
	);
}
