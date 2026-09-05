//! pdfmanagement.sty — the PDF management bundle (combined l3pdfmeta/l3pdffile/
//! l3pdfdict code + tagpdf-base). Loads raw as-is; the no-op stubs of its
//! expl3 API in `latex_constructs_rust_only.rs` carry `DefinitionOrigin::Pool`,
//! so the bundle's `\cs_new` re-declarations are quiet (K1 provenance; the
//! pre-K1 stub retraction was retired in batch 56n). Guard:
//! `perfect_kernel_batch56::tagpdf_base_redeclares_the_stubbed_api_cleanly`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("pdfmanagement", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
