use crate::{CpuError, Instruction};

const SRAM_SIZE: usize = 2048;

#[derive(Debug, Clone)]
pub struct Cpu {
	pub registers: [u8; 32], // General purpose registers (r0-r31)
	pub program_counter: usize,
	pub sram: [u8; SRAM_SIZE], // SRAM (2KB)
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			registers: [0; 32],
			program_counter: 0,
			sram: [0u8; SRAM_SIZE],
		}
	}

	pub fn is_builtin_led_on(&self) -> bool {
		let ddrb = self.get_ddrb();
		let portb = self.get_portb();
		(ddrb & 0x20) != 0 && (portb & 0x20) != 0
	}

	pub fn get_current_instruction(&self, instructions: &[Instruction]) -> Option<Instruction> {
		instructions.get(self.program_counter).copied()
	}

	fn get_ddrb(&self) -> u8 {
		self.sram[0x24]
	}

	fn get_portb(&self) -> u8 {
		self.sram[0x25]
	}

	fn get_register_value(&self, reg: usize) -> Result<u8, CpuError> {
		self.registers
			.get(reg)
			.copied()
			.ok_or(CpuError::InvalidRegister { reg })
	}

	fn set_register_value(&mut self, reg: usize, value: u8) -> Result<(), CpuError> {
		let mut_register = self
			.registers
			.get_mut(reg)
			.ok_or(CpuError::InvalidRegister { reg })?;
		*mut_register = value;
		Ok(())
	}

	fn set_ram(&mut self, addr: usize, value: u8) -> Result<(), CpuError> {
		let mut_ram = self
			.sram
			.get_mut(addr)
			.ok_or(CpuError::InvalidRamAddress { addr })?;
		*mut_ram = value;
		Ok(())
	}

	pub fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
		match instruction {
			Instruction::Ldi { reg, value } => {
				self.set_register_value(reg, value)?;
				self.program_counter += 1;
			}
			Instruction::Add { rd, rs } => {
				let rd_value = self.get_register_value(rd)?;
				let rs_value = self.get_register_value(rs)?;
				self.set_register_value(rd, rd_value.wrapping_add(rs_value))?;
				self.program_counter += 1;
			}
			Instruction::Jmp { offset } => {
				self.program_counter = (self.program_counter as i32 + offset) as usize;
			}
			Instruction::Store { reg, addr } => {
				let register_value = self.get_register_value(reg)?;
				self.set_ram(addr, register_value)?;
				self.program_counter += 1;
			}
			Instruction::Nop => {
				self.program_counter += 1;
			}
		}

		Ok(())
	}
}

impl Default for Cpu {
	fn default() -> Self {
		Self::new()
	}
}
