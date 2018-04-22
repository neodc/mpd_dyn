extern crate mpd;
extern crate rand;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use mpd::{Client, Query, Term, Idle, Status};
use mpd::song::QueuePlace;

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::env;

mod rng;
mod settings;
mod score;

use settings::Settings;
use rng::{Weighted, WeightedChoice};

lazy_static! {
    static ref SETTINGS: Settings = {
        let config_file = env::args().nth(1).unwrap_or("/etc/mpd-dyn/config.json".to_string());
		settings::Settings::from_config_file(config_file)
    };
}

fn main() {
    let mut conn = Client::connect(&SETTINGS.url).unwrap();
	conn.login(SETTINGS.password.as_ref()).unwrap();

	loop {
		match conn.status().unwrap() {

			Status{song: Some(QueuePlace{pos, ..}), ..} if pos > SETTINGS.keep_before => {
				conn.delete(0..(pos - SETTINGS.keep_before)).unwrap();
			}
			Status{queue_len, ..} if queue_len < SETTINGS.playlist_len => {
				add_music(&mut conn, SETTINGS.playlist_len - queue_len);
			}
			_ => {
				conn.wait(&[]).unwrap();
			}
		}
	}

}

fn add_music(conn: &mut Client, nb: u32) {
	let now = now();

	let list: Vec<_> = conn
		.list(&Term::File, &Query::default())
		.unwrap()
		.into_iter()
		.map(|file_path| {
			let score = get_score(conn, &file_path, now);

			Weighted{item: file_path, weight: score}
		})
		.collect();

	let mut rng = WeightedChoice::new(list);

	for _ in 0..nb {
		let song_to_add = rng.sample(&mut rand::thread_rng());

		println!("added: {}", song_to_add);

		conn.findadd(&Query::new().and(Term::File, song_to_add.as_ref())).unwrap();

		for action in SETTINGS.actions.iter() {
			action.exec(conn, &song_to_add, now, &mut HashMap::new());
		}
	}
}

fn now() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs()
}

fn get_stickers(conn: &mut Client, file_path: &str) -> Result<HashMap<String, String>, ()> {

	conn.stickers_map("song", file_path)
		.map_err(|_| ())
}

fn get_score(conn: &mut Client, file_path: &str, now: u64) -> u64 {

	let mut stickers: HashMap<String, String> = get_stickers(conn, file_path).unwrap_or_default();

	let old_stickers: HashMap<String, String> = stickers.clone();

	let score = SETTINGS.score_compute
		.iter()
		.fold(
			0,
			|score, compute| compute.compute(now, score, &mut stickers)
		);

	stickers.retain(|key, value| {
		let old = old_stickers.get(key);

		if let Some(old) = old {
			if old == value {
				return false;
			}
		}

		true
	});

	for (key, value) in stickers {

		let _ = conn.set_sticker("song", file_path, &key, &value);
	}

	score
}
