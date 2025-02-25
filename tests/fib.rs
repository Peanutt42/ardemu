use ardemu_assemble_macro::include_asm;
use ardemu_core::{
	Cpu, CpuStatus,
	Register::{R16, R20, R26},
	RegisterPair16,
};

fn testing_fib(n: usize) -> usize {
	if n == 0 || n == 1 {
		return n;
	}

	testing_fib(n - 1) + testing_fib(n - 2)
}

#[test]
fn fib() {
	for n in 0..=10 {
		let mut cpu = Cpu::new(include_asm!("src/fib.asm"));
		cpu.write_register(R16, n as u8);

		while matches!(cpu.step().unwrap(), CpuStatus::Normal) {}

		let result = cpu.read_register(R20) as usize;
		assert_eq!(result, testing_fib(n));
	}
}

#[test]
fn fib16() {
	for n in 0..=24 {
		let mut cpu = Cpu::new(include_asm!("src/fib16.asm"));
		cpu.write_register(R16, n as u8);

		while matches!(cpu.step().unwrap(), CpuStatus::Normal) {}

		let result = cpu.read_register_pair16(RegisterPair16::new(R26).unwrap()) as usize;

		assert_eq!(result, testing_fib(n));
	}
}
