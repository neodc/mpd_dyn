use std::collections::HashMap;

use mpd::Client;

#[derive(Debug, Deserialize)]
pub struct ScoreCompute {
	pub action: ScoreAction,
	pub condition: Option<ScoreCondition>,
	pub value: Value,
}

impl ScoreCompute {
	pub fn compute(&self, now: u64, previous_value: u64, stickers: &mut HashMap<String, String>) -> u64 {

		if let Some(ref condition) = self.condition {
			if !condition.check(now, previous_value, stickers) {
				return previous_value;
			}
		}

		if let Some(value) = self.value.compute(now, stickers) {
			self.action.apply(previous_value, value)
		} else {
			previous_value
		}
	}
}

#[derive(Debug, Deserialize)]
pub enum ScoreAction {
	#[serde(rename = "add")]
	Add,
	#[serde(rename = "sub")]
	Sub,
	#[serde(rename = "mul")]
	Mul,
	#[serde(rename = "pow")]
	Pow,
}

impl ScoreAction {
	pub fn apply(&self, previous_value: u64, value: u64) -> u64 {
		match *self {
			ScoreAction::Add => previous_value + value,
			ScoreAction::Sub => previous_value - value,
			ScoreAction::Mul => previous_value * value,
			ScoreAction::Pow => previous_value.pow(value as u32),
		}
	}
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ScoreCondition {
	#[serde(rename = "sticker_exist")]
	StickerExist{
		name: String,
	},
}

impl ScoreCondition {
	pub fn check(&self, _now: u64, _previous_value: u64, stickers: &HashMap<String, String>) -> bool {
		match *self {
			ScoreCondition::StickerExist {ref name} => {
				stickers.contains_key(name)
			}
		}
	}
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Value {
	#[serde(rename = "now")]
	Now,
	#[serde(rename = "const")]
	Const{
		value: u64,
	},
	#[serde(rename = "sticker")]
	Sticker{
		name: String,
		default: Option<Box<DefaultValue>>,
	},
}

impl Value {
	pub fn compute(&self, now: u64, stickers: &mut HashMap<String, String>) -> Option<u64> {

		match *self {
			Value::Now => Some(now),
			Value::Const {value} => Some(value),
			Value::Sticker {ref name, ref default} => {
				let mut value = None;

				if let Some(v) = stickers.get(name) {
					value = v.parse().ok();
				}

				if value.is_none() {
					if let Some(default) = default {

						value = default.value.compute(now, stickers);

						if let Some(value) = value {
							if default.saved {

								stickers.insert(name.to_string(), value.to_string());
							}
						}
					}
				}

				value
			}
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct  DefaultValue {
	pub value: Value,
	#[serde(default)]
	pub saved: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
	#[serde(rename = "sticker_update")]
	StickerUpdate {
		name: String,
		value: Value,
	},
}

impl Action {
	pub fn exec(&self, conn: &mut Client, file_path: &str, now: u64, stickers: &mut HashMap<String, String>) -> () {

		match *self {
			Action::StickerUpdate {ref name, ref value} => {

				if let Some(value) = value.compute(now, stickers) {
					let _ = conn.set_sticker("song", file_path, name, &value.to_string());
				}
			}
		}

	}
}
