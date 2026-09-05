//! tagpdf-base.sty — the tagging project's API surface (tagpdf-base.sty:30-60,
//! tagpdf-mc-generic.dtx). The raw package loads as-is; before it does, the
//! no-op stubs `latex_constructs_rust_only.rs` provides for documents that call
//! the expl3/`\tag…` API WITHOUT tagpdf are retracted, because tagpdf-base
//! declares every one of them with `\cs_new_protected`/`\NewDocumentCommand`,
//! whose "already defined" check is an `\errmessage` (counted since batch 56g:
//! 7 errors per `\RequirePackage{pdfmanagement}`, which loads tagpdf-base).
//! Guard: `perfect_kernel_batch56::tagpdf_base_redeclares_the_stubbed_api_cleanly`.
use latexml_engine::latex_constructs_rust_only::retract_pdf_api_stubs;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  retract_pdf_api_stubs();
  InputDefinitions!("tagpdf-base", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
