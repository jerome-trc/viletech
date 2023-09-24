use std::cell::{Cell, RefCell};

criterion::criterion_group!(benches, strings);
criterion::criterion_main!(benches);

fn strings(crit: &mut criterion::Criterion) {
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
