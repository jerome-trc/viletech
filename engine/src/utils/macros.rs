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

#[macro_export]
macro_rules! replace_expr {
	($_t:tt $sub:expr) => {
		$sub
	};
}

/// Convenience macro for defining a newtype (single-field tuple struct).
/// Implementations are provided for [`std::ops::Deref`] and [`std::ops::DerefMut`].
///
/// Usage examples:
/// ```
/// newtype!(pub struct NewType(i32));
/// newtype!(
///     /// Here's an informative comment.
///     pub struct NewType2(Vec<usize>)
/// );
/// ```
#[macro_export]
macro_rules! newtype {
	(
		$(#[$outer:meta])*
		$ownvis:vis struct $name:ident($innervis:vis $type:ty)
	) => {
		$(#[$outer])*
		$ownvis struct $name($innervis $type);

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

/// Serves a similar role to [`newtype`].
/// When given type `T`, creates a newtype wrapping `&mut T`.
#[macro_export]
macro_rules! newtype_mutref {
	(
		$(#[$outer:meta])*
		$ownvis:vis struct $name:ident($innervis:vis $type:ty)
	) => {
		$(#[$outer])*
		$ownvis struct $name<'inner>($innervis &'inner mut $type);

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
