use rand::distributions::{Range, Distribution};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Weighted<T> {
	/// The numerical weight of this item
	pub weight: u64,
	/// The actual item which is being weighted
	pub item: T,
}

#[derive(Debug)]
pub struct WeightedChoice<T> {
	items: Vec<Weighted<T>>,
	sum: u64,
}

impl<T> WeightedChoice<T> {
	pub fn new(items: Vec<Weighted<T>>) -> Self {
		let sum = items.iter().map(|item| item.weight).sum();

		Self{items, sum}
	}

	pub fn sample<R: Rng>(&mut self, rng: &mut R) -> T {
		let range = Range::new(0, self.sum);

		let selected = range.sample(rng);

		let mut accumulator = 0;

		let mut pos = None;

		for (i, item) in self.items.iter_mut().enumerate() {
			accumulator += item.weight;

			if accumulator > selected {
				pos = Some(i);

				break;
			}
		}

		if let Some(pos) = pos {
			let item = self.items.swap_remove(pos);

			self.sum -= item.weight;

			item.item
		} else {
			unreachable!();
		}
	}
}
