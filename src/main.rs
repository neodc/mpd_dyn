extern crate mpd;
extern crate rand;
extern crate config;
extern crate xdg;

use mpd::{Client, Query, Term, Idle, Status};
use mpd::song::QueuePlace;

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

mod rng;
mod settings;

use rng::{Weighted, WeightedChoice};

const STICKER_TIME: &str = "time";
const STICKER_FAV: &str = "fav";

fn main() {
	let settings = settings::Settings::from_env();

    let mut conn = Client::connect(settings.url).unwrap();
	conn.login(settings.password.as_ref()).unwrap();

	loop {
		match conn.status().unwrap() {

			Status{song: Some(QueuePlace{pos, ..}), ..} if pos > settings.keep_before => {
				conn.delete(0..(pos - settings.keep_before)).unwrap();
			}
			Status{queue_len, ..} if queue_len < settings.playlist_len => {
				add_music(&mut conn, settings.playlist_len - queue_len);
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
			get_stickers(conn, &file_path)
				.map(|stickers| get_score(conn, &file_path, now, &stickers))
				.map(|score| (file_path, score))
		})
		.filter_map(Result::ok)
		.map(|(item, weight)| Weighted{item, weight})
		.collect();

	let mut rng = WeightedChoice::new(list);

	for _ in 0..nb {
		let song_to_add = rng.sample(&mut rand::thread_rng());

		println!("added: {}", song_to_add);

		conn.findadd(&Query::new().and(Term::File, song_to_add.as_ref())).unwrap();

		conn.set_sticker("song", &song_to_add, STICKER_TIME, &format!("{}", now)).unwrap();
	}
}

fn update_time(conn: &mut Client, file_path: &str, time: u64) {

	let _ = conn.set_sticker("song", file_path, STICKER_TIME, &time.to_string());
}

fn now() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs()
}

fn get_stickers(conn: &mut Client, file_path: &str) -> Result<HashMap<String, String>, ()> {

	conn.stickers_map("song", file_path)
		.map_err(|_| ())
}

fn get_score(conn: &mut Client, file_path: &str, now: u64, stickers: &HashMap<String, String>) -> u64 {

	let time: u64 = stickers.get(STICKER_TIME)
		.and_then(|time| time.parse().ok())
		.unwrap_or_else(|| {update_time(conn, file_path, now); now });


	let mut score = now.saturating_sub(time).pow(2);

	if stickers.contains_key(STICKER_FAV) {
		score *= 5;
	}

	score
}
