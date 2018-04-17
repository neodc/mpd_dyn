use xdg::BaseDirectories;
use config::{Config, File, Environment};

#[derive(Debug)]
pub struct Settings {
	pub password: String,
	pub url: String,
	pub playlist_len: u32,
	pub keep_before: u32,
}

impl Settings {
	pub fn from_env() -> Self {
		let xdg_dirs = BaseDirectories::with_prefix("mpdDyn").unwrap();

		let mut settings = Config::default();

		settings.merge(File::with_name( "/etc/mpd-dyn/config" ).required(false)).unwrap();

		if let Some(config_file_path) = xdg_dirs.find_config_file("config") {
			settings.merge(File::with_name( &config_file_path.to_str().unwrap() ).required(false)).unwrap();
		}

		settings.merge(Environment::with_prefix("mpd_dyn")).unwrap();

		Self {
			password: settings.get("password").expect("password not given in config file"),
			url: settings.get("url").expect("url not given in config file"),
			playlist_len: settings.get("length").expect("length not given in config file"),
			keep_before: settings.get("keep").expect("keep not given in config file"),
		}
	}
}