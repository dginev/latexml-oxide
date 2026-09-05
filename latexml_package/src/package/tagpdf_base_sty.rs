//! tagpdf-base.sty — the tagging project's API surface (tagpdf-base.sty:30-60,
//! tagpdf-mc-generic.dtx). Loads raw as-is. The no-op stubs
//! `latex_constructs_rust_only.rs` provides for documents that call the
//! expl3/`\tag…` API WITHOUT tagpdf carry `DefinitionOrigin::Pool`, so the
//! package's `\cs_new_protected`/`\NewDocumentCommand` re-declarations are
//! quiet (K1 provenance; the pre-K1 retraction of the stubs was retired in
//! batch 56n). Guard:
//! `perfect_kernel_batch56::tagpdf_base_redeclares_the_stubbed_api_cleanly`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("tagpdf-base", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
