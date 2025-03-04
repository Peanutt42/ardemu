use ardemu_core::{
	Cpu, CpuError, CpuStatus, Instruction, LowerEvenRegister,
	Register::{self, R0, R1, R16, R17},
	UpperRegister, WordRegister,
};

#[test]
fn test_execute_push_pop() {
	let mut cpu = Cpu::default();
	cpu.write_register(R0, 42);
	cpu.write_register(R1, 15);
	cpu.execute(Instruction::Push { register: R0 }).unwrap();
	cpu.execute(Instruction::Push { register: R1 }).unwrap();
	cpu.write_register(R0, 0);
	cpu.write_register(R1, 0);
	cpu.execute(Instruction::Pop { register: R1 }).unwrap();
	cpu.execute(Instruction::Pop { register: R0 }).unwrap();
	assert_eq!(cpu.read_register(R0), 42);
	assert_eq!(cpu.read_register(R1), 15);
}

#[test]
fn test_stackoverflow() {
	let mut cpu = Cpu::default();
	cpu.write_register(R0, 0);
	loop {
		match cpu.execute(Instruction::Push { register: R0 }) {
			Ok(_) => (),
			Err(CpuError::StackOverflow) => break,
			Err(err) => panic!("Unexpected error before stack overflow: {err}"),
		}
	}
}

#[test]
fn test_stackunderflow() {
	let mut cpu = Cpu::default();
	assert_eq!(
		cpu.execute(Instruction::Pop { register: R0 }),
		Err(CpuError::StackUnderflow)
	);
}

#[test]
fn test_execute_jmp() {
	let mut cpu = Cpu::default();
	cpu.execute(Instruction::Jmp { address: 42 }).unwrap();
	assert_eq!(cpu.get_program_counter(), 42);
}

#[test]
fn test_register_pair() {
	let n: u16 = 0xFEFF;
	let mut cpu = Cpu::default();
	cpu.write_register_pair16(LowerEvenRegister::R16, n);
	assert_eq!(cpu.read_register(R16), 0xFF);
	assert_eq!(cpu.read_register(R17), 0xFE);
	assert_eq!(cpu.read_register_pair16(LowerEvenRegister::R16), n);
}

#[test]
fn test_breakpoint() {
	let program = vec![
		Instruction::Ldi {
			register: UpperRegister::R16,
			value: 42.into(),
		},
		Instruction::Ldi {
			register: UpperRegister::R16,
			value: 42.into(),
		},
		// breakpoint will be set here
		Instruction::Ldi {
			register: UpperRegister::R16,
			value: 42.into(),
		},
		Instruction::Ldi {
			register: UpperRegister::R16,
			value: 42.into(),
		},
	];
	let mut cpu = Cpu::new(program);
	cpu.add_breakpoint(2);
	assert_eq!(cpu.step(), Ok(CpuStatus::Normal));
	assert_eq!(cpu.step(), Ok(CpuStatus::Normal));
	assert_eq!(cpu.step(), Ok(CpuStatus::BreakpointHit));
	// should not continue execution after breakpoint!
	assert_eq!(cpu.step(), Ok(CpuStatus::BreakpointHit));
}

