/// boundary-safe forward cursor over `&str` — the shared answer to the
/// byte-index scanners inherited from Perl's character-oriented string ops
pub mod char_cursor;
/// image file helpers — port of `LaTeXML::Util::Image`
pub mod image;
/// log recording and reporting interface
pub mod logger;
/// helper methods for file system paths
pub mod pathname;
/// "radix" may be a misnomer here. Primarily used to generate labels, or uniquifying suffixes to
/// make ID's
pub mod radix;
/// helpers for extracting structured data from replacement (and other) strings
pub mod text;
