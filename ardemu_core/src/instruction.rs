use crate::{
	AsmOperand, FlagType, Imm16, Imm3, Imm8, LowerEvenRegister, Register, RegisterAddress,
	UpperRegister, WordAddress, WordOffset16, WordOffset8, WordRegister,
};
use ardemu_instruction_helper_macro::{
	DisplayInstruction, ParseAsmInstruction, ReferencedRegisters,
};
use self_rust_tokenize::SelfRustTokenize;

#[derive(
	Debug,
	DisplayInstruction,
	ReferencedRegisters,
	Clone,
	Copy,
	PartialEq,
	Eq,
	Hash,
	SelfRustTokenize,
	ParseAsmInstruction,
)]
pub enum Instruction {
	/// No operation
	Nop,
	/// Break will pause execution, returning CpuStatus
	Break,
	/// jump to absolute address: PC = address
	#[skip_parse_asm_instruction]
	Jmp { word_address: WordAddress },
	/// Logical or and stores it in reg_dest
	/// reg_dest = reg_dest | reg_read
	Or {
		reg_dest: Register,
		reg_read: Register,
	},
	/// Logical or of register and immediate, stores it in reg_dest
	/// register = register | value
	Ori {
		register: UpperRegister,
		value: Imm8,
	},
	/// Logical exclusive or and stores it in reg_dest
	/// reg_dest = reg_dest ^ reg_read
	Eor {
		reg_dest: Register,
		reg_read: Register,
	},
	/// One's complement of register and stores it in register
	/// register = 0xFF - register
	Com { register: Register },
	/// Negate value: Two's complement of register and stores it in register
	/// register = 0x00 - register
	Neg { register: Register },
	/// Swap high and low nibbles of register and stores it in register
	/// register = (register >> 4) | (register << 4)     // rotate right by 4 bits: "half"
	Swap { register: Register },
	/// Logical shift right of register and stores it in register
	/// register = register >> 1
	Lsr { register: Register },
	/// Logical rotate right of register through carry and stores it in register
	/// C -> register (bits shifted one to the right) -> C
	Ror { register: Register },
	/// Arithmetic shift right of register: shifts all bits to the right, bit 7 stays constant, bit 0 is loaded into carry
	Asr { register: Register },
	/// Multiply unsigned 8 bit values and stores 16 bit result in R1:R0
	/// R1:R0 = reg_dest * reg_read
	Mul {
		reg_dest: Register,
		reg_read: Register,
	},
	/// load immediate value into upper register: register = value
	Ldi {
		register: UpperRegister,
		value: Imm8,
	},
	/// moves value of reg_read into reg_dest
	Mov {
		reg_dest: Register,
		reg_read: Register,
	},
	/// copies 16 bit value of rd+r:rd into rr+1:rr
	/// reg_dest+1:reg_dest = reg_read+1:reg_read
	Movw {
		reg_dest: LowerEvenRegister,
		reg_read: LowerEvenRegister,
	},
	/// jump relative to current PC with a offset
	/// technically a 12 bit offset value
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	RJmp { word_offset: WordOffset16 },
	/// pushes register onto stack:
	/// SP--
	/// write value of register into new stack pointer address
	Push { register: Register },
	/// pops register from stack:
	/// read value from stack pointer address into register
	/// SP++
	Pop { register: Register },
	/// copares upper register value to value (no write to register)
	/// reg_dest - value
	Cpi {
		register: UpperRegister,
		value: Imm8,
	},
	/// compares register values (no write to register)
	/// reg_dest - reg_read
	Cp {
		reg_dest: Register,
		reg_read: Register,
	},
	/// compares register values with carry (no write to register)
	/// reg_dest - reg_read - carry
	Cpc {
		reg_dest: Register,
		reg_read: Register,
	},
	/// compares registers and skips next instruction if equal
	Cpse {
		reg_dest: Register,
		reg_read: Register,
	},
	/// branch if equal (Z flag is 1)
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	/// offset is technically a 7 bit offset value
	#[skip_parse_asm_instruction]
	Breq { word_offset: WordOffset8 },
	/// branch if not equal (Z flag is 0)
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	/// offset is technically a 7 bit offset value
	#[skip_parse_asm_instruction]
	Brne { word_offset: WordOffset8 },
	/// branch if signed less than (S flag is 1)
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	/// offset is technically a 7 bit offset value
	#[skip_parse_asm_instruction]
	Brlt { word_offset: WordOffset8 },
	/// branch if carry flag is set (C flag is 1)
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	/// offset is technically a 7 bit offset value
	#[skip_parse_asm_instruction]
	Brcs { word_offset: WordOffset8 },
	/// branch if carry flag is cleared (C flag is 0)
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	/// offset is technically a 7 bit offset value
	#[skip_parse_asm_instruction]
	Brcc { word_offset: WordOffset8 },
	/// call subroutine at address:
	/// ; PC + 2: return address: this instruction itself + next instruction as return address
	/// push (PC + 1) onto stack
	/// PC = address
	#[skip_parse_asm_instruction]
	Call { word_address: WordAddress },
	/// return from subroutine:
	/// pop return address from stack into PC:
	/// ; basically
	/// PC = pop()
	Ret,
	/// call subroutine relative to current address
	#[skip_parse_asm_instruction]
	RCall { word_offset: WordOffset16 },
	/// subtract register values and stores result in reg_dest (without carry)
	/// reg_dest = reg_dest - reg_read
	Sub {
		reg_dest: Register,
		reg_read: Register,
	},
	/// subtract register values with carry and stores result in reg_dest
	/// reg_dest = reg_dest - reg_read - carry
	Sbc {
		reg_dest: Register,
		reg_read: Register,
	},
	/// subtract immediate value from upper register
	/// register = register - value
	Subi {
		register: UpperRegister,
		value: Imm8,
	},
	/// subtract immediate value with carry from upper register
	/// only resets zero flag if result is not zero -> Z=1 doesnt mean result is zero
	/// register = register - value - carry
	Sbci {
		register: UpperRegister,
		value: Imm8,
	},
	/// subtract immediate value from word register
	/// register = register - value
	Sbiw { register: WordRegister, value: Imm8 },
	/// decrement register value
	/// register = register - 1
	Dec { register: Register },
	/// adds register values and stores result in reg_dest (without carry)
	/// reg_dest = reg_dest + reg_read
	Add {
		reg_dest: Register,
		reg_read: Register,
	},
	/// adds register values with carry and stores result in reg_dest
	/// reg_dest = reg_dest + reg_read + carry
	Adc {
		reg_dest: Register,
		reg_read: Register,
	},
	/// adds immediate value to word register
	/// register = register + value
	Adiw { register: WordRegister, value: Imm8 },
	/// increment register value
	/// register = register + 1
	Inc { register: Register },
	/// logical AND with register values, result stored in reg_dest
	/// reg_dest = reg_dest & reg_read
	And {
		reg_dest: Register,
		reg_read: Register,
	},
	/// logical AND register with immediate value, result stored in register
	/// register = register & value
	Andi {
		register: UpperRegister,
		value: Imm8,
	},
	/// set cpu flag
	/// (same as CLI when flag_type is FlagType::Interrupt)
	Bset { flag_type: FlagType },
	/// clear cpu flag
	Bclr { flag_type: FlagType },
	/// Set bit in register (argument is the limited io address 0x00 - 0x1F)
	Sbi {
		register_address: RegisterAddress,
		bit: Imm3,
	},
	/// Clear bit in register (argument is the limited io address 0x00 - 0x1F)
	Cbi {
		register_address: RegisterAddress,
		bit: Imm3,
	},
	/// bit store from bit in register to T bit in SREG (FlagType::BitCopy)
	Bst { register: Register, bit: Imm3 },
	/// bit load T bit in SREG (FlagType::BitCopy) into bit in register
	Bld { register: Register, bit: Imm3 },
	/// store value of register into sram address
	Sts { address: Imm16, register: Register },
	/// load value from sram address into register
	Lds { register: Register, address: Imm16 },
	/// load value from sram address into register
	In { register: Register, address: Imm8 },
	/// store value of register into sram address
	Out { address: Imm8, register: Register },
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryAddressRange {
	SingleByte(u32),
	// 1 word = 2 bytes
	SingleWord(u32),
}

impl MemoryAddressRange {
	/// Checks if the given address is included in the range.
	pub fn includes_address(&self, address: u32) -> bool {
		match self {
			MemoryAddressRange::SingleByte(address_range) => address == *address_range,
			MemoryAddressRange::SingleWord(address_range) => {
				//										- 1, since the stack grows backwards
				address == *address_range || address == *address_range - 1
			}
		}
	}
}

impl Instruction {
	pub fn get_referenced_memory_address_range(
		&self,
		stack_pointer: u16,
	) -> Option<MemoryAddressRange> {
		match self {
			Self::Sts { address, .. } => Some(MemoryAddressRange::SingleByte(address.0 as u32)),
			Self::Lds { address, .. } => Some(MemoryAddressRange::SingleByte(address.0 as u32)),
			Self::In { address, .. } => Some(MemoryAddressRange::SingleByte(address.0 as u32)),
			Self::Out { address, .. } => Some(MemoryAddressRange::SingleByte(address.0 as u32)),
			Self::Push { .. } => Some(MemoryAddressRange::SingleByte(stack_pointer as u32)),
			Self::Pop { .. } => Some(MemoryAddressRange::SingleByte(stack_pointer as u32 + 1)),
			Self::Call { .. } => Some(MemoryAddressRange::SingleWord(stack_pointer as u32)),
			Self::RCall { .. } => Some(MemoryAddressRange::SingleWord(stack_pointer as u32)),
			Self::Ret => Some(MemoryAddressRange::SingleWord(stack_pointer as u32)),
			_ => None,
		}
	}

	pub fn get_referenced_program_address(
		self,
		program_address_of_instruction: WordAddress,
		return_address_in_stack: WordAddress,
		is_currently_executing: bool,
	) -> Option<WordAddress> {
		match self {
			Self::Jmp { word_address } => Some(word_address),
			Self::Call { word_address } => Some(word_address),
			Self::RCall { word_offset } => Some(
				program_address_of_instruction
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1),
			),
			Self::Ret => {
				if is_currently_executing {
					Some(return_address_in_stack)
				} else {
					None
				}
			}
			Self::RJmp { word_offset } => Some(
				program_address_of_instruction
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1),
			),
			Self::Breq { word_offset } => Some(
				program_address_of_instruction
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1),
			),
			Self::Brne { word_offset } => Some(
				program_address_of_instruction
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1),
			),
			Self::Brlt { word_offset } => Some(
				program_address_of_instruction
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1),
			),
			Self::Brcs { word_offset } => Some(
				program_address_of_instruction
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1),
			),
			Self::Brcc { word_offset } => Some(
				program_address_of_instruction
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1),
			),
			_ => None,
		}
	}
}
