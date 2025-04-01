use std::{
	sync::mpsc::{Receiver, TryRecvError},
	time::{Duration, Instant},
};

use ardemu_core::{Cpu, CpuStatus};

use crate::{CpuSim, CpuSimMessage};

pub fn cpu_simulation_thread(
	receiver: Receiver<CpuSimMessage>,
	mut cpu: Cpu,
	mut writable_cpu_sim: triple_buffer::Input<CpuSim>,
) {
	let mut simulate_cpu = false;
	let mut realtime_speed = false;

	loop {
		if simulate_cpu {
			let start = Instant::now();
			let start_cycle = cpu.get_cycle();

			if realtime_speed {
				// gui gets new cpu state at 144 fps
				const CPU_UPDATES_PER_SECOND: u64 = 144;
				const CPU_STEPS_PER_FRAME: u64 = Cpu::FREQUENCY / CPU_UPDATES_PER_SECOND;

				let cpu_frame_update_duration =
					Duration::from_secs_f64(1.0 / CPU_UPDATES_PER_SECOND as f64);

				while (cpu.get_cycle() - start_cycle) < CPU_STEPS_PER_FRAME {
					match cpu.step() {
						Ok(cpu_status) => match cpu_status {
							CpuStatus::Normal => {}
							CpuStatus::BreakpointHit | CpuStatus::BreakHit => {
								break;
							}
							CpuStatus::ProgramFinished => {
								println!("Program finished");
								break;
							}
						},
						Err(e) => {
							eprintln!("failed to step cpu: {e}");
						}
					}
				}

				let compute_duration = start.elapsed();
				if compute_duration < cpu_frame_update_duration {
					std::thread::sleep(cpu_frame_update_duration - compute_duration);
				}
			} else {
				/// extra 1 in order to avoid having a repeating pattern, looking like the sim is not running.
				/// kind of like laminar flow
				const BULK_STEP_COUNT: usize = 1_000_001;

				for _ in 0..BULK_STEP_COUNT {
					match cpu.step() {
						Ok(cpu_status) => match cpu_status {
							CpuStatus::Normal => {}
							CpuStatus::BreakpointHit | CpuStatus::BreakHit => {
								break;
							}
							CpuStatus::ProgramFinished => {
								println!("Program finished");
								break;
							}
						},
						Err(e) => {
							eprintln!("failed to step cpu: {e}");
						}
					}
				}
			}

			let cycles_per_second =
				(cpu.get_cycle() - start_cycle) as f64 / start.elapsed().as_secs_f64();
			writable_cpu_sim.write(CpuSim {
				cpu: cpu.clone(),
				cycles_per_second,
			});
		}

		loop {
			let simulate_cpu_copy = simulate_cpu;

			let mut handle_message = |message: CpuSimMessage| {
				match message {
					CpuSimMessage::ResetAndLoadProgram(program) => {
						cpu = Cpu::new(program);
					}
					CpuSimMessage::Step => match cpu.step() {
						Ok(cpu_status) => match cpu_status {
							CpuStatus::Normal | CpuStatus::BreakpointHit | CpuStatus::BreakHit => {}
							CpuStatus::ProgramFinished => {
								println!("Program finished");
							}
						},
						Err(e) => {
							eprintln!("failed to step cpu: {e}");
						}
					},
					CpuSimMessage::Skip => {
						cpu.skip();
					}
					CpuSimMessage::SkipToInstruction(program_address) => {
						cpu.set_program_counter(program_address);
					}
					CpuSimMessage::SetSimulating(simulating) => {
						simulate_cpu = simulating;
					}
					CpuSimMessage::SetSimRealtimeSpeed(new_realtime_speed) => {
						realtime_speed = new_realtime_speed;
					}
					CpuSimMessage::AddBreakpoint(breakpoint_address) => {
						cpu.add_breakpoint(breakpoint_address);
					}
					CpuSimMessage::RemoveBreakpoint(breakpoint_address) => {
						cpu.remove_breakpoint(breakpoint_address);
					}
				}

				writable_cpu_sim.write(CpuSim {
					cpu: cpu.clone(),
					// single instruction is not recorded
					cycles_per_second: 0.0,
				});
			};

			if simulate_cpu_copy {
				match receiver.try_recv() {
					Ok(message) => handle_message(message),
					Err(e) => match e {
						TryRecvError::Empty => break,
						TryRecvError::Disconnected => return,
					},
				}
			} else {
				match receiver.recv() {
					Ok(message) => handle_message(message),
					Err(_) => return,
				}
			}
		}
	}
}
