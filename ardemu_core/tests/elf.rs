use ardemu_core::{
	load_elf, Instruction, Program,
	Register::{R1, R16, R17, R22, R23, R24, R28, R29},
	UpperRegister, WordAddress, WordOffset16, WordOffset8,
};

#[test]
fn test_load_elf_file() {
	match load_elf(include_bytes!("../../sample_programs/rust_fib.elf")) {
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
				Instruction::CLI,
				Instruction::RJmp { word_offset: WordOffset16(-1) /* -1 words = -2 bytes */ },
			];

			let expected_program = Program::load_instruction_list(&expected_instructions);

			assert_eq!(&program.flash, &expected_program.flash);

			assert_eq!(
				program.get_debug_symbol(WordAddress(0)).unwrap(),
				"__vectors"
			);
			assert_eq!(
				program.get_debug_symbol(WordAddress(52)).unwrap(),
				"__ctors_end"
			);
			assert_eq!(
				program.get_debug_symbol(WordAddress(62)).unwrap(),
				"__bad_interrupt"
			);
			assert_eq!(
				program.get_debug_symbol(WordAddress(64)).unwrap(),
				"_ZN8rust_fib3fib17h52828e8768a34918E"
			);
			assert_eq!(program.get_debug_symbol(WordAddress(83)).unwrap(), "main");
			assert_eq!(program.get_debug_symbol(WordAddress(91)).unwrap(), "_exit");
			assert_eq!(
				program.get_debug_symbol(WordAddress(92)).unwrap(),
				"__stop_program"
			);
		}
		Err(e) => panic!("Error loading rust_fib.elf: {e}"),
	}
}
