#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
	// arduino-cli is not supported on wasm, so this is never set there
	#[cfg(not(target_arch = "wasm32"))]
	pub arduino_cli_filepath: Option<PathBuf>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Settings {
	const FILENAME: &str = "settings.toml";

	fn get_filepath() -> Option<PathBuf> {
		Some(
			directories::ProjectDirs::from("", "", "ardemu")?
				.config_local_dir()
				.to_path_buf()
				.join(Self::FILENAME),
		)
	}

	fn load_from_file(filepath: &Path) -> Option<Self> {
		let file_content = std::fs::read_to_string(filepath).ok()?;
		let settings: Self = toml::from_str(&file_content).ok()?;
		Some(settings)
	}

	fn save_to_file(&self, filepath: &Path) {
		match toml::to_string_pretty(self) {
			Ok(file_content) => {
				if let Some(parent) = filepath.parent() {
					let _ = std::fs::create_dir_all(parent);
				}
				if let Err(e) = std::fs::write(filepath, file_content) {
					eprintln!("Failed to save settings: {e}");
				}
			}
			Err(e) => {
				eprintln!("Failed to serialize settings: {e}");
			}
		}
	}

	pub fn load() -> Option<Self> {
		Self::get_filepath().and_then(|filepath| Self::load_from_file(&filepath))
	}

	pub fn save(&self) {
		match Self::get_filepath() {
			Some(filepath) => self.save_to_file(&filepath),
			None => eprintln!("Failed to find settings filepath"),
		}
	}
}