#[test]
fn test_arithmetic_instructions() {
	fn test_single(
		instruction: Instruction,
		input_register: Register,
		output_register: Register,
		expected: impl Fn(u8) -> u8,
	) {
		let mut cpu = Cpu::default();

		for value in u8::MIN..u8::MAX {
			cpu.write_register(input_register, value);
			assert_eq!(cpu.execute(instruction).unwrap(), CpuStatus::Normal);
			let output = cpu.read_register(output_register);
			let expected_output = expected(value);
			assert_eq!(
				output, expected_output,
				"{instruction} with value: {value} resulted in {output} instead of {expected_output}"
			);
		}
	}

	fn test_a_b(
		instruction: Instruction,
		input_a_register: Register,
		input_b_register: Register,
		output_register: Register,
		expected: impl Fn(u8, u8) -> u8,
	) {
		let mut cpu = Cpu::default();

		for a in u8::MIN..u8::MAX {
			for b in u8::MIN..u8::MAX {
				cpu.write_register(input_a_register, a);
				cpu.write_register(input_b_register, b);
				assert_eq!(cpu.execute(instruction).unwrap(), CpuStatus::Normal);
				let output = cpu.read_register(output_register);
				let expected_output = expected(a, b);
				assert_eq!(
					output,
					expected_output,
					"{instruction} with a: {a} and b: {b} resulted in a result of {output} instead of {expected_output}"
				);
			}
		}
	}

	fn test_word(
		instruction: Instruction,
		input_register: WordRegister,
		output_register: WordRegister,
		expected: impl Fn(u16) -> u16,
	) {
		let mut cpu = Cpu::default();

		for value in u16::MIN..u16::MAX {
			cpu.write_register_pair16(input_register, value);
			assert_eq!(cpu.execute(instruction).unwrap(), CpuStatus::Normal);
			let output = cpu.read_register_pair16(output_register);
			let expected_output = expected(value);
			assert_eq!(
				output,
				expected_output,
				"{instruction} with value: {value} resulted in a result of {output} instead of {expected_output}"
			);
		}
	}

	test_single(
		Instruction::Ori {
			register: UpperRegister::R16,
			value: 42.into(),
		},
		R16,
		R16,
		|value| value | 42,
	);
	test_single(Instruction::Com { register: R0 }, R0, R0, |value| {
		u8::MAX.wrapping_sub(value)
	});
	test_single(Instruction::Neg { register: R0 }, R0, R0, |value| {
		0_u8.wrapping_sub(value)
	});
	test_single(Instruction::Lsr { register: R0 }, R0, R0, |value| {
		value.wrapping_shr(1)
	});
	test_single(
		Instruction::Subi {
			register: UpperRegister::R16,
			value: 42.into(),
		},
		R16,
		R16,
		|value| value.wrapping_sub(42),
	);
	test_single(
		Instruction::Andi {
			register: UpperRegister::R16,
			value: 42.into(),
		},
		R16,
		R16,
		|value| value & 42,
	);
	test_single(Instruction::Inc { register: R0 }, R0, R0, |value| {
		value.wrapping_add(1)
	});
	test_single(Instruction::Dec { register: R0 }, R0, R0, |value| {
		value.wrapping_sub(1)
	});
	test_a_b(
		Instruction::Add {
			reg_dest: R0,
			reg_read: R1,
		},
		R0,
		R1,
		R0,
		|a, b| a.wrapping_add(b),
	);
	test_a_b(
		Instruction::Or {
			reg_dest: R0,
			reg_read: R1,
		},
		R0,
		R1,
		R0,
		|a, b| a | b,
	);
	test_a_b(
		Instruction::Eor {
			reg_dest: R0,
			reg_read: R1,
		},
		R0,
		R1,
		R0,
		|a, b| a ^ b,
	);
	test_a_b(
		Instruction::Mul {
			reg_dest: R0,
			reg_read: R1,
		},
		R0,
		R1,
		R0,
		|a, b| a.wrapping_mul(b),
	);
	test_a_b(
		Instruction::Sub {
			reg_dest: R0,
			reg_read: R1,
		},
		R0,
		R1,
		R0,
		|a, b| a.wrapping_sub(b),
	);
	test_a_b(
		Instruction::And {
			reg_dest: R0,
			reg_read: R1,
		},
		R0,
		R1,
		R0,
		|a, b| a & b,
	);
	test_word(
		Instruction::Adiw {
			register: WordRegister::R24,
			value: 42.into(),
		},
		WordRegister::R24,
		WordRegister::R24,
		|value| value.wrapping_add(42),
	);
	test_word(
		Instruction::Sbiw {
			register: WordRegister::R24,
			value: 42.into(),
		},
		WordRegister::R24,
		WordRegister::R24,
		|value| value.wrapping_sub(42),
	);
}
