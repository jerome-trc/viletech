use crate::{parser::Parser, zdoom::Token};

use super::Syntax;

/// Builds a [`Syntax::Root`] node.
pub fn file(p: &mut Parser<Syntax>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		if p.at(Token::KwMap) {
			block(p, Syntax::MapDef, Syntax::KwMap, post_kw_mapdef);
		} else if p.at_str_nc(Token::Ident, "automap") {
			block(p, Syntax::AutomapDef, Syntax::KwAutomap, |_| {});
		} else if p.at_str_nc(Token::Ident, "automap_overlay") {
			block(p, Syntax::AutomapDef, Syntax::KwAutomapOverlay, |_| {});
		} else if p.at_str_nc(Token::Ident, "clearepisodes") {
			let node = p.open();
			p.advance(Syntax::KwClearEpisodes);
			p.close(node, Syntax::ClearEpisodes);
		} else if p.at_str_nc(Token::Ident, "cluster") {
			block(p, Syntax::ClusterDef, Syntax::KwCluster, post_kw_intlit);
		} else if p.at_str_nc(Token::Ident, "conversationids") {
			block(
				p,
				Syntax::ConversationDef,
				Syntax::KwConversationIds,
				|_| {},
			);
		} else if p.at_str_nc(Token::Ident, "damagetype") {
			block(
				p,
				Syntax::DamageTypeDef,
				Syntax::KwDamageType,
				post_kw_ident,
			);
		} else if p.at_str_nc(Token::Ident, "defaultmap") {
			block(p, Syntax::DefaultMapDef, Syntax::KwDefaultMap, |_| {});
		} else if p.at_str_nc(Token::Ident, "doomednums") {
			block(p, Syntax::KwDoomEdNums, Syntax::KwDoomEdNums, |_| {});
		} else if p.at_str_nc(Token::Ident, "episode") {
			block(p, Syntax::EpisodeDef, Syntax::KwEpisode, post_kw_episodedef);
		} else if p.at_str_nc(Token::Ident, "gameinfo") {
			block(p, Syntax::GameInfoDef, Syntax::KwGameInfo, |_| {});
		} else if p.at_str_nc(Token::Ident, "gamedefaults") {
			block(p, Syntax::GameDefaults, Syntax::KwGameDefaults, |_| {});
		} else if p.at_str_nc(Token::Ident, "include") {
			include_directive(p);
		} else if p.at_str_nc(Token::Ident, "intermission") {
			block(
				p,
				Syntax::IntermissionDef,
				Syntax::KwIntermission,
				post_kw_ident,
			);
		} else if p.at_str_nc(Token::Ident, "skill") {
			block(p, Syntax::SkillDef, Syntax::KwSkill, post_kw_ident);
		} else if p.at_str_nc(Token::Ident, "spawnnums") {
			block(p, Syntax::SpawnNumDefs, Syntax::KwSpawnNums, |_| {});
		} else {
			p.advance_with_error(
				Syntax::from(p.nth(0)),
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

	p.close(root, Syntax::Root);
}

fn include_directive(p: &mut Parser<Syntax>) {
	let node = p.open();
	p.advance(Syntax::KwInclude);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syntax::StringLit, &[&["a string"]]);
	p.close(node, Syntax::IncludeDirective);
}

fn post_kw_episodedef(p: &mut Parser<Syntax>) {
	post_kw_ident(p);
	trivia_0plus(p);

	if p.at_str_nc(Token::Ident, "teaser") {
		p.advance(Syntax::KwTeaser);
		trivia_0plus(p);
		p.expect(Token::Ident, Syntax::Ident, &[&["an identifier"]]);
	}
}

fn post_kw_mapdef(p: &mut Parser<Syntax>) {
	post_kw_ident(p);
	trivia_0plus(p);

	if p.at_str_nc(Token::Ident, "lookup") {
		p.advance(Syntax::KwLookup);
		trivia_0plus(p);
		p.expect(Token::StringLit, Syntax::StringLit, &[&["a string"]]);
	}
}

// Common //////////////////////////////////////////////////////////////////////

fn block(p: &mut Parser<Syntax>, node: Syntax, kw: Syntax, post_kw: fn(&mut Parser<Syntax>)) {
	let marker = p.open();
	p.advance(kw);
	trivia_0plus(p);
	post_kw(p);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
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

	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(marker, node);
}

fn post_kw_ident(p: &mut Parser<Syntax>) {
	p.expect(Token::Ident, Syntax::Ident, &[&["an identifier"]]);
}

fn post_kw_intlit(p: &mut Parser<Syntax>) {
	p.expect(Token::IntLit, Syntax::IntLit, &[&["an integer"]]);
}

fn property(p: &mut Parser<Syntax>) {
	let node = p.open();

	if p.nth(0).is_keyword() {
		p.advance(Syntax::Ident);
	} else {
		p.expect_any(
			&[
				(Token::Ident, Syntax::Ident),
				(Token::IntLit, Syntax::IntLit),
			],
			&[&["an integer", "a string"]],
		);
	}

	if p.find(0, |token| !token.is_trivia()) == Token::Eq {
		trivia_0plus(p);
		p.advance(Syntax::Eq);
		trivia_0plus(p);
		property_values(p);
	}

	p.close(node, Syntax::Property);
}

fn property_values(p: &mut Parser<Syntax>) {
	fn value(p: &mut Parser<Syntax>) {
		let node = p.open();
		let token = p.nth(0);

		if token.is_keyword() && !matches!(token, Token::KwFalse | Token::KwTrue) {
			p.advance(Syntax::Ident);
		} else if token == Token::Minus {
			p.advance(Syntax::Minus);
			p.expect_any(
				&[
					(Token::IntLit, Syntax::IntLit),
					(Token::FloatLit, Syntax::FloatLit),
				],
				&[&["an integer", "a floating-point number"]],
			);
		} else {
			p.expect_any(
				&[
					(Token::Ident, Syntax::Ident),
					(Token::StringLit, Syntax::StringLit),
					(Token::IntLit, Syntax::IntLit),
					(Token::FloatLit, Syntax::FloatLit),
					(Token::KwFalse, Syntax::KwFalse),
					(Token::KwTrue, Syntax::KwTrue),
					(Token::Plus, Syntax::Plus),
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

		p.close(node, Syntax::Value);
	}

	value(p);

	while !p.eof() && p.find(0, |token| !token.is_trivia()) == Token::Comma {
		trivia_0plus(p);
		p.advance(Syntax::Comma);
		trivia_0plus(p);
		value(p);
	}
}

fn sub_block(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::Ident);
	block(p, Syntax::SubBlock, Syntax::Ident, |_| {});
}

fn trivia(p: &mut Parser<Syntax>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syntax::Whitespace),
		(Token::Comment, Syntax::Comment),
		(Token::DocComment, Syntax::Comment),
		(Token::RegionStart, Syntax::RegionStart),
		(Token::RegionEnd, Syntax::RegionEnd),
	])
}

fn trivia_0plus(p: &mut Parser<Syntax>) {
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
		const SAMPLE: &str = r"
DoomEdNums
{
	9875 = None	// ZDRay light probe
}
";

		let ptree: ParseTree = crate::parse(SAMPLE, file, zdoom::lex::Context::NON_ZSCRIPT);
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
