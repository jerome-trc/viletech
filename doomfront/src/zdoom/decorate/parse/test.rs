use rowan::{ast::AstNode, SyntaxNode};

use crate::{
	util::{builder::GreenCacheNoop, testing::*},
	zdoom::decorate::{
		ast::{self},
		parse, Syn,
	},
};
