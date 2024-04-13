//! (GZ) A red-black tree implementation for building minisegs.

#[derive(Debug, Default)]
pub(super) struct EventTree {
	events: Vec<Event>,
	root: Option<EventKey>,
	spare: Option<EventKey>,
}

impl EventTree {
	#[must_use]
	pub(super) fn get_new_node(&mut self) -> EventKey {
		let ret;

		if let Some(spare) = self.spare {
			ret = spare;
			self.spare = self.events[ret.0].left;
		} else {
			ret = EventKey(self.events.len());
			self.events.push(Event::default());
		}

		ret
	}

	pub(super) fn insert(&mut self, key: EventKey) {
		let mut opt_x = self.root;
		let mut y = EventKey(usize::MAX);

		while let Some(x) = opt_x {
			y = x;

			if self.events[key.0].distance < self.events[x.0].distance {
				opt_x = self.events[x.0].left;
			} else {
				opt_x = self.events[x.0].right;
			}
		}

		self.events[key.0].parent = Some(y);

		if y.0 == usize::MAX {
			self.root = Some(key);
		} else if self.events[key.0].distance < self.events[y.0].distance {
			self.events[y.0].left = Some(key);
		} else {
			self.events[y.0].right = Some(key);
		}

		self.events[key.0].left = None;
		self.events[key.0].right = None;
	}

	pub(super) fn clear(&mut self) {
		self.events.clear();
		self.root = None;
		self.spare = None;
	}

	#[must_use]
	pub(super) fn successor(&self, event: EventKey) -> EventKey {
		if let Some(right) = self.events[event.0].right {
			let mut ret = right;

			while let Some(e) = self.events[ret.0].left {
				ret = e;
			}

			ret
		} else {
			let mut e = event;
			let mut opt_y = self.events[event.0].parent;

			loop {
				let Some(y) = opt_y else { break; };

				if self.events[y.0].right.filter(|r| *r == e).is_none() {
					break;
				}

				e = y;
				opt_y = self.events[y.0].parent;
			}

			opt_y.unwrap()
		}
	}

	#[must_use]
	pub(super) fn predecessor(&self, event: EventKey) -> EventKey {
		if let Some(left) = self.events[event.0].left {
			let mut ret = left;

			while let Some(e) = self.events[ret.0].right {
				ret = e;
			}

			ret
		} else {
			let mut e = event;
			let mut opt_y = self.events[event.0].parent;

			loop {
				let Some(y) = opt_y else { break; };

				if self.events[y.0].left.filter(|l| *l == e).is_none() {
					break;
				}

				e = y;
				opt_y = self.events[y.0].parent;
			}

			opt_y.unwrap()
		}
	}

	#[must_use]
	pub(super) fn find(&self, dist: f64) -> Option<EventKey> {
		let mut ret = self.root;

		while let Some(n) = ret {
			if self.events[n.0].distance == dist {
				return ret;
			} else if self.events[n.0].distance > dist {
				ret = self.events[n.0].left;
			} else {
				ret = self.events[n.0].right;
			}
		}

		None
	}

	#[must_use]
	pub(super) fn minimum(&self) -> Option<EventKey> {
		let mut ret = self.root;

		loop {
			let Some(r) = ret else { break; };
			ret = self.events[r.0].left;
		}

		ret
	}
}

#[derive(Debug)]
struct Event {
	parent: Option<EventKey>,
	left: Option<EventKey>,
	right: Option<EventKey>,
	distance: f64,
	vert: usize,
	front_seg: usize,
}

impl Default for Event {
	fn default() -> Self {
		Self {
			parent: None,
			left: None,
			right: None,
			distance: 0.0,
			vert: usize::MAX,
			front_seg: usize::MAX,
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct EventKey(usize);
