use crate::{CpuError, Instruction, Register};

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

	fn get_register_value(&self, reg: Register) -> u8 {
		reg.get_from(&self.registers)
	}

	fn set_register_value(&mut self, reg: Register, value: u8) {
		reg.set_in(&mut self.registers, value);
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
				self.set_register_value(reg, value);
				self.program_counter += 1;
			}
			Instruction::Add { rd, rs } => {
				let rd_value = self.get_register_value(rd);
				let rs_value = self.get_register_value(rs);
				self.set_register_value(rd, rd_value.wrapping_add(rs_value));
				self.program_counter += 1;
			}
			Instruction::Jmp { offset } => {
				self.program_counter = (self.program_counter as i32 + offset) as usize;
			}
			Instruction::Store { reg, addr } => {
				let register_value = self.get_register_value(reg);
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use super::*;

	#[test]
	fn test_execute_ldi() {
		let mut cpu = Cpu::default();
		cpu.execute(Instruction::Ldi {
			reg: Register::R0,
			value: 0x42,
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 0x42);
	}

	#[test]
	fn test_execute_add() {
		let mut cpu = Cpu::default();
		cpu.registers[0] = 0x42;
		cpu.registers[1] = 0x23;
		cpu.execute(Instruction::Add {
			rd: Register::R0,
			rs: Register::R1,
		})
		.unwrap();
		assert_eq!(cpu.registers[0], 0x65);
	}

	#[test]
	fn test_execute_jmp() {
		let mut cpu = Cpu::default();
		cpu.registers[0] = 0x42;
		cpu.registers[1] = 0x23;
		cpu.execute(Instruction::Jmp { offset: 0x42 }).unwrap();
		assert_eq!(cpu.program_counter, 0x42);
	}

	#[test]
	fn test_execute_store() {
		let mut cpu = Cpu::default();
		cpu.registers[0] = 0x42;
		cpu.registers[1] = 0x23;
		cpu.execute(Instruction::Store {
			reg: Register::R0,
			addr: 0x42,
		})
		.unwrap();
		assert_eq!(cpu.sram[0x42], 0x42);
	}
}
