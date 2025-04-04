use ardemu_core::{load_elf, Program};
use iced::{
	futures::{channel::mpsc::Sender, SinkExt, Stream},
	stream,
};
use std::{path::PathBuf, process::Stdio};
use tokio::{io::AsyncBufReadExt, process::Command};

use crate::{program_source::ProgramSourceMessage, Message};

pub fn compile_arduino_sketch_stream(
	source_code: String,
	arduino_cli_filepath: PathBuf,
) -> impl Stream<Item = Message> {
	stream::channel(100, |mut output| async move {
		let program = compile_arduino_sketch(source_code, arduino_cli_filepath, &mut output).await;

		let _ = output.send(Message::LoadProgram(program)).await;
	})
}

async fn compile_arduino_sketch(
	source_code: String,
	arduino_cli_filepath: PathBuf,
	output: &mut Sender<Message>,
) -> Result<Program, String> {
	let temp_arduino_sketch_dir = std::env::temp_dir().join("ardemu_arduino_sketch");
	std::fs::create_dir_all(&temp_arduino_sketch_dir).map_err(|e| e.to_string())?;

	let temp_arduino_sketch_output_dir = temp_arduino_sketch_dir.join("build_output");
	std::fs::create_dir_all(&temp_arduino_sketch_output_dir).map_err(|e| e.to_string())?;

	let temp_arduino_sketch_file = temp_arduino_sketch_dir.join("ardemu_arduino_sketch.ino");
	std::fs::write(&temp_arduino_sketch_file, source_code).map_err(|e| e.to_string())?;

	let mut compile_command_child_process = Command::new(arduino_cli_filepath)
		.args([
			"compile",
			"-b",
			"arduino:avr:uno",
			"--build-path",
			&temp_arduino_sketch_output_dir.to_string_lossy(),
			"--warnings",
			"all",
			"--verbose",
			"--no-color",
			&temp_arduino_sketch_dir.to_string_lossy(),
		])
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.map_err(|e| e.to_string())?;

	let mut output_clone = output.clone();
	let std_out_thread = compile_command_child_process.stdout.take().map(|stdout| {
		tokio::spawn(async move {
			let mut reader = tokio::io::BufReader::new(stdout);
			let mut line = String::new();
			while let Ok(bytes_read) = reader.read_line(&mut line).await {
				if bytes_read == 0 {
					break;
				}
				let _ = output_clone
					.send(ProgramSourceMessage::CompileCliOutput(std::mem::take(&mut line)).into())
					.await;
			}
		})
	});

	let compile_command_output = compile_command_child_process
		.wait_with_output()
		.await
		.map_err(|e| e.to_string())?;

	if !compile_command_output.status.success() {
		return Err(if compile_command_output.stderr.is_empty() {
			"Arduino compilation failed (non-zero exit code): no stderr output!".to_string()
		} else {
			String::from_utf8_lossy(&compile_command_output.stderr).to_string()
		});
	}

	if let Some(std_out_thread) = std_out_thread {
		let _ = std_out_thread.await;
	}

	let output_elf_filepath = temp_arduino_sketch_output_dir.join("ardemu_arduino_sketch.ino.elf");

	let output_elf_content = std::fs::read(&output_elf_filepath).map_err(|e| e.to_string())?;

	let program = load_elf(&output_elf_content).map_err(|e| e.to_string())?;

	std::fs::remove_dir_all(&temp_arduino_sketch_dir).map_err(|e| e.to_string())?;

	Ok(program)
}
