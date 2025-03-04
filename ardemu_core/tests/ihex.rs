use ardemu_core::{
	load_ihex_str, Instruction,
	Register::{R1, R16, R17, R18, R19, R20, R24, R25, R28, R29},
	UpperRegister,
};

#[test]
fn load_fib_avr_rust_sample_hex_file() {
	match load_ihex_str(include_str!("fib_avr_rust_sample/fib_avr_rust_sample.hex")) {
		Ok(instructions) => {
			let expected_instructions = // see fib_avr_rust_sample/fib_avr_rust_sample.asm for reference
			[
				Instruction::Jmp { word_address: 52 /* 52 words = 0x68 (104) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
				Instruction::Jmp { word_address: 62 /* 62 words = 0x7c (124) bytes */ },
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
				Instruction::Call { word_address: 77 /* 77 words = 0x9a (154) bytes */ },
				Instruction::Jmp { word_address: 92 /* 92 words = 0xb8 (184) bytes */ },
				Instruction::Jmp { word_address: 0 },
				Instruction::Inc { register: R24 },
				Instruction::Ldi { register: UpperRegister::R18, value: 0x01.into() },
				Instruction::Mov { reg_dest: R20, reg_read: R1 },
				Instruction::Ldi {
					register: UpperRegister::R19,
					value: 0x01.into()
				},
				Instruction::Mov {
					reg_dest: R25,
					reg_read: R20
				},
				Instruction::Add {
					reg_dest: R19,
					reg_read: R25
				},
				Instruction::Dec { register: R24 },
				Instruction::Cpi {
					register: UpperRegister::R24,
					value: 0x00.into()
				},
				Instruction::Mov {
					reg_dest: R20,
					reg_read: R18
				},
				Instruction::Mov {
					reg_dest: R18,
					reg_read: R19
				},
				Instruction::Brne { word_offset: -7 /* -7 words = -14 bytes */ },
				Instruction::Mov {
					reg_dest: R24,
					reg_read: R25
				},
				Instruction::Ret,
				Instruction::Push { register: R16 },
				Instruction::Push { register: R17 },
				Instruction::Mov {
					reg_dest: R17,
					reg_read: R1
				},
				Instruction::Mov {
					reg_dest: R16,
					reg_read: R1
				},
				Instruction::Mov { reg_dest: R24, reg_read: R16 },
				Instruction::Call { word_address: 64 /* 64 words = 0x80 (128) bytes */ },
				Instruction::Add {
					reg_dest: R17,
					reg_read: R24
				},
				Instruction::Inc { register: R16 },
				Instruction::Cpi {
					register: UpperRegister::R16,
					value: 0xff.into()
				},
				Instruction::Brne { word_offset: -7 /* -7 words = -14 bytes */ },
				Instruction::Mov {
					reg_dest: R24,
					reg_read: R17
				},
				Instruction::Pop { register: R17 },
				Instruction::Pop { register: R16 },
				Instruction::Ret,
				Instruction::Cli,
				Instruction::RJmp { word_offset: -1 /* -1 words = -2 bytes */ },
			];
			assert_eq!(
				instructions.len(),
				expected_instructions.len(),
				"\noutput: {instructions:?}\nexpected: {expected_instructions:?}"
			);
			for i in 0..instructions.len() {
				assert_eq!(instructions[i], expected_instructions[i]);
			}
		}
		Err(e) => panic!("Error loading ihex: {e}"),
	}
}
