use ardemu_core::{
	Instruction, LowerEvenRegister, Opcode,
	Register::{R0, R15, R16, R31},
	UpperRegister, WordAddress, WordOffset16, WordOffset8, WordRegister,
};

#[test]
fn test_load_instructions_from_opcodes() {
	fn test_16bit(opcode_16bit: u16, expected_instruction: Instruction) {
		let instruction = Instruction::load((opcode_16bit as u32) << 16).unwrap_or_else(|| {
			panic!("failed to load instruction {expected_instruction} from opcode {opcode_16bit:#018b}")
		});

		assert_eq!(
			instruction, expected_instruction,
			"16-bit Opcode {opcode_16bit:016b} should be {expected_instruction}, but got {instruction}"
		);
	}

	fn test_32bit(opcode_32bit: u32, expected_instruction: Instruction) {
		let instruction = Instruction::load(opcode_32bit).unwrap_or_else(|| {
			panic!("failed to load instruction {expected_instruction} from opcode {opcode_32bit:#034b}")
		});

		assert_eq!(
			instruction, expected_instruction,
			"32-bit Opcode {opcode_32bit:032b} should be {expected_instruction}, but got {instruction}"
		);
	}

	test_16bit(0b0000_0000_0000_0000, Instruction::Nop);
	test_16bit(
		0b0010_1001_1111_1111,
		Instruction::Or {
			reg_dest: R31,
			reg_read: R15,
		},
	);
	test_16bit(
		0b0110_0010_0000_1010,
		Instruction::Ori {
			register: UpperRegister::R16,
			// 42
			value: 0b10_1010.into(),
		},
	);
	test_16bit(
		0b1011_1110_1111_1111,
		Instruction::Out {
			address: 0b11_1111.into(),
			register: R15,
		},
	);
	test_16bit(0b1001_0001_0000_1111, Instruction::Pop { register: R16 });
	test_16bit(0b1001_0011_0000_1111, Instruction::Push { register: R16 });
	test_16bit(0b1001_0101_0000_1000, Instruction::Ret);
	test_16bit(
		0b1100_0000_0000_0000 | (u16::from_le_bytes((-2047_i16).to_le_bytes()) & 0x0fff),
		Instruction::RJmp {
			word_offset: WordOffset16(-2047),
		},
	);
	test_16bit(0b1001_0101_1111_0111, Instruction::Ror { register: R31 });
	test_16bit(
		0b0000_1001_1111_1111,
		Instruction::Sbc {
			reg_dest: R31,
			reg_read: R15,
		},
	);
	test_16bit(
		0b0100_0010_0000_1010,
		Instruction::Sbci {
			register: UpperRegister::R16,
			value: 42.into(),
		},
	);
	test_16bit(
		0b1001_1010_0111_1100,
		Instruction::Sbi {
			register_address: R15.into(),
			bit: 4.try_into().unwrap(),
		},
	);
	test_16bit(
		0b1001_0111_1100_1111,
		Instruction::Sbiw {
			register: WordRegister::R24,
			value: 63.into(),
		},
	);
	test_32bit(
		0b1001_0011_1111_0000_1000_0000_0000_0000,
		Instruction::Sts {
			address: 32768.into(),
			register: R31,
		},
	);
	test_16bit(
		0b0001_1011_1111_0000,
		Instruction::Sub {
			reg_dest: R31,
			reg_read: R16,
		},
	);
	test_16bit(
		0b0101_1000_0000_0000,
		Instruction::Subi {
			register: UpperRegister::R16,
			value: 0b1000_0000.into(),
		},
	);
	test_16bit(0b1001_0101_1111_0010, Instruction::Swap { register: R31 });
	test_16bit(0b1001_0101_1001_1000, Instruction::Break);
	test_32bit(
		0b1001_0100_0000_1100_1111_1111_1111_1111,
		Instruction::Jmp {
			word_address: WordAddress(0xffff),
		},
	);
	test_16bit(
		0b0010_0111_1111_0000,
		Instruction::Eor {
			reg_dest: R31,
			reg_read: R16,
		},
	);
	test_16bit(0b1001_0101_1111_0000, Instruction::Com { register: R31 });
	test_16bit(0b1001_0101_1111_0001, Instruction::Neg { register: R31 });
	test_16bit(0b1001_0101_1111_0110, Instruction::Lsr { register: R31 });
	test_16bit(0b1001_0101_1111_0101, Instruction::Asr { register: R31 });
	test_16bit(
		0b1001_1111_0000_0000,
		Instruction::Mul {
			reg_dest: R16,
			reg_read: R16,
		},
	);
	test_16bit(
		0b1110_1111_0000_1111,
		Instruction::Ldi {
			register: UpperRegister::R16,
			value: 255.into(),
		},
	);
	test_16bit(
		0b0010_1111_0000_1111,
		Instruction::Mov {
			reg_dest: R16,
			reg_read: R31,
		},
	);
	test_16bit(
		0b0000_0001_1111_0000,
		Instruction::Movw {
			reg_dest: LowerEvenRegister::R30,
			reg_read: LowerEvenRegister::R0,
		},
	);
	test_16bit(
		0b0011_1000_1111_0000,
		Instruction::Cpi {
			register: UpperRegister::R31,
			value: 128.into(),
		},
	);
	test_16bit(
		0b0001_0110_0000_1111,
		Instruction::Cp {
			reg_dest: R0,
			reg_read: R31,
		},
	);
	test_16bit(
		0b0000_0111_1111_1111,
		Instruction::Cpc {
			reg_dest: R31,
			reg_read: R31,
		},
	);
	test_16bit(
		0b0001_0011_1111_1111,
		Instruction::Cpse {
			reg_dest: R31,
			reg_read: R31,
		},
	);
	test_16bit(
		0b1111_0010_0000_1001,
		Instruction::Breq {
			word_offset: WordOffset8(-63),
		},
	);
	test_16bit(
		0b1111_0110_0000_1001,
		Instruction::Brne {
			word_offset: WordOffset8(-63),
		},
	);
	test_16bit(
		0b1111_0010_0000_1000,
		Instruction::Brcs {
			word_offset: WordOffset8(-63),
		},
	);
	test_16bit(
		0b1111_0110_0000_1000,
		Instruction::Brcc {
			word_offset: WordOffset8(-63),
		},
	);
	test_32bit(
		0b1001_0100_0000_1110_0011_1111_1111_1111,
		Instruction::Call {
			word_address: WordAddress(0x3FFF),
		},
	);
	test_16bit(
		0b0000_1111_1111_1111,
		Instruction::Add {
			reg_dest: R31,
			reg_read: R31,
		},
	);
	test_16bit(0b1001_0101_1111_1010, Instruction::Dec { register: R31 });
	test_16bit(0b1001_0100_1111_1000, Instruction::Cli);
	test_16bit(0b1001_0101_1111_0011, Instruction::Inc { register: R31 });
}
