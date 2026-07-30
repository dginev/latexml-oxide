//! Streaming XML substrate — the shared representation layer for fragmented
//! (bounded-memory) conversion.
//!
//! The eager pipeline holds one whole-document DOM from Build to the final
//! write, so peak RSS scales with document size (measured ~1.84 GB per MB of
//! source on the 131 MB witness — `docs/performance/STREAMING_CORE_DESIGN_2026-07-29.md`).
//! Fragmented mode bounds peak RSS by *fragment* size instead: closed subtrees
//! are serialized to disk ("spilled") during Build and re-materialized one at a
//! time for the later phases. This module is the substrate both halves stand
//! on; it knows nothing about TeX or the schema:
//!
//! * [`SegmentStore`](crate::sxml::SegmentStore) — the on-disk spill area:
//!   numbered segment files under a directory beside the destination (same
//!   volume, so [`crate::watchdog::available_disk_bytes`] headroom checks
//!   measure the right filesystem). Pass 1 writes parseable,
//!   `_lxfragment`-wrapped segments; pass 2 replaces each with its processed,
//!   splice-ready output text.
//! * [`FragmentIndex`](crate::sxml::FragmentIndex) — the document-global facts
//!   that must survive a spill as plain strings (never as node handles into a
//!   freed DOM — the historical finalize-SIGSEGV class): spilled `xml:id`s and
//!   which segment holds them, `label → id`, RDFa prefix declarations.
//! * [`FragmentReader`](crate::sxml::FragmentReader) — a streaming iterator
//!   over a segment (or any XML file): materializes ONE top-level subtree at a
//!   time as an owned, mutable [`libxml::tree::Document`] via
//!   `xmlTextReaderExpand`, while the rest of the file stays unparsed.
//!
//! Activation policy, spill-eligibility, and the placeholder-splice assembly
//! live with their owners (`stomach`/`document`/`core_interface` for the core
//! half, `latexml_post::stream_split` for the post half); this module only
//! moves bytes and facts.

mod fragment_index;
mod fragment_reader;
mod segment_store;

pub use fragment_index::FragmentIndex;
pub use fragment_reader::FragmentReader;
pub use segment_store::{SegmentId, SegmentMeta, SegmentStore};
