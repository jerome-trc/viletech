use crate::{parser::Parser, zdoom::Token};

use super::Syn;

/// Builds a [`Syn::Root`] node.
pub fn file(p: &mut Parser<Syn>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		if p.at(Token::KwMap) {
			block(p, Syn::MapDef, Syn::KwMap, post_kw_mapdef);
		} else if p.at_str_nc(Token::Ident, "automap") {
			block(p, Syn::AutomapDef, Syn::KwAutomap, |_| {});
		} else if p.at_str_nc(Token::Ident, "automap_overlay") {
			block(p, Syn::AutomapDef, Syn::KwAutomapOverlay, |_| {});
		} else if p.at_str_nc(Token::Ident, "clearepisodes") {
			let node = p.open();
			p.advance(Syn::KwClearEpisodes);
			p.close(node, Syn::ClearEpisodes);
		} else if p.at_str_nc(Token::Ident, "cluster") {
			block(p, Syn::ClusterDef, Syn::KwCluster, post_kw_intlit);
		} else if p.at_str_nc(Token::Ident, "conversationids") {
			block(p, Syn::ConversationDef, Syn::KwConversationIds, |_| {});
		} else if p.at_str_nc(Token::Ident, "damagetype") {
			block(p, Syn::DamageTypeDef, Syn::KwDamageType, post_kw_ident);
		} else if p.at_str_nc(Token::Ident, "defaultmap") {
			block(p, Syn::DefaultMapDef, Syn::KwDefaultMap, |_| {});
		} else if p.at_str_nc(Token::Ident, "doomednums") {
			block(p, Syn::KwDoomEdNums, Syn::KwDoomEdNums, |_| {});
		} else if p.at_str_nc(Token::Ident, "episode") {
			block(p, Syn::EpisodeDef, Syn::KwEpisode, post_kw_episodedef);
		} else if p.at_str_nc(Token::Ident, "gameinfo") {
			block(p, Syn::GameInfoDef, Syn::KwGameInfo, |_| {});
		} else if p.at_str_nc(Token::Ident, "gamedefaults") {
			block(p, Syn::GameDefaults, Syn::KwGameDefaults, |_| {});
		} else if p.at_str_nc(Token::Ident, "include") {
			include_directive(p);
		} else if p.at_str_nc(Token::Ident, "intermission") {
			block(p, Syn::IntermissionDef, Syn::KwIntermission, post_kw_ident);
		} else if p.at_str_nc(Token::Ident, "skill") {
			block(p, Syn::SkillDef, Syn::KwSkill, post_kw_ident);
		} else if p.at_str_nc(Token::Ident, "spawnnums") {
			block(p, Syn::SpawnNumDefs, Syn::KwSpawnNums, |_| {});
		} else {
			p.advance_with_error(
				Syn::from(p.nth(0)),
				&[&[
					"`automap`",
					"`automap_overlay`",
					"`clearepisodes`",
					"`cluster`",
					"`conversationids`",
					"`damagetype`",
					"`defaultmap`",
					"`doomednums`",
					"`episode`",
					"`gameinfo`",
					"`include`",
					"`intermission`",
					"`map`",
					"`skill`",
					"`spawnnums`",
				]],
			);
		}
	}

	p.close(root, Syn::Root);
}

fn include_directive(p: &mut Parser<Syn>) {
	let node = p.open();
	p.advance(Syn::KwInclude);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syn::StringLit, &[&["a string"]]);
	p.close(node, Syn::IncludeDirective);
}

fn post_kw_episodedef(p: &mut Parser<Syn>) {
	post_kw_ident(p);
	trivia_0plus(p);

	if p.at_str_nc(Token::Ident, "teaser") {
		p.advance(Syn::KwTeaser);
		trivia_0plus(p);
		p.expect(Token::Ident, Syn::Ident, &[&["an identifier"]]);
	}
}

fn post_kw_mapdef(p: &mut Parser<Syn>) {
	post_kw_ident(p);
	trivia_0plus(p);

	if p.at_str_nc(Token::Ident, "lookup") {
		p.advance(Syn::KwLookup);
		trivia_0plus(p);
		p.expect(Token::StringLit, Syn::StringLit, &[&["a string"]]);
	}
}

