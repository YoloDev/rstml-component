mod builtins;
mod component;
mod fmt;

#[cfg(feature = "sanitize")]
mod sanitize;

pub use builtins::For;
pub use component::HtmlComponent;
pub use fmt::{HtmlAttributeFormatter, HtmlAttributeValue, HtmlContent, HtmlFormatter, RawText};
pub use rstml_component_macro::{html, move_html, write_html, HtmlComponent};

#[cfg(feature = "sanitize")]
pub use sanitize::{SanitizeConfig, Sanitized};
