use bytes::BytesMut;
use rstml_component::{write_html, HtmlFormatter};

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
fn empty() {
	assert_html_eq!("",);
}

#[test]
fn empty_div() {
	assert_html_eq!("<div></div>", <div />);
	assert_html_eq!("<div></div>", <div></div>);
}

#[test]
fn empty_div_with_attributes() {
	assert_html_eq!("<div class=\"test\"></div>", <div class="test" />);
}

#[test]
fn doctype() {
	assert_html_eq!("<!DOCTYPE html>", <!DOCTYPE html>);
}

#[test]
fn fragment() {
	assert_html_eq!("<div></div><div></div>", <><div></div><div /></>);
}

#[test]
fn children() {
	assert_html_eq!("<div>test</div>", <div> test </div>);
	assert_html_eq!("<div>test</div>", <div>"test"</div>);
}

#[test]
fn kitchen_sink() {
	let title = "Hello";
	let attribute = "world";
	let children = "I have <html> in me";

	let expected = r#"
		<!DOCTYPE html>
		<html>
			<head>
				<title>Hello</title>
			</head>
			<body>
				<!--comment-->
				<div hello="world"></div>
				<div hello="world"></div>
				<div>1</div>
				<div>Hello world with spaces</div>
				<div>3</div>
				<div>world</div>
				<hr />
				<hr />
				<main>
					I have &lt;html&gt; in me
				</main>
			</body>
		</html>
	"#
	.trim()
	.lines()
	.map(|l| l.trim())
	.collect::<String>();

	assert_html_eq!(
		expected,
		<!DOCTYPE html>
		<html>
			<head>
				<title>{title}</title>
			</head>
			<body>
				<!-- "comment" -->
				<div hello=attribute.clone() />
				<div hello=attribute.clone() />
				<>
					<div>"1"</div>
					<div> Hello  world with spaces </div>
					<div>"3"</div>
					<div>{attribute}</div>
					// <div {"some-attribute-from-rust-block"}/>
					<hr><hr />
				</>

				<main>
					{children}
				</main>
			</body>
		</html>
	);
}