// Common //////////////////////////////////////////////////////////////////////

fn block(p: &mut Parser<Syn>, node: Syn, kw: Syn, post_kw: fn(&mut Parser<Syn>)) {
	let marker = p.open();
	p.advance(kw);
	trivia_0plus(p);
	post_kw(p);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		let lookahead = p.find(1, |token| !token.is_trivia());

		if lookahead == Token::BraceL {
			sub_block(p);
		} else {
			property(p);
		}

		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
	p.close(marker, node);
}

fn post_kw_ident(p: &mut Parser<Syn>) {
	p.expect(Token::Ident, Syn::Ident, &[&["an identifier"]]);
}

fn post_kw_intlit(p: &mut Parser<Syn>) {
	p.expect(Token::IntLit, Syn::IntLit, &[&["an integer"]]);
}

fn property(p: &mut Parser<Syn>) {
	let node = p.open();

	if p.nth(0).is_keyword() {
		p.advance(Syn::Ident);
	} else {
		p.expect_any(
			&[(Token::Ident, Syn::Ident), (Token::IntLit, Syn::IntLit)],
			&[&["an integer", "a string"]],
		);
	}

	if p.find(0, |token| !token.is_trivia()) == Token::Eq {
		trivia_0plus(p);
		p.advance(Syn::Eq);
		trivia_0plus(p);
		property_values(p);
	}

	p.close(node, Syn::Property);
}

fn property_values(p: &mut Parser<Syn>) {
	fn value(p: &mut Parser<Syn>) {
		let node = p.open();
		let token = p.nth(0);

		if token.is_keyword() && !matches!(token, Token::KwFalse | Token::KwTrue) {
			p.advance(Syn::Ident);
		} else if token == Token::Minus {
			p.advance(Syn::Minus);
			p.expect_any(
				&[
					(Token::IntLit, Syn::IntLit),
					(Token::FloatLit, Syn::FloatLit),
				],
				&[&["an integer", "a floating-point number"]],
			);
		} else {
			p.expect_any(
				&[
					(Token::Ident, Syn::Ident),
					(Token::StringLit, Syn::StringLit),
					(Token::IntLit, Syn::IntLit),
					(Token::FloatLit, Syn::FloatLit),
					(Token::KwFalse, Syn::KwFalse),
					(Token::KwTrue, Syn::KwTrue),
					(Token::Plus, Syn::Plus),
				],
				&[&[
					"an identifier",
					"a string",
					"an integer",
					"a floating-point number",
					"`false`",
					"`true`",
				]],
			);
		}

		p.close(node, Syn::Value);
	}

	value(p);

	while !p.eof() && p.find(0, |token| !token.is_trivia()) == Token::Comma {
		trivia_0plus(p);
		p.advance(Syn::Comma);
		trivia_0plus(p);
		value(p);
	}
}

fn sub_block(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::Ident);
	block(p, Syn::SubBlock, Syn::Ident, |_| {});
}

fn trivia(p: &mut Parser<Syn>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syn::Whitespace),
		(Token::Comment, Syn::Comment),
		(Token::DocComment, Syn::Comment),
		(Token::RegionStart, Syn::RegionStart),
		(Token::RegionEnd, Syn::RegionEnd),
	])
}

fn trivia_0plus(p: &mut Parser<Syn>) {
	while trivia(p) {}
}

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{self, mapinfo::ParseTree},
	};

	use super::*;

	#[test]
	fn smoke_commented_property() {
		const SOURCE: &str = r"
DoomEdNums
{
	9875 = None	// ZDRay light probe
}
";

		let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::lex::Context::NON_ZSCRIPT);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}

	#[test]
	#[ignore]
	fn with_sample_data() {
		let (_, sample) = match read_sample_data("DOOMFRONT_MAPINFO_SAMPLE") {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Skipping MAPINFO sample data-based unit test. Reason: {err}");
				return;
			}
		};

		let ptree: ParseTree = crate::parse(&sample, file, zdoom::lex::Context::NON_ZSCRIPT);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}
