// only enables the `doc_cfg` feature when
// the `docsrs` configuration attribute is defined
#![cfg_attr(docsrs, feature(doc_cfg))]

mod builtins;
mod component;
mod fmt;

#[cfg(feature = "sanitize")]
mod sanitize;

pub use builtins::For;
pub use component::HtmlComponent;
pub use fmt::{
	HtmlAttributeFormatter, HtmlAttributeValue, HtmlAttributes, HtmlAttributesFormatter, HtmlContent,
	HtmlFormatter, RawText,
};
pub use rstml_component_macro::{component, html, write_html, HtmlComponent};

#[cfg(feature = "sanitize")]
#[cfg_attr(docsrs, doc(cfg(feature = "sanitize")))]
pub use sanitize::{SanitizeConfig, Sanitized};
