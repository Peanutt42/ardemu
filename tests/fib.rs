use ardemu_asm_parse_macro::include_asm;
use ardemu_core::{
	Cpu, CpuStatus,
	Register::{A, Z},
};

#[test]
fn fib() {
	let mut cpu = Cpu::new(include_asm!("src/fib.asm"));

	let n = 10;

	cpu.write_register(A, n);

	while matches!(cpu.step(), Ok(CpuStatus::Normal)) {}

	let result = cpu.read_register(Z);

	assert_eq!(result, 55);
}
