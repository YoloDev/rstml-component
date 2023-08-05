use bytes::BytesMut;
use rstml_component::{write_html, HtmlAttributeValue, HtmlComponent, HtmlContent, HtmlFormatter};

#[derive(HtmlComponent)]
struct NavBar;

impl HtmlContent for NavBar {
	fn fmt(self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
		write_html!(formatter,
			<nav>
				<ul>
					<li><a href="/">Home</a></li>
					<li><a href="/about">About</a></li>
				</ul>
			</nav>
		)
	}
}

#[derive(HtmlComponent)]
pub(crate) struct Template<T, A, C>
where
	T: Into<String>,
	A: HtmlAttributeValue + HtmlContent + Clone,
	C: HtmlContent,
{
	title: T,
	attribute: A,
	children: C,
}

impl<T, A, C> HtmlContent for Template<T, A, C>
where
	T: Into<String>,
	A: HtmlAttributeValue + HtmlContent + Clone,
	C: HtmlContent,
{
	fn fmt(self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
		write_html!(formatter,
			<!DOCTYPE html>
			<html>
				<head>
					<title>{self.title.into()}</title>
				</head>
				<body>
					<!-- "comment" -->
					<div hello=self.attribute.clone() />
					<div hello=self.attribute.clone() />
					<>
						<div>"1"</div>
						<div> Hello  world with spaces </div>
						<div>"3"</div>
						<div>{self.attribute}</div>
						<hr><hr />
					</>

					<main>
						<NavBar />
						{self.children}
					</main>
				</body>
			</html>
		)
	}
}

#[derive(HtmlComponent)]
struct Page<T>
where
	T: Into<String>,
{
	title: T,
	heading: String,
}

impl<T> HtmlContent for Page<T>
where
	T: Into<String>,
{
	fn fmt(self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
		write_html!(formatter,
			<Template title=self.title attribute="world">
				<h1>{self.heading}</h1>
				<p>"This is a test"</p>
			</Template>
		)
	}
}

fn test_output() {
	let page = Page {
		title: "Hello",
		heading: "Hello world".into(),
	};

	let output = page
		.into_string()
		.expect("formatting works and produces valid utf-8");

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
				<nav>
					<ul>
						<li><a href="/">Home</a></li>
						<li><a href="/about">About</a></li>
					</ul>
				</nav>
				<h1>Hello world</h1>
				<p>This is a test</p>
			</main>
		</body>
	</html>
	"#
	.trim()
	.lines()
	.map(|l| l.trim())
	.collect::<String>();

	assert_eq!(output, expected);
}
