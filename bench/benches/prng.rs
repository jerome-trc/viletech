//! Microbenchmarking different pseudo-random number generators.

use nanorand::{Rng, WyRand};
use rand_core::{RngCore, SeedableRng};
use sfmt::SFMT;

criterion::criterion_group!(benches, wyrand, sfmt);
criterion::criterion_main!(benches);

fn wyrand(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("WyRand");

	grp.bench_function("0..256", |bencher| {
		let mut rng = WyRand::new();

		bencher.iter(|| {
			let _ = std::hint::black_box(rng.generate_range(0_u32..256_u32));
		});
	});

	grp.finish();
}

fn sfmt(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("SFMT");

	grp.bench_function("0..256", |bencher| {
		let mut rng = SFMT::from_entropy();

		bencher.iter(|| {
			let _ = std::hint::black_box(rng.next_u32() % 256);
		});
	});

	grp.finish();
}
