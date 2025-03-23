use ardemu_core::{
	FlagType, Imm3, Instruction, LPMZPointerRegisterAction, LowerEvenRegister, Opcode,
	PointerRegister,
	Register::{R0, R15, R16, R31},
	RegisterAddress, UpperRegister, WordAddress, WordOffset16, WordOffset8, WordRegister,
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
		0b1111_0010_0000_1100,
		Instruction::Brlt {
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
	test_16bit(0b1001_0100_0111_1000, Instruction::SEI);
	test_16bit(0b1001_0100_1111_1000, Instruction::CLI);
	test_16bit(0b1001_0101_1111_0011, Instruction::Inc { register: R31 });
	test_16bit(
		0b0001_1100_1111_0000,
		Instruction::Adc {
			reg_dest: R15,
			reg_read: R0,
		},
	);
	test_16bit(
		0b1001_0110_1100_1111,
		Instruction::Adiw {
			register: WordRegister::R24,
			value: 63.into(),
		},
	);
	test_16bit(
		0b0010_0011_1111_1111,
		Instruction::And {
			reg_dest: R31,
			reg_read: R31,
		},
	);
	test_16bit(
		0b0111_1111_0000_1111,
		Instruction::Andi {
			register: UpperRegister::R16,
			value: 255.into(),
		},
	);
	test_16bit(
		0b1001_0100_0101_1000,
		Instruction::Bset {
			flag_type: FlagType::HalfCarry,
		},
	);
	test_16bit(
		0b1001_0100_1101_1000,
		Instruction::Bclr {
			flag_type: FlagType::HalfCarry,
		},
	);
	test_16bit(
		0b1001_1000_1111_1000,
		Instruction::Cbi {
			register_address: RegisterAddress(R31),
			bit: Imm3(0),
		},
	);
	test_16bit(
		0b1111_1011_1111_0001,
		Instruction::Bst {
			register: R31,
			bit: Imm3(1),
		},
	);
	test_16bit(
		0b1111_1001_1111_0001,
		Instruction::Bld {
			register: R31,
			bit: Imm3(1),
		},
	);
	test_32bit(
		0b1001_0001_1111_0000_1111_1111_1111_1111,
		Instruction::Lds {
			register: R31,
			address: 65535.into(),
		},
	);
	test_16bit(
		0b1011_0110_0000_1111,
		Instruction::In {
			register: R0,
			address: 63.into(),
		},
	);
	test_16bit(
		0b1101_0000_0000_0000 | (u16::from_le_bytes((-2047_i16).to_le_bytes()) & 0x0fff),
		Instruction::RCall {
			word_offset: WordOffset16(-2047),
		},
	);
	test_16bit(0b1001_0101_0001_1000, Instruction::Reti);
	test_16bit(
		0b1001_0001_1111_1100,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::X,
		},
	);
	test_16bit(
		0b1001_0001_1111_1101,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::X_POST_INC,
		},
	);
	test_16bit(
		0b1001_0001_1111_1110,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::X_PRE_DEC,
		},
	);
	test_16bit(
		0b1000_0001_1111_1000,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::Y,
		},
	);
	test_16bit(
		0b1001_0001_1111_1001,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::Y_POST_INC,
		},
	);
	test_16bit(
		0b1001_0001_1111_1010,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::Y_PRE_DEC,
		},
	);
	test_16bit(
		0b1000_0001_1111_0000,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::Z,
		},
	);
	test_16bit(
		0b1001_0001_1111_0001,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::Z_POST_INC,
		},
	);
	test_16bit(
		0b1001_0001_1111_0010,
		Instruction::Ld {
			register: R31,
			pointer_register: PointerRegister::Z_PRE_DEC,
		},
	);
	test_16bit(
		0b1001_0011_1111_1100,
		Instruction::St {
			pointer_register: PointerRegister::X,
			register: R31,
		},
	);
	test_16bit(
		0b1001_0011_1111_1101,
		Instruction::St {
			pointer_register: PointerRegister::X_POST_INC,
			register: R31,
		},
	);
	test_16bit(
		0b1001_0011_1111_1110,
		Instruction::St {
			pointer_register: PointerRegister::X_PRE_DEC,
			register: R31,
		},
	);
	test_16bit(
		0b1000_0011_1111_1000,
		Instruction::St {
			pointer_register: PointerRegister::Y,
			register: R31,
		},
	);
	test_16bit(
		0b1001_0011_1111_1001,
		Instruction::St {
			pointer_register: PointerRegister::Y_POST_INC,
			register: R31,
		},
	);
	test_16bit(
		0b1001_0011_1111_1010,
		Instruction::St {
			pointer_register: PointerRegister::Y_PRE_DEC,
			register: R31,
		},
	);
	test_16bit(
		0b1000_0011_1111_0000,
		Instruction::St {
			pointer_register: PointerRegister::Z,
			register: R31,
		},
	);
	test_16bit(
		0b1001_0011_1111_0001,
		Instruction::St {
			pointer_register: PointerRegister::Z_POST_INC,
			register: R31,
		},
	);
	test_16bit(
		0b1001_0011_1111_0010,
		Instruction::St {
			pointer_register: PointerRegister::Z_PRE_DEC,
			register: R31,
		},
	);
	test_16bit(
		0b1001_0001_1111_0101,
		Instruction::Lpm {
			register: R31,
			z_pointer_action: LPMZPointerRegisterAction::PostIncrement,
		},
	);
}

// tests every supported 16 bit instruction and check that the reproducing opcode is the same
#[test]
fn test_get_opcode_from_16bit_instruction() {
	for opcode_16bit in 0..u16::MAX {
		let opcode_32_bit = (opcode_16bit as u32) << 16;

		if let Some(instruction) = Instruction::load(opcode_32_bit) {
			if instruction.is_32bit() {
				continue;
			}

			let reproduced_opcode = instruction.get_opcode();
			assert_eq!(
				opcode_32_bit,
				reproduced_opcode,
				"expected: {opcode_16bit:#018b}, got: {:#018b} (instruction: {instruction})",
				reproduced_opcode >> 16
			);
		}
	}
}

#[test]
fn test_get_opcode_from_32bit_instruction() {
	fn test(opcode_32bit: u32) {
		let instruction =
			Instruction::load(opcode_32bit).expect("this opcode should be supported!");

		assert!(instruction.is_32bit());

		let produced_opcode_32bit = instruction.get_opcode();

		assert_eq!(
			opcode_32bit,
			produced_opcode_32bit,
			"expected: {opcode_32bit:#034b}, got: {produced_opcode_32bit:#034b}, (instruction: {instruction})"
		);
	}

	// Instruction::Sts { address: 32768.into(), register: R31 }
	test(0b1001_0011_1111_0000_1000_0000_0000_0000);

	// Instruction::Lds { register: R31, address: 65535.into() }
	test(0b1001_0001_1111_0000_1111_1111_1111_1111);

	// Instruction::Jmp { word_address: WordAddress(0xffff) }
	test(0b1001_0100_0000_1100_1111_1111_1111_1111);

	// Instruction::Call { word_address: WordAddress(0x3FFF) }
	test(0b1001_0100_0000_1110_0011_1111_1111_1111);
}
