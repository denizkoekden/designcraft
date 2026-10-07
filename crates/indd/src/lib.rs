//! InDesign document (`.indd`) import: [`to_idml`] converts the binary document into an IDML
//! package that `designcraft-idml` imports.
//!
//! Implemented clean-room from the structure of documents the user created themselves; no Adobe
//! code, SDK or documentation of the binary format was used. Format notes: `docs/indd.md`.
//!
//! ## Layers of the reader
//! - [`container`]: 4 KiB pages, master pages, the logical page map, the UID B-trees and object
//!   records (split records and multi-page objects).
//! - [`objects`]: an object is a list of `(implementation id, data)` blocks.
//! - [`model`]: spreads, pages, page items, swatches, styles and stories.
//! - [`writer`]: the IDML package.
//!
//! ## Supported subset
//! Spreads and pages, layers (order, visibility of items), rectangles/polygons/text frames/groups
//! with path geometry, fill and stroke colours, process/spot swatches (CMYK, RGB, Lab), paragraph
//! and character styles with overrides (font, style, size, leading, colour, justification),
//! stories, and placed graphics (embedded PDF, the TIFF preview of embedded EPS, or InDesign's
//! proxy image when the link was not embedded).
//!
//! ## TODO
//! - Text frame insets, first-baseline and vertical justification options, threading.
//! - Tracking/kerning, master page items, tables, effects, text wrap, corner options.
//! - Big-endian (PowerPC-era) documents.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod attrs;
mod bytes;
pub mod container;
pub mod model;
pub mod objects;
mod text;
mod tiff;
pub mod writer;

#[cfg(any(test, feature = "synthetic"))]
#[doc(hidden)]
pub mod synthetic;
#[cfg(test)]
mod tests;

/// Errors while reading an InDesign document.
#[derive(Debug, thiserror::Error)]
pub enum InddError {
    #[error("not an InDesign document")]
    NotIndd,
    #[error("unsupported InDesign document: {0}")]
    Unsupported(&'static str),
    #[error("damaged InDesign document: {0}")]
    Damaged(String),
    #[error("writing IDML failed: {0}")]
    Write(String),
}

pub type Result<T> = std::result::Result<T, InddError>;

/// True when `bytes` start like an InDesign document (master page GUID + `DOCUMENT`).
pub fn is_indd(bytes: &[u8]) -> bool {
    bytes.get(..16) == Some(&container::GUID[..]) && bytes.get(16..24) == Some(&b"DOCUMENT"[..])
}

/// Converts an InDesign document into an IDML package (ZIP bytes).
pub fn to_idml(bytes: &[u8]) -> Result<Vec<u8>> {
    to_idml_named(bytes, "document.indd")
}

/// Like [`to_idml`], with the document name written into `designmap.xml`.
pub fn to_idml_named(bytes: &[u8], name: &str) -> Result<Vec<u8>> {
    let c = container::Container::parse(bytes)?;
    let mut doc = model::Builder::new(&c).build()?;
    doc.name = name.to_string();
    writer::write(&doc)
}
