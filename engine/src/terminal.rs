//! Abstraction for text-based UI, used by the headless server and client's console.

use util::lazy_regex;

/// This combines storage for text-based commands and aliases with a parser
/// for matching against those commands, allowing both the client's console
/// and headless server to seamlessly use the same code and UI.
pub struct Terminal<C: Command> {
	commands: Vec<CommandWrapper<C>>,
	command_not_found: fn(&str),
	aliases: Vec<Alias>,
}

impl<C: Command + std::fmt::Debug> std::fmt::Debug for Terminal<C> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Terminal")
			.field("commands", &self.commands)
			.field("aliases", &self.aliases)
			.finish()
	}
}

// Public interface.
impl<C: Command> Terminal<C> {
	pub fn new(command_not_found: fn(&str)) -> Self {
		Self {
			aliases: Vec::<Alias>::default(),
			commands: Vec::<CommandWrapper<C>>::default(),
			command_not_found,
		}
	}

	pub fn submit(&self, string: &str) -> Vec<C::Output> {
		let mut ret = Vec::<_>::default();
		let mut string = string.to_owned();

		// "Recursive" alias expansion, no more than 8 levels deep.

		for _ in 0..8 {
			let mut s = String::default();

			for token in string.split_whitespace() {
				if let Some(alias) = self.find_alias(token) {
					s.push_str(&alias.expanded);
				} else {
					s.push(' ');
					s.push_str(token);
				}
			}

			string = s;
		}

		let inputs = string.split(';');

		for input in inputs {
			let input = input.trim();
			let mut tokens = input.splitn(2, ' ');

			let key = if let Some(k) = tokens.next() {
				k
			} else {
				continue;
			};

			let args = if let Some(a) = tokens.next() { a } else { "" };
			let args_iter = lazy_regex!(r#"'([^']+)'|"([^"]+)"|([^'" ]+) *"#).captures_iter(args);
			let mut args = vec![key];

			for arg in args_iter {
				let arg_match = match arg.get(1).or_else(|| arg.get(2)).or_else(|| arg.get(3)) {
					Some(a) => a,
					None => {
						continue;
					}
				};

				args.push(arg_match.as_str());
			}

			match self.find_command(args[0]) {
				Some(cmd) => {
					ret.push(cmd.call(CommandArgs::new(args)));
				}
				None => {
					(self.command_not_found)(key);
				}
			};
		}

		ret
	}

	pub fn register_command(&mut self, id: &'static str, command: C, enabled: bool) {
		debug_assert!(!id.is_empty());
		debug_assert!(Self::id_valid(id));
		debug_assert!(!self.commands.iter().any(|cmd| cmd.id == id));

		self.commands.push(CommandWrapper {
			id: id.trim(),
			enabled,
			command,
		});
	}

	/// If the existing alias already exists, it gets replaced.
	pub fn register_alias(&mut self, alias: String, string: String) {
		debug_assert!(!alias.is_empty() && !string.is_empty());

		match self.aliases.iter().position(|a| a.alias == alias) {
			Some(pos) => {
				self.aliases[pos].alias = alias;
				self.aliases[pos].expanded = string;
			}
			None => {
				self.aliases.push(Alias {
					alias,
					expanded: string,
				});
			}
		};
	}

	/// If the given predicate returns `true` for a contained `C`,
	/// that command will be enabled.
	pub fn enable_commands(&mut self, predicate: fn(&C) -> bool) {
		for wrapper in &mut self.commands {
			if (predicate)(&wrapper.command) {
				wrapper.enabled = true;
			}
		}
	}

	/// If the given predicate returns `true` for a contained `C`,
	/// that command will be disabled.
	pub fn disable_commands(&mut self, predicate: fn(&C) -> bool) {
		for wrapper in &mut self.commands {
			if (predicate)(&wrapper.command) {
				wrapper.enabled = false;
			}
		}
	}

	pub fn enable_all_commands(&mut self) {
		for wrapper in &mut self.commands {
			wrapper.enabled = true;
		}
	}

	pub fn disable_all_commands(&mut self) {
		for wrapper in &mut self.commands {
			wrapper.enabled = false;
		}
	}

	pub fn all_commands(&self) -> impl Iterator<Item = (&'static str, &C)> {
		self.commands.iter().map(|w| (w.id, &w.command))
	}

	pub fn all_aliases(&self) -> impl Iterator<Item = &Alias> {
		self.aliases.iter()
	}

	#[must_use]
	pub fn find_command(&self, key: &str) -> Option<&C> {
		self.commands
			.iter()
			.find(|wrapper| key == wrapper.id && wrapper.enabled)
			.map(|wrapper| &wrapper.command)
	}

	#[must_use]
	pub fn find_alias(&self, key: &str) -> Option<&Alias> {
		self.aliases.iter().find(|a| key == a.alias)
	}

	// Internal implementation details /////////////////////////////////////////////

	/// Valid command IDs must contain at least two characters,
	/// and must begin with one ASCII letter or number.
	#[must_use]
	fn id_valid(id: &str) -> bool {
		id.chars().count() > 2 && id.chars().next().unwrap().is_ascii_alphanumeric()
	}
}

pub trait Command {
	type Output;

