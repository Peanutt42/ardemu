#[derive(Debug, Clone)]
struct CpuState {
    registers: [u8; 32], // General purpose registers (r0-r31)
    pc: i32,             // Program counter
    sram: [u8; 2048],    // SRAM (2KB)
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            registers: [0; 32],
            pc: 0,
            sram: [0; 2048],
        }
    }
}

impl CpuState {
    const DDRB_ADDR: usize = 0x24;
    const PORTB_ADDR: usize = 0x25;

    fn is_builtin_led_on(&self) -> bool {
        let ddrb = self.sram[Self::DDRB_ADDR];
        let portb = self.sram[Self::PORTB_ADDR];
        (ddrb & 0x20) != 0 && (portb & 0x20) != 0
    }
}

#[derive(Clone, Copy)]
enum Instruction {
    // Move immediate value to register
    Ldi { reg: usize, value: u8 },
    // Add two registers (store result in first)
    Add { rd: usize, rs: usize },
    // Jump to relative address
    Jmp { offset: i32 },
    // Stores value from register to address
    Store { reg: usize, addr: usize },
    // No operation
    Nop,
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Instruction::Ldi { reg, value } => write!(f, "LDI r{reg}, {value:#04x}"),
            Instruction::Add { rd, rs } => write!(f, "ADD r{rd}, r{rs}"),
            Instruction::Jmp { offset } => write!(f, "JMP {offset}"),
            Instruction::Store { reg, addr } => write!(f, "STORE r{reg}, {addr:#04x}"),
            Instruction::Nop => write!(f, "NOP"),
        }
    }
}

impl Instruction {
    fn execute(&self, state: &mut CpuState) {
        match *self {
            Instruction::Ldi { reg, value } => {
                state.registers[reg] = value;
                state.pc += 1;
            }
            Instruction::Add { rd, rs } => {
                state.registers[rd] = state.registers[rd].wrapping_add(state.registers[rs]);
                state.pc += 1;
            }
            Instruction::Jmp { offset } => {
                state.pc += offset;
            }
            Instruction::Store { reg, addr } => {
                state.sram[addr] = state.registers[reg];
                state.pc += 1;
            }
            Instruction::Nop => {
                state.pc += 1;
            }
        }
    }
}

fn main() {
    let program = [
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
        // unreachable
        Instruction::Nop,
    ];

    let mut state = CpuState::default();

    loop {
        if state.pc >= program.len() as i32 {
            println!("Program finished");
            break;
        }

        let instr = program[state.pc as usize];
        println!("{}: {instr:?}", state.pc);
        instr.execute(&mut state);
        println!(
            "\t\t-> r0={:#04x}, r1={:#04x}, LED={}",
            state.registers[0],
            state.registers[1],
            if state.is_builtin_led_on() {
                "HIGH"
            } else {
                "LOW"
            }
        );
    }
}
