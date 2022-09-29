use std::path::{PathBuf};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
	pub addon_dir: PathBuf,
	pub addon_json: PathBuf,
}