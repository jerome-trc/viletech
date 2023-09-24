use std::{
	any::Any,
	cell::{Cell, RefCell},
	sync::{atomic, Arc},
};

use append_only_vec::AppendOnlyVec;
use arc_swap::ArcSwap;
use criterion::Criterion;
use crossbeam::epoch;
use haphazard::{AtomicPtr, HazardPointer};
use hybrid_rc::HybridRc;
use parking_lot::RwLock;
use slotmap::SlotMap;

criterion::criterion_group!(
	benches,
	atomics,
	epoch,
	dereference,
	rt_poly,
	strings,
	threading
);
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
		grp.bench_function("AppendOnlyVec Push", |bencher| {
			bencher.iter_batched(
				|| AppendOnlyVec::new(),
				|aov| {
					let i = aov.push([0_usize, 0_usize, 0_usize]);
					let _ = std::hint::black_box(i);
				},
				criterion::BatchSize::SmallInput,
			);
		});

		grp.bench_function("AppendOnlyVec Access", |bencher| {
			let aov = AppendOnlyVec::new();
			let i = aov.push([0_usize, 0_usize, 0_usize]);

			bencher.iter(|| {
				let _ = std::hint::black_box(aov[i]);
			});
		});
	}

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

fn strings(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("Strings");

	// When building an extremely temporary string, is it faster to allocate anew
	// or to use thread-local storage?

	grp.bench_function("Temporary", |bencher| {
		bencher.iter(|| {
			let mut string = String::with_capacity(32);
			string.push_str("This");
			string.push_str(" fits");
			string.push_str(" within");
			string.push_str(" 31");
			string.push_str(" characters.");
			let _ = std::hint::black_box(string);
		});
	});

	{
		grp.bench_function("Temporary, TLS, RefCell", |bencher| {
			thread_local! {
				static BUF: RefCell<String> = RefCell::new(
					String::with_capacity(32)
				);
			}

			bencher.iter(|| {
				BUF.with(|refc| {
					let mut r = refc.borrow_mut();
					r.clear();
					r.push_str("This");
					r.push_str(" fits");
					r.push_str(" within");
					r.push_str(" 31");
					r.push_str(" characters.");
					let _ = std::hint::black_box(r);
				});
			});
		});
	}

	{
		grp.bench_function("Temporary, TLS, Cell", |bencher| {
			thread_local! {
				static BUF: Cell<String> = Cell::new(String::with_capacity(32));
			}

			bencher.iter(|| {
				BUF.with(|string| {
					let mut s = string.take();
					s.clear();
					s.push_str("This");
					s.push_str(" fits");
					s.push_str(" within");
					s.push_str(" 31");
					s.push_str(" characters.");
					let _ = std::hint::black_box(string.set(s));
				});
			});
		});
	}

	{
		grp.bench_function("Temporary, TLS, Unsafe", |bencher| {
			thread_local! {
				static BUF: String = String::with_capacity(32);
			}

			bencher.iter(|| {
				BUF.with(|string| unsafe {
					let ptr = std::ptr::addr_of!(*string).cast_mut();
					(*ptr).clear();
					(*ptr).push_str("This");
					(*ptr).push_str(" fits");
					(*ptr).push_str(" within");
					(*ptr).push_str(" 31");
					(*ptr).push_str(" characters.");
					let _ = std::hint::black_box(ptr);
				});
			});
		});
	}

	grp.finish();
}

fn threading(crit: &mut Criterion) {
	let mut grp = crit.benchmark_group("Threading");

	grp.bench_function("Spawn", |bencher| {
		bencher.iter(|| {
			let j = std::thread::spawn(|| {});
			let _ = std::hint::black_box(j.join().unwrap());
		});
	});

	grp.bench_function("Thread ID", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(std::thread::current().id());
		});
	});

	grp.bench_function("Available Parallelism", |bencher| {
		bencher.iter(|| {
			let _ = std::hint::black_box(std::thread::available_parallelism());
		});
	});

	grp.finish();
}
