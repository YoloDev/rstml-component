use crate::{HtmlContent, HtmlFormatter};
use std::{fmt, io, sync::OnceLock};

pub use ammonia::Builder;

static DEFAULT_SANITIZER: OnceLock<Builder<'static>> = OnceLock::new();

fn default_sanitizer() -> &'static Builder<'static> {
	DEFAULT_SANITIZER.get_or_init(Builder::default)
}

#[derive(Clone, Copy)]
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

/// A wrapper struct for adding potentially unsanitized HTML content that will be sanitized before rendering.
///
/// The `Sanitized` struct allows you to include HTML content that might be potentially unsafe,
/// and ensures that it's properly sanitized before being rendered within your HTML components.
/// This is particularly useful when you want to include user-generated content or any content
/// that might contain unsafe HTML elements or scripts.
#[derive(Clone)]
pub struct Sanitized<V>(V, Sanitizer);

impl<V: fmt::Debug> fmt::Debug for Sanitized<V> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Sanitized").field(&self.0).finish()
	}
}

impl<V> Sanitized<V>
where
	V: AsRef<[u8]>,
{
	/// Creates a new `Sanitized` instance with the HTML content to be sanitized before rendering.
	///
	/// # Arguments
	///
	/// - `value`: The HTML content to be sanitized as a byte slice.
	///
	/// # Returns
	///
	/// A `Sanitized` instance wrapping the HTML content that will be sanitized before rendering.
	pub fn new(value: V) -> Self {
		Self(value, Sanitizer::default())
	}

	/// Creates a new `Sanitized` instance with the HTML content and a custom sanitizer.
	///
	/// # Arguments
	///
	/// - `value`: The HTML content to be sanitized as a byte slice.
	/// - `sanitizer`: The custom sanitizer to be used for this specific instance.
	///
	/// # Returns
	///
	/// A `Sanitized` instance wrapping the HTML content that will be sanitized using the specified sanitizer.
	pub fn new_with_sanitizer(value: V, sanitizer: &'static Builder<'static>) -> Self {
		Self(value, Sanitizer::Builder(sanitizer))
	}

	/// Sets a custom sanitizer for this `Sanitized` instance.
	///
	/// This method allows you to override the default sanitizer for a specific `Sanitized` instance.
	///
	/// # Arguments
	///
	/// - `sanitizer`: The custom sanitizer to be used for this specific instance.
	///
	/// # Returns
	///
	/// A new `Sanitized` instance with the specified sanitizer.
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
