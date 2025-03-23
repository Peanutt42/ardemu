use std::{collections::HashSet, ops::RangeInclusive};

use crate::{
	get_bit_from_u8, set_bit_in_u8, u8s_from_u16, u8s_to_u16, CpuError, FlagType, Flags,
	Instruction, LowerEvenRegister, Opcode, PointerRegisterAction, Program, Register, WordAddress,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuStatus {
	Normal,
	/// the current instruction was not run, as it was marked as a breakpoint
	BreakpointHit,
	/// the current instruction was not run, as it is a break instruction
	BreakHit,
	ProgramFinished,
}

#[derive(Debug, Clone)]
pub struct Cpu {
	program: Program,
	registers: [u8; Register::COUNT],
	/// in words (1 word = 2 bytes)
	program_counter: WordAddress,
	stack_pointer: u16,
	/// Increments with each instruction executed
	cycle: u64,
	flags: Flags,
	sram: [u8; Self::SRAM_SIZE], // SRAM (64KB)
	/// contains the program address of the breakpoints
	breakpoints: HashSet<WordAddress>,
}

impl Cpu {
	/// 64 KB
	const SRAM_SIZE: usize = 64 * 1024;
	const STACK_START_ADDRESS: u16 = 0xFEFF;
	const STACK_END_ADDRESS: u16 = 0xFC00;

	/// pins
	const DDRB_ADDR: usize = 0x24;
	const PORTB_ADDR: usize = 0x25;

	pub fn new(program: Program) -> Self {
		Self {
			program,
			registers: [0; Register::COUNT],
			program_counter: WordAddress(0),
			stack_pointer: Self::STACK_START_ADDRESS,
			cycle: 0,
			flags: Flags::default(),
			sram: [0u8; Self::SRAM_SIZE],
			breakpoints: HashSet::new(),
		}
	}

	/// resets everything except the last loaded program
	pub fn reset(&mut self) {
		self.registers = [0; Register::COUNT];
		self.program_counter = WordAddress(0);
		self.stack_pointer = Self::STACK_START_ADDRESS;
		self.cycle = 0;
		self.sram = [0u8; Self::SRAM_SIZE];
		self.breakpoints.clear();
	}

	pub fn add_breakpoint(&mut self, address: WordAddress) {
		self.breakpoints.insert(address);
	}

	pub fn remove_breakpoint(&mut self, address: WordAddress) {
		self.breakpoints.remove(&address);
	}

	pub fn get_breakpoints(&self) -> &HashSet<WordAddress> {
		&self.breakpoints
	}

	pub fn flags(&self) -> Flags {
		self.flags
	}

	pub fn get_program(&self) -> &Program {
		&self.program
	}

	pub fn get_program_counter(&self) -> WordAddress {
		self.program_counter
	}

	pub fn get_stack_pointer(&self) -> u16 {
		self.stack_pointer
	}

	pub fn get_cycle(&self) -> u64 {
		self.cycle
	}

	pub fn get_current_instruction(&self) -> Option<Instruction> {
		self.program.get_instruction(self.program_counter)
	}

	pub fn is_builtin_led_on(&self) -> bool {
		let ddrb = self.sram[Self::DDRB_ADDR];
		let ddrb_is_output = (ddrb & 0x20) != 0;
		let portb = self.sram[Self::PORTB_ADDR];
		let builtin_led_on = (portb & 0x20) != 0;
		ddrb_is_output && builtin_led_on
	}

	pub fn read_register(&self, reg: impl Into<Register>) -> u8 {
		reg.into().read_from(&self.registers)
	}

	pub fn write_register(&mut self, reg: impl Into<Register>, value: u8) {
		reg.into().write_in(&mut self.registers, value);
	}

	/// reads from register pair: low_register+1:low_register
	pub fn read_register_pair16(&self, low_register: impl Into<LowerEvenRegister>) -> u16 {
		let low_register = low_register.into();
		let low = self.read_register(low_register);
		let high = self.read_register(low_register.get_higher_uneven_register());
		u8s_to_u16(low, high)
	}

	/// write to register pair: low_register+1:low_register
	pub fn write_register_pair16(
		&mut self,
		low_register: impl Into<LowerEvenRegister>,
		value: u16,
	) {
		let low_register = low_register.into();
		let [low_value, high_value] = u8s_from_u16(value);
		self.write_register(low_register.get_higher_uneven_register(), high_value);
		self.write_register(low_register, low_value);
	}

	pub fn read_ram(&self, address: u16) -> Result<u8, CpuError> {
		let ram = self
			.sram
			.get(address as usize)
			.ok_or(CpuError::InvalidRamAddress {
				addr: address.into(),
			})?;
		Ok(*ram)
	}

	pub fn read_ram_range(&self, address_range: RangeInclusive<u16>) -> Result<&[u8], CpuError> {
		self.sram
			.get(*address_range.start() as usize..=*address_range.end() as usize)
			.ok_or(CpuError::InvalidRamAddress {
				addr: (*address_range.start()).into(),
			})
	}

	pub fn write_ram(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
		let mut_ram = self
			.sram
			.get_mut(address as usize)
			.ok_or(CpuError::InvalidRamAddress {
				addr: address.into(),
			})?;
		*mut_ram = value;
		Ok(())
	}

	fn push(&mut self, value: u8) -> Result<(), CpuError> {
		if self.stack_pointer <= Self::STACK_END_ADDRESS {
			return Err(CpuError::StackOverflow);
		}
		self.sram[self.stack_pointer as usize] = value;
		self.stack_pointer -= 1;
		Ok(())
	}
	fn pop(&mut self) -> Result<u8, CpuError> {
		self.stack_pointer += 1;
		if self.stack_pointer > Self::STACK_START_ADDRESS {
			return Err(CpuError::StackUnderflow);
		}
		let value = self.sram[self.stack_pointer as usize];
		Ok(value)
	}
	fn push_address(&mut self, address: WordAddress) -> Result<(), CpuError> {
		let [low, high] = u8s_from_u16(address.0 as u16);
		self.push(low)?;
		self.push(high)?;
		Ok(())
	}
	fn pop_address(&mut self) -> Result<WordAddress, CpuError> {
		let high = self.pop()?;
		let low = self.pop()?;
		Ok(u8s_to_u16(low, high).into())
	}
	/// returns the return address in the stack that would be popped if the Ret instruction would be executed
	/// this will return invalid word addresses if the stack does not have a return address to be popped
	pub fn peek_return_address(&self) -> WordAddress {
		let high = self.sram[self.stack_pointer as usize + 1];
		let low = self.sram[self.stack_pointer as usize + 2];
		u8s_to_u16(low, high).into()
	}

	pub fn execute(&mut self, instruction: Instruction) -> Result<CpuStatus, CpuError> {
		if self.breakpoints.contains(&self.program_counter) {
			return Ok(CpuStatus::BreakpointHit);
		}

		let mut cycles = 1;

		match instruction {
			Instruction::Nop => {
				self.program_counter += 1;
			}
			Instruction::Break => {
				return Ok(CpuStatus::BreakHit);
			}
			Instruction::Jmp { word_address } => {
				self.program_counter = word_address;
				// jmp is 3 cycles
				cycles += 2;
			}
			Instruction::Eor { reg_dest, reg_read } => {
				let result = self.read_register(reg_dest) ^ self.read_register(reg_read);
				self.write_register(reg_dest, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Com { register } => {
				let result = 0xFF - self.read_register(register);
				self.write_register(register, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Neg { register } => {
				let value = self.read_register(register);
				let result = 0_u8.wrapping_sub(value);
				self.write_register(register, result);
				self.flags.set_neg_znsvch(value, result);
				self.program_counter += 1;
			}
			Instruction::Swap { register } => {
				let value = self.read_register(register);
				let result = value.rotate_right(4);
				self.write_register(register, result);
				self.program_counter += 1;
			}
			Instruction::Lsr { register } => {
				let value = self.read_register(register);
				let result = value >> 1;
				self.write_register(register, result);
				self.flags.set_lsr_znsvc(value, result);
				self.program_counter += 1;
			}
			Instruction::Ror { register } => {
				let value = self.read_register(register);
				let carry_bit = if self.flags.carry() { 0x80 } else { 0x00 };
				let result = carry_bit | (value >> 1);
				self.write_register(register, result);
				self.flags.set_znsvc(value, result);
				self.program_counter += 1;
			}
			Instruction::Asr { register } => {
				let value = self.read_register(register);
				let result = (value >> 1) | (value & 0x80);
				self.write_register(register, result);
				self.flags.set_znsvc(value, result);
				self.program_counter += 1;
			}
			Instruction::Mul { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result: u16 = dest_value as u16 * read_value as u16;
				self.write_register_pair16(LowerEvenRegister::R0, result);
				self.flags.set_mul_zc(result);
				self.program_counter += 1;
				// mul is 2 cycles
				cycles += 1;
			}
			Instruction::Or { reg_dest, reg_read } => {
				let result = self.read_register(reg_dest) | self.read_register(reg_read);
				self.write_register(reg_dest, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Ori { register, value } => {
				let result = self.read_register(register) | value.0;
				self.write_register(register, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Ldi { register, value } => {
				self.write_register(register, value.0);
				self.program_counter += 1;
			}
			Instruction::Mov { reg_dest, reg_read } => {
				self.write_register(reg_dest, self.read_register(reg_read));
				self.program_counter += 1;
			}
			Instruction::Movw { reg_dest, reg_read } => {
				let value = self.read_register_pair16(reg_read);
				self.write_register_pair16(reg_dest, value);
				self.program_counter += 1;
			}
			Instruction::RJmp { word_offset } => {
				self.program_counter = self
					.program_counter
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1);
			}
			Instruction::Push { register } => {
				self.push(self.read_register(register))?;
				self.program_counter += 1;
			}
			Instruction::Pop { register } => {
				let value = self.pop()?;
				self.write_register(register, value);
				self.program_counter += 1;
			}
			Instruction::Cpi { register, value } => {
				let register_value = self.read_register(register);
				let result = register_value.wrapping_sub(value.0);
				self.flags.set_sub_znsvch(register_value, value.0, result);
				self.program_counter += 1;
			}
			Instruction::Cp { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value.wrapping_sub(read_value);
				self.flags.set_sub_znsvch(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Cpc { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value
					.wrapping_sub(read_value)
					.wrapping_sub(self.flags.carry_u8());
				self.flags.set_sub_znsvch(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Cpse { reg_dest, reg_read } => {
				if self.read_register(reg_dest) == self.read_register(reg_read) {
					let next_instruction_word_size = match self
						.program
						.get_instruction(self.get_program_counter() + 1u32)
					{
						Some(next_instruction) => next_instruction.get_word_size(),
						None => 1,
					};
					self.program_counter += 1 + next_instruction_word_size;
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Breq { word_offset } => {
				if self.flags.zero() {
					cycles += 1;
					self.program_counter = self
						.program_counter
						.wrapping_add_signed(word_offset)
						.wrapping_add_signed(1);
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Brne { word_offset } => {
				if !self.flags.zero() {
					cycles += 1;
					self.program_counter = self
						.program_counter
						.wrapping_add_signed(word_offset)
						.wrapping_add_signed(1);
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Brlt { word_offset } => {
				if self.flags.sign() {
					cycles += 1;
					self.program_counter = self
						.program_counter
						.wrapping_add_signed(word_offset)
						.wrapping_add_signed(1);
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Brcs { word_offset } => {
				if self.flags.carry() {
					cycles += 1;
					self.program_counter = self
						.program_counter
						.wrapping_add_signed(word_offset)
						.wrapping_add_signed(1);
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Brcc { word_offset } => {
				if !self.flags.carry() {
					cycles += 1;
					self.program_counter = self
						.program_counter
						.wrapping_add_signed(word_offset)
						.wrapping_add_signed(1);
				} else {
					self.program_counter += 1;
				}
			}
			Instruction::Call { word_address } => {
				self.push_address(self.program_counter + instruction.get_word_size() as u16)?;
				self.program_counter = word_address;
				// Call is 4 cycles
				cycles += 3;
			}
			Instruction::Ret => {
				self.program_counter = self.pop_address()?;
				// Ret is 4 cycles
				cycles += 3;
			}
			Instruction::Reti => {
				self.program_counter = self.pop_address()?;
				self.flags.set(FlagType::Interrupt);
				// Reti is 4 cycles
				cycles += 3;
			}
			Instruction::RCall { word_offset } => {
				self.push_address(self.program_counter + instruction.get_word_size() as u16)?;
				self.program_counter = self
					.program_counter
					.wrapping_add_signed(word_offset)
					.wrapping_add_signed(1);
				// RCall is mostly 2 cycles
				cycles += 1;
			}
			Instruction::Sub { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value.wrapping_sub(read_value);
				self.write_register(reg_dest, result);
				self.flags.set_sub_znsvch(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Sbc { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value
					.wrapping_sub(read_value)
					.wrapping_sub(self.flags.carry_u8());
				self.write_register(reg_dest, result);
				self.flags.set_sub_znsvch(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Subi { register, value } => {
				let register_value = self.read_register(register);
				let result = register_value.wrapping_sub(value.0);
				self.write_register(register, result);
				self.flags.set_sub_znsvch(register_value, value.0, result);
				self.program_counter += 1;
			}
			Instruction::Sbci { register, value } => {
				let register_value = self.read_register(register);
				let result = register_value
					.wrapping_sub(value.0)
					.wrapping_sub(self.flags.carry_u8());
				self.write_register(register, result);
				self.flags.set_sub_rznsvch(register_value, value.0, result);
				self.program_counter += 1;
			}
			Instruction::Sbiw { register, value } => {
				let register_value = self.read_register_pair16(register);
				let result = register_value.wrapping_sub(value.0 as u16);
				self.write_register_pair16(register, result);
				self.flags.set_zns16(result);
				self.program_counter += 1;
			}
			Instruction::Dec { register } => {
				let register_value = self.read_register(register);
				let result = register_value.wrapping_sub(1);
				self.write_register(register, result);
				self.flags.set_sub_znsvch(register_value, 1, result);
				self.program_counter += 1;
			}
			Instruction::Add { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value.wrapping_add(read_value);
				self.write_register(reg_dest, result);
				self.flags.set_add_znsvch(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Adc { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value
					.wrapping_add(read_value)
					.wrapping_add(self.flags.carry_u8());
				self.write_register(reg_dest, result);
				self.flags.set_add_znsvch(dest_value, read_value, result);
				self.program_counter += 1;
			}
			Instruction::Adiw { register, value } => {
				let register_value = self.read_register_pair16(register);
				let result = register_value.wrapping_add(value.0.into());
				self.write_register_pair16(register, result);
				self.flags.set_zns16(result);
				self.program_counter += 1;
			}
			Instruction::Inc { register } => {
				let register_value = self.read_register(register);
				let result = register_value.wrapping_add(1);
				self.write_register(register, result);
				self.flags.set_add_znsvch(register_value, 1, result);
				self.program_counter += 1;
			}
			Instruction::And { reg_dest, reg_read } => {
				let dest_value = self.read_register(reg_dest);
				let read_value = self.read_register(reg_read);
				let result = dest_value & read_value;
				self.write_register(reg_dest, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Andi { register, value } => {
				let register_value = self.read_register(register);
				let result = register_value & value.0;
				self.write_register(register, result);
				self.flags.set_zns_v0(result);
				self.program_counter += 1;
			}
			Instruction::Bset { flag_type } => {
				self.flags.set(flag_type);
				self.program_counter += 1;
			}
			Instruction::Bclr { flag_type } => {
				self.flags.clear(flag_type);
				self.program_counter += 1;
			}
			Instruction::Sbi {
				register_address,
				bit,
			} => {
				let value = self.read_register(register_address.0);
				self.write_register(register_address.0, set_bit_in_u8(value, bit.0, true));
				self.program_counter += 1;
			}
			Instruction::Cbi {
				register_address,
				bit,
			} => {
				let value = self.read_register(register_address.0);
				self.write_register(register_address.0, set_bit_in_u8(value, bit.0, false));
				self.program_counter += 1;
			}
			Instruction::Bst { register, bit } => {
				let bit = get_bit_from_u8(self.read_register(register), bit.0);
				self.flags.set_copy_bit(bit);
				self.program_counter += 1;
			}
			Instruction::Bld { register, bit } => {
				let bit_value = self.flags.bit_copy();
				self.write_register(
					register,
					set_bit_in_u8(self.read_register(register), bit.0, bit_value),
				);
				self.program_counter += 1;
			}
			Instruction::Sts { address, register } => {
				self.write_ram(address.0, self.read_register(register))?;
				self.program_counter += instruction.get_word_size();
				// sts is 2 cycles
				cycles += 1;
			}
			Instruction::Lds { register, address } => {
				self.write_register(register, self.read_ram(address.0)?);
				self.program_counter += instruction.get_word_size();
				// lds is 2 cycles
				cycles += 1;
			}
			Instruction::St {
				pointer_register,
				register,
			} => {
				let action = pointer_register.action();

				let mut pointer_value = self.read_register_pair16(pointer_register);
				if let PointerRegisterAction::PreDecrement = action {
					pointer_value -= 1;
				}
				self.write_ram(pointer_value, self.read_register(register))?;
				if let PointerRegisterAction::PostIncrement = action {
					pointer_value += 1;
				}
				self.write_register_pair16(pointer_register, pointer_value);

				self.program_counter += 1;
				// st is 2 cycles
				cycles += 1;
			}
			Instruction::Ld {
				register,
				pointer_register,
			} => {
				let action = pointer_register.action();

				let mut pointer_value = self.read_register_pair16(pointer_register);
				if let PointerRegisterAction::PreDecrement = action {
					pointer_value -= 1;
				}
				self.write_register(register, self.read_ram(pointer_value)?);
				if let PointerRegisterAction::PostIncrement = action {
					pointer_value += 1;
				}
				self.write_register_pair16(pointer_register, pointer_value);

				self.program_counter += 1;
				// ld is 2 cycles
				cycles += 1;
			}
			Instruction::Out { address, register } => {
				self.write_ram(address.0 as u16, self.read_register(register))?;
				self.program_counter += 1;
			}
			Instruction::In { register, address } => {
				self.write_register(register, self.read_ram(address.0 as u16)?);
				self.program_counter += 1;
			}
		}

		self.cycle += cycles;

		Ok(CpuStatus::Normal)
	}

	pub fn step(&mut self) -> Result<CpuStatus, CpuError> {
		match self.get_current_instruction() {
			Some(instruction) => self.execute(instruction),
			None => Ok(CpuStatus::ProgramFinished),
		}
	}

	/// Skips the current instruction, even if it is a break instruction.
	pub fn skip(&mut self) {
		if let Some(current_instruction) = self.get_current_instruction() {
			self.program_counter += current_instruction.get_word_size();
		}
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new(Program::default())
	}
}
