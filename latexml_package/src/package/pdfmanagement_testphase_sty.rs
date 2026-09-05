//! pdfmanagement-testphase.sty — the PDF management bundle (combined l3pdfmeta/l3pdffile/
//! l3pdfdict code + tagpdf-base). Loads raw; the no-op stubs of its expl3 API
//! in `latex_constructs_rust_only.rs` are retracted first, since the bundle
//! `\cs_new`s every one of them (`already defined` = a counted `\errmessage`).
//! Guard: `perfect_kernel_batch56::tagpdf_base_redeclares_the_stubbed_api_cleanly`.
use latexml_engine::latex_constructs_rust_only::retract_pdf_api_stubs;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  retract_pdf_api_stubs();
  InputDefinitions!("pdfmanagement-testphase", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