	/// The first argument given is always the command's ID,
	/// never aliased, with surrounding whitespace trimmed.
	fn call(&self, args: CommandArgs) -> Self::Output;
}

/// The command input, trimmed of whitespace between tokens and dissected for
/// easier consumption by commands themselves. Guaranteed to be in the end user's
/// given order.
#[derive(Debug)]
pub struct CommandArgs<'a>(Vec<&'a str>);

impl<'a> std::ops::Index<usize> for CommandArgs<'a> {
	type Output = &'a str;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl<'a> CommandArgs<'a> {
	#[must_use]
	pub fn new(args: Vec<&'a str>) -> Self {
		debug_assert!(
			!args.is_empty(),
			"`CommandArgs::new` can not take an empty collection."
		);

		Self(args)
	}

	/// If the end user submits `foo bar`, this will be `foo`.
	#[must_use]
	pub fn command_name(&self) -> &str {
		self.0[0]
	}

	/// The total number of arguments, including the name.
	#[must_use]
	#[allow(clippy::len_without_is_empty)] // Never empty
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Remember that 0 is always the command's name.
	#[must_use]
	pub fn get(&self, position: usize) -> Option<&str> {
		self.0.get(position).copied()
	}

	/// If the end user's argument list is broken by a `--` argument, this will
	/// contain every argument before that delimiter (excluding the delimiter itself).
	/// If there's no `--`, this will contain every argument that doesn't start
	/// with `-` or `--`.
	pub fn operands(&self) -> impl Iterator<Item = &&str> {
		let (iter, unfiltered) = match self.0[1..].iter().position(|string| *string == "--") {
			Some(pos) => (self.0[(pos + 1)..].iter(), true),
			None => (self.0[1..].iter(), false),
		};

		iter.filter(move |string| {
			unfiltered || !(string.starts_with('-') || string.starts_with("--"))
		})
	}

	#[must_use]
	pub fn operand_count(&self) -> usize {
		self.operands().count()
	}

	#[must_use]
	pub fn no_operands(&self) -> bool {
		self.operands().next().is_none()
	}

	/// If the end user's argument list is broken by a `--` argument, this will
	/// yield every argument after that delimiter (excluding the delimiter itself).
	/// If there's no `--`, this will yield every argument that doesn't start
	/// with `-` or `--`.
	pub fn options(&self) -> impl Iterator<Item = &&str> {
		let (iter, unfiltered) = match self.0[1..].iter().position(|string| *string == "--") {
			Some(pos) => (self.0[1..pos].iter(), true),
			None => (self.0[1..].iter(), false),
		};

		iter.filter(move |string| {
			unfiltered || (string.starts_with('-') || string.starts_with("--"))
		})
	}

