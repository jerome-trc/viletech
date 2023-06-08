//! [LANGUAGE](https://zdoom.org/wiki/LANGUAGE) is a language for defining
//! localized strings.

pub mod parse;
mod syn;

use smallvec::SmallVec;
pub use syn::Syn;

use rowan::{GreenNode, GreenToken};

use crate::util::GreenElement;

peg::parser! {
	pub grammar parser() for str {
		pub rule file() -> GreenNode
			= elems:(trivia() / key_val_pair() / locale_tag())* ![_]
		{
			GreenNode::new(Syn::Root.into(), elems)
		}

		pub rule key_val_pair() -> GreenElement
			= 	id:ident()
				t0:trivia()*
				eq:$("=")
				strings:string()+
				term:$(";")?
		{
			let mut elems: SmallVec<[_; 8]> = smallvec::smallvec![id.into()];

			for t in t0 {
				elems.push(t);
			}

			elems.push(GreenToken::new(Syn::Eq.into(), eq).into());

			for subvec in strings {
				for s in subvec {
					elems.push(s);
				}
			}

			if let Some(semicolon) = term {
				elems.push(GreenToken::new(Syn::Semicolon.into(), semicolon).into());
			}

			let mut node = GreenNode::new(Syn::KeyValuePair.into(), elems);
			node.into()
		}

		pub rule string() -> Vec<GreenElement>
			= t:trivia()* s:$("\"" (("\\" "\"") / ([^ '"']))* "\"")
		{
			let mut t = t;
			t.push(GreenToken::new(Syn::StringLit.into(), s).into());
			t
		}

		pub rule locale_tag() -> GreenElement
			= 	lb:$("[")
				t0:trivia()*
				id:ident()
				t1:trivia()+
				kw_def:$("default")
				t2:trivia()*
				rb:$("]")
		{
			let mut elems: SmallVec<[_; 8]> = smallvec::smallvec![
				GreenToken::new(Syn::BracketL.into(), lb).into()
			];

			for t in t0 {
				elems.push(t);
			}

			elems.push(id.into());

			for t in t1 {
				elems.push(t);
			}

			elems.push(GreenToken::new(Syn::KwDefault.into(), kw_def).into());

			for t in t2 {
				elems.push(t);
			}

			elems.push(GreenToken::new(Syn::BracketR.into(), rb).into());

			let mut node = GreenNode::new(Syn::LocaleTag.into(), elems);
			node.into()
		}

		rule ident() -> GreenToken
			= string:$(
				['a'..='z' | 'A'..='Z' | '_']
				['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*
			)
		{
			GreenToken::new(Syn::Ident.into(), string)
		}

		pub rule trivia() -> GreenElement
			= t:(wsp() / comment())
		{
			t.into()
		}

		pub rule wsp() -> GreenToken
			= string:$(
				['\0'..=' ']+
			)
		{
			GreenToken::new(Syn::Whitespace.into(), string)
		}

		pub rule comment() -> GreenToken
			= string:(
				$(
					"//" [^ '\n']* "\n"*
				) /
				$(
					"/*" ([^ '*'] / ("*" [^ '/']))* "*"+ "/"
				)
			)
		{
			GreenToken::new(Syn::Comment.into(), string)
		}
	}
}
