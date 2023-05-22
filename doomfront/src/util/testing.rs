/// Unit testing helper; checks that `elem` is a node with the given syntax tag.
pub fn assert_node<L>(
	elem: rowan::NodeOrToken<rowan::SyntaxNode<L>, rowan::SyntaxToken<L>>,
	kind: L::Kind,
) where
	L: rowan::Language,
{
	let node = elem.as_node();

	assert!(
		node.is_some(),
		"Element {elem:?} is unexpectedly not a node.",
	);

	let node = node.unwrap();

	assert_eq!(
		node.kind(),
		kind,
		"Expected token kind {kind:?}, found {:?}.",
		node.kind()
	);
}

/// Unit testing helper; checks that `elem` is a token with the given syntax tag and text.
pub fn assert_token<L>(
	elem: rowan::NodeOrToken<rowan::SyntaxNode<L>, rowan::SyntaxToken<L>>,
	kind: L::Kind,
	text: &'static str,
) where
	L: rowan::Language,
{
	let token = elem.as_token();

	assert!(
		token.is_some(),
		"Element {elem:?} is unexpectedly not a token.",
	);

	let token = token.unwrap();

	assert_eq!(
		token.kind(),
		kind,
		"Expected token kind {kind:?}, found {:?}.",
		token.kind()
	);

	assert_eq!(
		token.text(),
		text,
		"Expected token text {text}, found {}.",
		token.text()
	);
}

/// Unit testing helper; checks that [`rowan::WalkEvent::Enter`] events match
/// the node or token data provided in `seq`.
#[cfg(test)]
pub fn assert_sequence<L>(
	seq: &'static [(L::Kind, Option<&'static str>)],
	cursor: rowan::SyntaxNode<L>,
) where
	L: rowan::Language,
{
	use rowan::WalkEvent;

	let seq_count = seq.iter().clone().count();
	let elem_count = cursor.preorder_with_tokens().count();

	assert_eq!(
		seq_count,
		(elem_count / 2),
		"Expected a parsed sequence of {seq_count} elements, but found {elem_count}.",
	);

	let iter_s = seq.iter().copied();
	let iter_c = cursor
		.preorder_with_tokens()
		.filter_map(|event| match event {
			WalkEvent::Enter(enter) => Some(enter),
			WalkEvent::Leave(_) => None,
		});

	let iter_z = iter_s.zip(iter_c);

	for (i, ((kind, text), elem)) in iter_z.enumerate() {
		assert_eq!(
			elem.kind(),
			kind,
			"Expected element {i} to have kind {kind:?} but found {:?}.",
			elem.kind()
		);

		if let Some(text) = text {
			let token = elem.as_token();

			assert!(token.is_some());

			assert!(token.is_some(), "Element {i} is unexpectedly not a token.",);

			let token = token.unwrap();

			assert_eq!(
				token.text(),
				text,
				"Expected element {i} to have text {text} but found {}.",
				token.text()
			);
		} else {
			assert!(
				elem.as_node().is_some(),
				"Element {i} is unexpectedly not a node."
			);
		}
	}
}

/// For diagnosing combinators (or tests). Walks the node tree in preorder,
/// printing each node and token's display representation with indentation
/// according to the depth in the tree.
pub fn prettyprint<L>(cursor: rowan::SyntaxNode<L>)
where
	L: rowan::Language,
{
	let mut depth = 0;

	for event in cursor.preorder_with_tokens() {
		match event {
			rowan::WalkEvent::Enter(elem) => {
				let mut print = String::new();

				for _ in 0..depth {
					print.push_str("    ");
				}

				print.push_str(&format!("{elem:?}"));
				println!("{print}");

				depth += 1;
			}
			rowan::WalkEvent::Leave(_) => {
				depth -= 1;
			}
		}
	}
}
