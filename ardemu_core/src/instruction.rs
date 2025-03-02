use crate::{
	AsmOperand, FlagType, Imm16, Imm3, Imm8, Register, RegisterAddress, RegisterPair16,
	UpperRegister, WordRegister,
};
use ardemu_asm_parse_macro::ParseAsmInstruction;
use ardemu_display_instr_macro::DisplayInstruction;
use self_rust_tokenize::SelfRustTokenize;

#[derive(
	Debug,
	DisplayInstruction,
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
	Jmp { address: Imm16 },
	/// Logical or and stores it in reg_dest
	/// reg_dest = reg_dest | reg_read
	Or {
		reg_dest: Register,
		reg_read: Register,
	},
	/// Logical or of register and immediate, stores it in reg_dest
	/// register = register | value
	Ori { register: Register, value: Imm8 },
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
		reg_dest: RegisterPair16,
		reg_read: RegisterPair16,
	},
	/// jump relative to current PC with a offset
	/// technically a 12 bit offset value
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	RJmp { offset: i16 },
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
	Breq { offset: i8 },
	/// branch if not equal (Z flag is 0)
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	/// offset is technically a 7 bit offset value
	#[skip_parse_asm_instruction]
	Brne { offset: i8 },
	/// branch if signed less than (S flag is 1)
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	/// offset is technically a 7 bit offset value
	#[skip_parse_asm_instruction]
	Brlt { offset: i8 },
	/// call subroutine at address:
	/// ; PC + 2: return address: this instruction itself + next instruction as return address
	/// push (PC + 1) onto stack
	/// PC = address
	#[skip_parse_asm_instruction]
	Call { address: Imm16 },
	/// return from subroutine:
	/// pop return address from stack into PC:
	/// ; basically
	/// PC = pop()
	Ret,
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
	Sbiw {
		register: WordRegister,
		value: Imm16,
	},
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
	Adiw {
		register: WordRegister,
		value: Imm16,
	},
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
