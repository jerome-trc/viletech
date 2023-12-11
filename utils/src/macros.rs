//! Some useful general-purpose macros. See the [crate docs](crate).

/// Courtesy of Reddit user [YatoRust].
///
/// See <https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html>.
///
/// [YatoRust]: <https://www.reddit.com/r/rust/comments/d3yag8/the_little_book_of_rust_macros/>
#[macro_export]
macro_rules! count_tts {
	() => { 0 };
    ($odd:tt $($a:tt $b:tt)*) => { (count_tts!($($a)*) << 1) | 1 };
    ($($a:tt $even:tt)*) => { count_tts!($($a)*) << 1 };
}

#[macro_export]
macro_rules! replace_expr {
	($_t:tt $sub:expr) => {
		$sub
	};
}

/// Creates an anonymous block with a lazy-initialised static regular expression.
/// From <https://docs.rs/once_cell/latest/once_cell/index.html#lazily-compiled-regex>.
#[macro_export]
macro_rules! lazy_regex {
	($re:literal $(,)?) => {{
		static RGX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

		RGX.get_or_init(|| {
			regex::Regex::new($re).expect(concat!(
				"failed to evaluate regex: ",
				module_path!(),
				":",
				line!(),
				":",
				column!(),
			))
		})
	}};
}

/// See [`lazy_regex`].
#[macro_export]
macro_rules! lazy_regexset {
	($($re:literal),+) => {{
		static RGXSET: std::sync::OnceLock<regex::RegexSet> = std::sync::OnceLock::new();

		RGXSET.get_or_init(|| regex::RegexSet::new([$($re),+]).expect(
			concat!(
				"failed to evaluate regex set: ",
				module_path!(),
				":",
				line!(),
				":",
				column!(),
		)))
	}};
}
