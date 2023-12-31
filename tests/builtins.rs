use bytes::BytesMut;
use rstml_component::{write_html, For, HtmlFormatter};

macro_rules! assert_html_eq {
	($expected:expr, $($rest:tt)*) => {
		let mut buffer = BytesMut::new();
		let mut formatter = HtmlFormatter::new(&mut buffer);
		write_html!(formatter, $($rest)*).expect("failed to write html");

		let raw = buffer.as_ref();
		let as_str = std::str::from_utf8(raw).expect("invalid utf-8");
		assert_eq!(as_str, $expected);
	};
}

#[test]
fn for_iter() {
	let items = vec!["a", "b", "c"];

	assert_html_eq!(
		"<ul><li id=\"a\">a</li><li id=\"b\">b</li><li id=\"c\">c</li></ul>",
		<ul>
			<For items={items}>
				{|f, item| write_html!(f, <li id=item>{item}</li>)}
			</For>
		</ul>
	);
}
