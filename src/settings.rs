use std::fs::File;
use std::path::Path;

use serde_json;

use score::{ScoreCompute, Action};

#[derive(Debug, Deserialize)]
pub struct Settings {
	pub password: String,
	pub url: String,
	pub playlist_len: u32,
	pub keep_before: u32,
	pub score_compute: Vec<ScoreCompute>,
	pub actions: Vec<Action>,
}

impl Settings {
	pub fn from_config_file<P: AsRef<Path>>(path: P) -> Self {
		let file = File::open(path).expect("Coudn't open config file");

		serde_json::from_reader(file).expect("Coudn't parse config file")
	}
}
