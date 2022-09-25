use std::path::{PathBuf, Path};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
	pub addon_dir: PathBuf,
}

impl Config {
	pub fn new<P: AsRef<Path>>(path: P) -> Self {
		Self {
			addon_dir: PathBuf::from(path.as_ref()),
		}
	}
}