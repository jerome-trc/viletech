use std::{
	any::Any,
	sync::{atomic, Arc},
};

use arc_swap::ArcSwap;
use criterion::Criterion;
use crossbeam::epoch;
use haphazard::{AtomicPtr, HazardPointer};
use hybrid_rc::HybridRc;
use parking_lot::RwLock;
use slotmap::SlotMap;

criterion::criterion_group!(benches, atomics, epoch, dereference, rt_poly);
criterion::criterion_main!(benches);

fn atomics(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("Atomics");

	grp.bench_function("ARC Clone", |bencher| {
		let arc = Arc::new("???".to_string());

		bencher.iter_with_large_drop(|| {
			let _ = std::hint::black_box(Arc::clone(&arc));
		});
	});

	{
		grp.bench_function("ArcSwap Load", |bencher| {
			let arcswap = ArcSwap::from_pointee("???".to_string());

			bencher.iter(|| {
				let g = arcswap.load();
				let _ = std::hint::black_box(g.len());
			});
		});

		grp.bench_function("ArcSwap Store", |bencher| {
			bencher.iter_batched(
				|| ArcSwap::from_pointee("???".to_string()),
				|arcswap| {
					arcswap.store(Arc::new("!!!".to_string()));
					let _ = std::hint::black_box(arcswap);
				},
				criterion::BatchSize::SmallInput,
			);
		});

		grp.bench_function("ArcSwap RCU", |bencher| {
			bencher.iter_batched(
				|| ArcSwap::from_pointee("???".to_string()),
				|arcswap| {
					arcswap.rcu(|prev| prev.clone());
					let _ = std::hint::black_box(arcswap);
				},
				criterion::BatchSize::SmallInput,
			);
		});
	}

	{
		let aptr = AtomicPtr::from(Box::new("???".to_string()));

		grp.bench_function("Hazard Pointer Creation", |bencher| {
			bencher.iter(|| {
				let _ = std::hint::black_box(HazardPointer::new());
			});
		});

		grp.bench_function("Hazard Load", |bencher| {
			let mut haz = HazardPointer::new();

			bencher.iter(|| {
				let _ = std::hint::black_box(aptr.safe_load(&mut haz).unwrap());
			});
		});
	}

	{
		let arc = hybrid_rc::Arc::new("???".to_string());
		let rc = HybridRc::to_local(&arc).unwrap();

		grp.bench_function("Hybrid RC Clone", |bencher| {
			bencher.iter_with_large_drop(|| {
				let _ = std::hint::black_box(HybridRc::clone(&rc));
			});
		});

		grp.bench_function("Hybrid ARC Clone", |bencher| {
			bencher.iter_with_large_drop(|| {
				let _ = std::hint::black_box(HybridRc::clone(&arc));
			});
		});
	}

	{
		let rwlock = RwLock::new("???".to_string());

		grp.bench_function("RwLock Read", |bencher| {
			bencher.iter(|| {
				let _ = std::hint::black_box(rwlock.read());
			});
		});

		grp.bench_function("RwLock Write", |bencher| {
			bencher.iter(|| {
				let _ = std::hint::black_box(rwlock.write());
			});
		});
	}

	grp.finish();
}

fn epoch(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("Crossbeam Epoch");

	grp.bench_function("Pin", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(epoch::pin());
		});
	});

	grp.bench_function("Atomic Load", |bencher| {
		let ptr = epoch::Atomic::new("???".to_string());
		let guard = epoch::pin();

		bencher.iter(|| {
			let _ = std::hint::black_box(ptr.load(atomic::Ordering::Acquire, &guard));
		});
	});

	grp.finish();
}

fn dereference(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("Dereference");

	grp.bench_function("Arc Deref.", |bencher| {
		let string: Arc<str> = Arc::from("???".to_string());

		bencher.iter(|| {
			let _ = std::hint::black_box(string.len());
		});
	});

	grp.bench_function("Vec Index", |bencher| {
		let v = vec!["", "", ""];

		bencher.iter(|| {
			let _ = std::hint::black_box(v[2].len());
		});
	});

	grp.bench_function("Vec Index Unchecked", |bencher| {
		let v = vec!["", "", ""];

		bencher.iter(|| unsafe {
			let _ = std::hint::black_box(v.get_unchecked(2).len());
		})
	});

	grp.bench_function("SlotMap Index", |bencher| {
		let mut sm = SlotMap::new();
		let _ = sm.insert("");
		let _ = sm.insert("");
		let k2 = sm.insert("");

		bencher.iter(|| {
			let _ = std::hint::black_box(sm[k2].len());
		})
	});

	grp.bench_function("SlotMap Index Unchecked", |bencher| {
		let mut sm = SlotMap::new();
		let _ = sm.insert("");
		let _ = sm.insert("");
		let k2 = sm.insert("");

		bencher.iter(|| unsafe {
			let _ = std::hint::black_box(sm.get_unchecked(k2).len());
		})
	});

	grp.finish();
}

fn rt_poly(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("RT Polymorphism");

	grp.bench_function("Downcast", |bencher| {
		let b: Box<dyn Any> = Box::new("???");

		bencher.iter(|| {
			let d = b.downcast_ref::<&str>().unwrap();
			let _ = std::hint::black_box(d.len());
		});
	});

	grp.bench_function("Downcast Unchecked", |bencher| {
		let b: Box<dyn Any> = Box::new("???");

		bencher.iter(|| unsafe {
			let d = b.downcast_ref::<&str>().unwrap_unchecked();
			let _ = std::hint::black_box(d.len());
		});
	});

	grp.finish();
}