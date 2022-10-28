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

/// Convenience macro for defining a newtype (single-field tuple struct). Provide
/// a type to wrap, a comma, optionally a visibility qualifier, and a name.
/// Implementations are provided for [`std::ops::Deref`] and [`std::ops::DerefMut`].
#[macro_export]
macro_rules! newtype {
	(
		$(#[$outer:meta])*
		$visqual:vis struct $name:ident($type:ty)
	) => {
		$(#[$outer])*
		$visqual struct $name($type);

		impl std::ops::Deref for $name {
			type Target = $type;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl std::ops::DerefMut for $name {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}
	};
}

/// Serves a similar role to [`newtype`]. Provide a type to wrap, a comma,
/// optionally a visibility qualifier, and a name.
/// When given type `T`, creates a newtype wrapping `&mut T`.
#[macro_export]
macro_rules! newtype_mutref {
	(
		$(#[$outer:meta])*
		$visqual:vis struct $name:ident($type:ty)
	) => {
		$(#[$outer])*
		$visqual struct $name<'inner>(&'inner mut $type);

		impl<'inner> std::ops::Deref for $name<'_> {
			type Target = $type;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl<'inner> std::ops::DerefMut for $name<'_> {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}
	}
}
