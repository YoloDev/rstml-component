use crate::{HtmlContent, HtmlFormatter};
use std::{fmt, io, sync::OnceLock};

pub use ammonia::Builder;

static DEFAULT_SANITIZER: OnceLock<Builder<'static>> = OnceLock::new();

fn default_sanitizer() -> &'static Builder<'static> {
	DEFAULT_SANITIZER.get_or_init(Builder::default)
}

enum Sanitizer {
	/// Use the default sanitizer.
	Default,

	/// Use the given sanitizer.
	Builder(&'static Builder<'static>),
}

impl Default for Sanitizer {
	fn default() -> Self {
		Self::Default
	}
}

impl AsRef<Builder<'static>> for Sanitizer {
	fn as_ref(&self) -> &Builder<'static> {
		match self {
			Self::Default => default_sanitizer(),
			Self::Builder(builder) => builder,
		}
	}
}

pub struct Sanitized<V>(V, Sanitizer);

impl<V> Sanitized<V>
where
	V: AsRef<[u8]>,
{
	pub fn new(value: V) -> Self {
		Self(value, Sanitizer::default())
	}

	pub fn new_with_sanitizer(value: V, sanitizer: &'static Builder<'static>) -> Self {
		Self(value, Sanitizer::Builder(sanitizer))
	}

	pub fn with_sanitizer(self, sanitizer: &'static Builder<'static>) -> Self {
		Self(self.0, Sanitizer::Builder(sanitizer))
	}
}

impl<V: AsRef<[u8]>> HtmlContent for Sanitized<V> {
	fn fmt(self, formatter: &mut HtmlFormatter) -> fmt::Result {
		let bytes = self.0.as_ref();
		let bytes = self
			.1
			.as_ref()
			.clean_from_reader(bytes)
			.map_err(|_| fmt::Error)?;
		bytes.write_to(IoWrite(formatter)).map_err(|_| fmt::Error)
	}
}

struct IoWrite<'a, 'b>(&'a mut HtmlFormatter<'b>);

impl<'a, 'b> io::Write for IoWrite<'a, 'b> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.0.write_bytes(buf);
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}
