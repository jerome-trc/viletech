//! Abstraction for text-based UI, used by the headless server and client's console.

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::ops::Deref;

use crate::lazy_regex;

/// `::0` is the alias, `::1` is what it expands into.
pub type Alias = (String, String);

/// Wraps a vec of string slices with convenience functions for command functions.
pub struct CommandArgs<'a>(pub Vec<&'a str>);

impl<'a> Deref for CommandArgs<'a> {
	type Target = Vec<&'a str>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'a> CommandArgs<'a> {
	/// If one of the arguments given by the user is `--`, every other argument
	/// after it (not including the `--` itself) is yielded by the returned iterator.
	/// If that token isn't found, every argument except the command ID is returned.
	/// Either way, only values beginning with `-` or `--` are returned.
	pub fn operands(&self) -> impl Iterator<Item = &&str> {
		match self[1..].iter().position(|string| *string == "--") {
			Some(pos) => self[(pos + 1)..].iter(),
			None => self[1..].iter(),
		}
		.filter(|string| !(string.starts_with('-') || string.starts_with("--")))
	}

	/// If one of the arguments given by the user is `--`, every other argument
	/// before it (not including the `--` itself, and not including the command's
	/// ID), is yielded by the returned iterator.
	/// If that token isn't found, every argument except the command ID is returned.
	/// Either way, only values not beginning with `-` or `--` are returned.
	pub fn options(&self) -> impl Iterator<Item = &&str> {
		match self[1..].iter().position(|string| *string == "--") {
			Some(pos) => self[1..pos].iter(),
			None => self[1..].iter(),
		}
		.filter(|string| string.starts_with('-') || string.starts_with("--"))
	}

	#[must_use]
	pub fn has_option(&self, option: &str) -> bool {
		debug_assert!(!option.is_empty());

		self.options().any(|o| *o == option)
	}

	/// See [`CommandArgs::has_option`].
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

	/// Convenience function which concatenates string slices
	/// with one space between each (no whitespace leading or trailing).
	pub fn concat(args: &[&str]) -> String {
		// TODO: Would be nice if this was a method taking `R: RangeBounds`
		// but I think that needs a std. function to become stable
		let cap = args.iter().map(|arg| arg.len()).sum();

		let mut ret = args
			.iter()
			.fold(String::with_capacity(cap), |mut acc, elem| {
				acc.push_str(elem);
				acc.push(' ');
				acc
			});

		ret.pop();

		ret
	}

	/// Returns `true` if the argument vector contains `--help` or `-h`.
	pub fn help(&self) -> bool {
		self.has_any_option(&["--help", "-h"])
	}

	pub fn id_only(&self) -> bool {
		self.len() <= 1
	}
}

pub trait Command {
	type Output;

	/// The first argument given is always the command's ID,
	/// never aliased, with surrounding whitespace trimmed.
	fn call(&self, args: CommandArgs) -> Self::Output;
}

struct CommandWrapper<C: Command> {
	id: &'static str,
	enabled: bool,
	command: C,
}

/// This combines storage for text-based commands and aliases with a parser
/// for matching against those commands, allowing both the client's console
/// and headless server to seamlessly use the same code and UI.
pub struct Terminal<C: Command> {
	commands: Vec<CommandWrapper<C>>,
	command_not_found: fn(&str),
	aliases: Vec<Alias>,
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

		// "Recursive" alias expansion, no more than 8 levels deep

		for _ in 0..8 {
			let mut s = String::default();

			for token in string.split_whitespace() {
				if let Some(alias) = self.find_alias(token) {
					s.push_str(&alias.1);
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
					ret.push(cmd.call(CommandArgs(args)));
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

		match self.aliases.iter().position(|a| a.0 == alias) {
			Some(pos) => {
				self.aliases[pos].0 = alias;
				self.aliases[pos].1 = string;
			}
			None => {
				self.aliases.push((alias, string));
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
		for wrapper in &self.commands {
			if key != wrapper.id || !wrapper.enabled {
				continue;
			}

			return Some(&wrapper.command);
		}

		None
	}

	#[must_use]
	pub fn find_alias(&self, key: &str) -> Option<&Alias> {
		for alias in &self.aliases {
			if key != alias.0 {
				continue;
			}

			return Some(alias);
		}

		None
	}
}

// Internal implementation details.
impl<C: Command> Terminal<C> {
	/// Valid command IDs must contain at least two characters,
	/// and must begin with one ASCII letter or number.
	#[must_use]
	fn id_valid(id: &str) -> bool {
		id.chars().count() > 2 && id.chars().next().unwrap().is_ascii_alphanumeric()
	}
}
