use crate::{
	register::{Imm16, Imm8, RegisterPair16, UpperRegister},
	Register,
};
use ardemu_display_instr_macro::DisplayInstruction;
use self_rust_tokenize::SelfRustTokenize;

#[derive(Debug, DisplayInstruction, Clone, Copy, PartialEq, Eq, Hash, SelfRustTokenize)]
pub enum Instruction {
	/// jump to absolute address: PC = address
	Jmp { address: Imm16 },
	/// Computes result of reg_dest ^ reg_read and stores it in reg_dest
	Eor {
		reg_dest: Register,
		reg_read: Register,
	},
	/* ============ TODO: out ============ */
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
	/// branch if not equal (Z flag is 0)
	/// PC = PC + offset + 1 (+1 because of the instruction itself)
	/// offset is technically a 7 bit offset value
	Brne { offset: i8 },
	/* ============ TODO: cpc ============ */
	/// call subroutine at address:
	/// ; PC + 2: return address: this instruction itself + next instruction as return address
	/// push (PC + 2) onto stack
	/// PC = address
	Call { address: Imm16 },
	/// return from subroutine:
	/// pop return address from stack into PC:
	/// ; basically
	/// PC = pop()
	Ret {},
	/// subtract immediate value from upper register
	/// register = register - value
	Subi {
		register: UpperRegister,
		value: Imm8,
	},
	/// adds register values and stores result in reg_dest (without carry)
	/// reg_dest = reg_dest + reg_read
	Add {
		reg_dest: Register,
		reg_read: Register,
	},
	/* ============ TODO: sbiw, brlt, subi, sbc, add, adc, andi ============ */
	/// store value of register into sram address
	Sts { address: Imm16, register: Register },
	/// load value from sram address into register
	Lds { register: Register, address: Imm16 },
	/* ============ TODO: sei, in, ori, adiw, reti, breq, cli, sbi ============ */
}
