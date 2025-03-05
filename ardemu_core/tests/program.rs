use ardemu_core::{Instruction, Program, WordAddress, WordOffset16};

#[test]
fn test_program_iterator() {
	let program = Program::new(&[
		Instruction::Jmp {
			word_address: WordAddress(0),
		},
		Instruction::Call {
			word_address: WordAddress(0),
		},
		Instruction::Ret,
		Instruction::RJmp {
			word_offset: WordOffset16(0),
		},
	]);
	let mut iter = program.iter();
	assert_eq!(
		iter.next(),
		Some((
			WordAddress(0x00),
			Instruction::Jmp {
				word_address: WordAddress(0)
			}
		))
	);
	assert_eq!(
		iter.next(),
		Some((
			WordAddress(0x02),
			Instruction::Call {
				word_address: WordAddress(0)
			}
		))
	);
	assert_eq!(iter.next(), Some((WordAddress(0x04), Instruction::Ret)));
	assert_eq!(
		iter.next(),
		Some((
			WordAddress(0x05),
			Instruction::RJmp {
				word_offset: WordOffset16(0)
			}
		))
	);
	assert_eq!(iter.next(), None);
}
