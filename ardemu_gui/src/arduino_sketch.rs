use std::{
	borrow::Cow,
	path::PathBuf,
	process::{Command, Stdio},
};

use ardemu_core::{load_elf, Program};

pub fn compile_arduino_sketch(
	source_code: &str,
	arduino_cli_filepath: PathBuf,
) -> Result<Program, String> {
	let temp_arduino_sketch_dir = std::env::temp_dir().join("ardemu_arduino_sketch");
	std::fs::create_dir_all(&temp_arduino_sketch_dir).map_err(|e| e.to_string())?;

	let temp_arduino_sketch_output_dir = temp_arduino_sketch_dir.join("build_output");
	std::fs::create_dir_all(&temp_arduino_sketch_output_dir).map_err(|e| e.to_string())?;

	let temp_arduino_sketch_file = temp_arduino_sketch_dir.join("ardemu_arduino_sketch.ino");
	std::fs::write(&temp_arduino_sketch_file, source_code).map_err(|e| e.to_string())?;

	let compile_command_output = Command::new(arduino_cli_filepath)
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
		.map_err(|e| e.to_string())?
		.wait_with_output()
		.map_err(|e| e.to_string())?;

	if !compile_command_output.status.success() {
		return Err(format!(
			"Arduino compilation failed (non-zero exit code):{}{}",
			if compile_command_output.stderr.is_empty() {
				Cow::Borrowed("")
			} else {
				String::from_utf8_lossy(&compile_command_output.stderr)
			},
			if compile_command_output.stdout.is_empty() {
				Cow::Borrowed("")
			} else {
				String::from_utf8_lossy(&compile_command_output.stdout)
			}
		));
	}

	let output_elf_filepath = temp_arduino_sketch_output_dir.join("ardemu_arduino_sketch.ino.elf");

	let output_elf_content = std::fs::read(&output_elf_filepath).map_err(|e| e.to_string())?;

	let program = load_elf(&output_elf_content).map_err(|e| e.to_string())?;

	std::fs::remove_dir_all(&temp_arduino_sketch_dir).map_err(|e| e.to_string())?;
	Ok(program)
}