	#[must_use]
	pub fn option_count(&self) -> usize {
		self.options().count()
	}

	#[must_use]
	pub fn no_options(&self) -> bool {
		self.options().next().is_none()
	}

	#[must_use]
	pub fn get_option(&self, option: &str) -> Option<&str> {
		debug_assert!(!option.is_empty());
		self.options().find(|o| **o == option).copied()
	}

	#[must_use]
	pub fn find_option<P: FnMut(&str) -> bool>(&self, mut predicate: P) -> Option<&str> {
		self.options().find(|o| predicate(o)).copied()
	}

	#[must_use]
	pub fn has_option(&self, option: &str) -> bool {
		debug_assert!(!option.is_empty());
		self.get_option(option).is_some()
	}

	/// See [`has_option`](CommandArgs::has_option).
	#[must_use]
	pub fn has_any_option(&self, options: &[&str]) -> bool {
		debug_assert!(!options.iter().any(|o| o.is_empty()));

		for opt in options {
			if self.options().any(|o| o == opt) {
				return true;
			}
		}

		false
	}

	#[must_use]
	pub fn get_operand(&self, operand: &str) -> Option<&str> {
		debug_assert!(!operand.is_empty());
		self.operands().find(|op| **op == operand).copied()
	}

	#[must_use]
	pub fn has_operand(&self, operand: &str) -> bool {
		self.get_operand(operand).is_some()
	}

	/// See [`has_operand`](CommandArgs::has_operand).
	#[must_use]
	pub fn has_any_operand(&self, operands: &[&str]) -> bool {
		debug_assert!(!operands.iter().any(|o| o.is_empty()));

		for opt in operands {
			if self.operands().any(|o| o == opt) {
				return true;
			}
		}

		false
	}

	/// Returns `true` if the option `--help` or `-h` was given. This should be
	/// treated as taking precedent over any possible other command behavior.
	#[must_use]
	pub fn help_requested(&self) -> bool {
		self.has_any_option(&["--help", "-h"])
	}

	/// The end user only provided the command's name and no operands or options.
	#[must_use]
	pub fn name_only(&self) -> bool {
		self.0.len() == 1
	}

	/// Provide a list of all the option prefixes the command expects; that is,
	/// if the option is intended to be given as `--foo=bar`, give `--foo`.
	/// If one of the options the end user supplies doesn't match any of the
	/// expected options, return it in a `Some`.
	/// If `None` is returned, the command has no invalid options.
	#[must_use]
	pub fn any_invalid_options(&self, valid: &[&str]) -> Option<&str> {
		self.options()
			.find(|&opt| !valid.iter().any(|v| opt.starts_with(v)))
			.copied()
	}

	/// Concatenates each argument - optionally excluding some head elements -
	/// with one space between each (no whitespace leading or trailing).
	#[must_use]
	pub fn concat(&self, start: usize) -> String {
		let cap = self.0.iter().map(|arg| arg.len()).sum();

		let mut ret = self.0[start..]
			.iter()
			.fold(String::with_capacity(cap), |mut acc, elem| {
				acc.push_str(elem);
				acc.push(' ');
				acc
			});

		ret.pop();

		ret
	}

	/// If given the argument `--foo=bar`, this will return `bar`. The returned
	/// string will be empty if given `--foo=` or `--foo`.
	#[must_use]
	pub fn option_value(text: &str) -> &str {
		debug_assert!(!text.is_empty() && text != "=");
		let mut split = text.split(|c| c == '=');
		split.next().unwrap();
		split.next().unwrap_or("")
	}
}

#[derive(Debug)]
pub struct Alias {
	pub alias: String,
	pub expanded: String,
}

#[derive(Debug)]
struct CommandWrapper<C: Command> {
	id: &'static str,
	enabled: bool,
	command: C,
}
