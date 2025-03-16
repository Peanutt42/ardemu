use ardemu_core::{
	load_ihex_str, FlagType, Instruction, Program,
	Register::{R1, R16, R17, R22, R23, R24, R28, R29},
	UpperRegister, WordAddress, WordOffset16, WordOffset8,
};

#[test]
fn load_fib_avr_rust_sample_hex_file() {
	match load_ihex_str(include_str!("../../sample_programs/rust_fib.hex")) {
		Ok(program) => {
			let expected_instructions = // see fib_avr_rust_sample/fib_avr_rust_sample.asm for reference
			[
				Instruction::Jmp { word_address: WordAddress(52) /* 52 words = 0x68 (104) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: WordAddress(62) /* 62 words = 0x7c (124) bytes */ },
				Instruction::Eor {
					reg_dest: R1,
					reg_read: R1
				},
				Instruction::Out {
					address: 0x3f.into(),
					register: R1
				},
				Instruction::Ldi {
					register: UpperRegister::R28,
					value: 0xff.into()
				},
				Instruction::Ldi {
					register: UpperRegister::R29,
					value: 0x08.into()
				},
				Instruction::Out {
					address: 0x3e.into(),
					register: R29
				},
				Instruction::Out {
					address: 0x3d.into(),
					register: R28
				},
				Instruction::Call { word_address: WordAddress(83) /* 83 words = 0xa6 (166) bytes */ },
				Instruction::Jmp { word_address: WordAddress(91) /* 91 words = 0xb6 (182) bytes */ },
				Instruction::Jmp { word_address: WordAddress(0) },
				Instruction::Push { register: R16 },
				Instruction::Push { register: R17 },
				Instruction::Mov { reg_dest: R17, reg_read: R24 },
				Instruction::Mov { reg_dest: R16, reg_read: R1 },
				Instruction::Cpi { register: UpperRegister::R17, value: 0x02.into() },
				Instruction::Brcs { word_offset: WordOffset8(8) /* 8 words = 16 bytes */ },
				Instruction::Mov { reg_dest: R24, reg_read: R17 },
				Instruction::Dec { register: R24 },
				Instruction::Call { word_address: WordAddress(0x40) /* 64 words = 128 bytes */ },
				Instruction::Add { reg_dest: R16, reg_read: R24 },
				Instruction::Subi { register: UpperRegister::R17, value: 0x02.into() },
				Instruction::Cpi { register: UpperRegister::R17, value: 0x02.into() },
				Instruction::Brcc { word_offset: WordOffset8(-8) /* -8 words = -16 bytes */ },
				Instruction::Add { reg_dest: R16, reg_read: R17 },
				Instruction::Mov { reg_dest: R24, reg_read: R16 },
				Instruction::Pop { register: R17 },
				Instruction::Pop { register: R16 },
				Instruction::Ret,
				Instruction::Ldi { register: UpperRegister::R24, value: 0x0A.into() },
				Instruction::Call { word_address: WordAddress(0x40) /* 64 words = 128 bytes */ },
				Instruction::Mov { reg_dest: R22, reg_read: R24 },
				Instruction::Eor { reg_dest: R23, reg_read: R23 },
				Instruction::Ldi { register: UpperRegister::R24, value: 0x00.into() },
				Instruction::Ldi { register: UpperRegister::R25, value: 0x00.into() },
				Instruction::Ret,
				// (CLI)
				Instruction::Bclr { flag_type: FlagType::Interrupt },
				Instruction::RJmp { word_offset: WordOffset16(-1) /* -1 words = -2 bytes */ },
			];

			let expected_program = Program::new(&expected_instructions);

			assert_eq!(
				program.len(),
				expected_program.len(),
				"program length should be {}, but is {}\noutput: {program:#?}\nexpected: {expected_program:#?}",
				expected_program.len(),
				program.len()
			);
			for i in 0..program.len() {
				let program_address = WordAddress(i as u32);
				assert_eq!(
					program.get(program_address),
					expected_program.get(program_address)
				);
			}
		}
		Err(e) => panic!("Error loading ihex: {e}"),
	}
}
