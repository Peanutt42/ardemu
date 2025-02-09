use ardemu::{include_asm, Cpu};

fn main() {
	/*let program = [
		// r0 = LOW
		Instruction::Ldi { reg: 0, value: 0 },
		// r1 = HIGH
		Instruction::Ldi {
			reg: 1,
			value: 0x20,
		},
		// addr 0x24 = DDRB, if 0x20 is set, used as OUTPUT
		Instruction::Store { reg: 1, addr: 0x24 },
		// turn LED on
		Instruction::Store { reg: 1, addr: 0x25 },
		// turn LED off
		Instruction::Store { reg: 0, addr: 0x25 },
		// loop back to 'turn LED on'
		Instruction::Jmp { offset: -2 },
	];*/

	let program = include_asm!("blink.asm");

	let mut cpu = Cpu::default();

	while let Some(instr) = cpu.get_current_instruction(&program) {
		if let Err(e) = cpu.execute(instr) {
			eprintln!("failed to execute instruction: {e}");
			return;
		}
		println!(
			"{}: {instr:?}\n\t-> r0={:#04x}, r1={:#04x}, LED={}",
			cpu.program_counter,
			cpu.registers[0],
			cpu.registers[1],
			if cpu.is_builtin_led_on() {
				"HIGH"
			} else {
				"LOW"
			}
		);
	}

	println!("Program finished");
}

#[cfg(test)]
mod tests {
	use super::*;
	use ardemu::*;

	#[test]
	fn test_execute_ldi() {
		let mut cpu = Cpu::default();
		cpu.execute(Instruction::Ldi {
			reg: 0,
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
		cpu.execute(Instruction::Add { rd: 0, rs: 1 }).unwrap();
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
		cpu.execute(Instruction::Store { reg: 0, addr: 0x42 })
			.unwrap();
		assert_eq!(cpu.sram[0x42], 0x42);
	}
}
