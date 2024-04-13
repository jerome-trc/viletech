/// Alternatively a "map".
#[derive(Debug)]
pub struct LevelDef {
	pub meta: LevelMeta,
	pub format: LevelFormat,
	pub bounds: MinMaxBox,
	pub geom: LevelGeom,
	pub bsp: LevelBsp,
	pub thingdefs: Vec<ThingDef>,
}

impl LevelDef {
	#[must_use]
	pub fn bounds(vertdefs: &[Vertex]) -> MinMaxBox {
		let mut min = glam::vec3a(0.0, 0.0, 0.0);
		let mut max = glam::vec3a(0.0, 0.0, 0.0);

		for vert in vertdefs {
			if vert.x < min.x {
				min.x = vert.x;
			} else if vert.x > max.x {
				max.x = vert.x;
			}

			if vert.y < min.y {
				min.y = vert.y;
			} else if vert.y > max.y {
				max.y = vert.y;
			}

			if vert.bottom() < min.z {
				min.z = vert.bottom();
			}

			if vert.top() > max.z {
				max.z = vert.top();
			}
		}

		MinMaxBox { min, max }
	}

	/// (GZ) Collision detection against lines with 0.0 length can cause zero-division,
	/// so use this to remove them. Returns the number of lines pruned.
	pub fn prune_0len_lines(&mut self) -> usize {
		let mut n = 0;

		for i in 0..self.geom.linedefs.len() {
			let linedef = &self.geom.linedefs[i];
			let v1 = &self.geom.vertdefs[linedef.vert_start];
			let v2 = &self.geom.vertdefs[linedef.vert_end];

			if std::ptr::eq(v1, v2) {
				continue;
			}

			if i != n {
				self.geom.linedefs[n] = self.geom.linedefs[i].clone();
			}

			n += 1;
		}

		let l = self.geom.linedefs.len();
		self.geom.linedefs.truncate(n);
		l - n
	}

	/// (GZ) Sides not referenced by any lines are just wasted space,
	/// and can be removed. Returns the number of sides pruned.
	pub fn prune_unused_sides(&mut self) -> usize {
		let mut used: BitVec<AtomicUsize, Lsb0> = BitVec::with_capacity(self.geom.sidedefs.len());
		used.resize(self.geom.sidedefs.len(), false);
		let mut remap: Vec<usize> = Vec::with_capacity(self.geom.sidedefs.len());

		self.geom.linedefs.par_iter().for_each(|linedef| {
			used.set_aliased(linedef.side_right, true);
			let Some(side_left) = linedef.side_left else {
				return;
			};
			used.set_aliased(side_left, true);
		});

		// SAFETY: `AtomicUsize` has identical representation to `usize`.
		let used = unsafe { std::mem::transmute::<_, BitVec<usize, Lsb0>>(used) };

		let mut new_len = 0;

		for i in 0..self.geom.sidedefs.len() {
			if !used[i] {
				remap[i] = usize::MAX;
				continue;
			}

			if i != new_len {
				self.geom.sidedefs.swap(new_len, i);
			}

			remap[i] = new_len;
			new_len += 1;
		}

		let ret = self.geom.sidedefs.len() - new_len;

		if ret > 0 {
			self.geom.sidedefs.truncate(new_len);

			// Re-assign linedefs' side indices.

			self.geom.linedefs.par_iter_mut().for_each(|linedef| {
				linedef.side_right = remap[linedef.side_right];
				let Some(side_left) = linedef.side_left.as_mut() else {
					return;
				};
				*side_left = remap[*side_left];
			});
		}

		ret
	}

	/// (GZ) Sectors not referenced by any sides are just wasted space,
	/// and can be removed. Returns a "remap table" for use in fixing REJECT tables.
	pub fn prune_unused_sectors(&mut self) -> Vec<usize> {
		let mut used: BitVec<AtomicUsize, Lsb0> = BitVec::with_capacity(self.geom.sectordefs.len());
		used.resize(self.geom.sectordefs.len(), false);
		let mut remap: Vec<usize> = Vec::with_capacity(self.geom.sectordefs.len());

		self.geom.sidedefs.par_iter_mut().for_each(|sidedef| {
			used.set_aliased(sidedef.sector, true);
		});

		// SAFETY: `AtomicUsize` has identical representation to `usize`.
		let used = unsafe { std::mem::transmute::<_, BitVec<usize, Lsb0>>(used) };

		let mut new_len = 0;

		for i in 0..self.geom.sectordefs.len() {
			if !used[i] {
				remap[i] = usize::MAX;
				continue;
			}

			if i != new_len {
				self.geom.sectordefs.swap(new_len, i);
			}

			remap[i] = new_len;
			new_len += 1;
		}

		let mut ret = vec![];

		if new_len < self.geom.sectordefs.len() {
			// Re-assign sidedefs' sector indices.

			self.geom.sidedefs.par_iter_mut().for_each(|sidedef| {
				sidedef.sector = remap[sidedef.sector];
			});

			// (GZ) Make a reverse map for fixing reject lumps.
			ret.resize(new_len, usize::MAX);

			for i in 0..self.geom.sectordefs.len() {
				ret[remap[i]] = i;
			}

			self.geom.sectordefs.truncate(new_len);
		}

		ret
	}

	#[must_use]
	pub fn is_udmf(&self) -> bool {
		matches!(self.format, LevelFormat::Udmf(_))
	}
}
