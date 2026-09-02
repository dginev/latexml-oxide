//! datetime2.sty — raw-loaded (N. Talbot; etoolbox + xkeyval + tracklang).
//!
//! The former stub defined a dozen no-ops and left `\DTMsetup`, `\DTMdate`,
//! `\DTMdisplaydate`, `\DTMnow` and every regional style undefined (cnltx
//! and chemformula manuals set `\DTMsetup{useregional}` in their classes).
//! The real file runs cleanly once `\pdfcreationdate` expands to pdfTeX's
//! `D:YYYYMMDDhhmmss` stamp (datetime2.sty:46-48 seeds its clock from it).
//! Regional modules (`datetime2-<lang>.ldf`) load raw through tracklang.
//! Guard: `perfect_kernel_batch54::datetime2_raw_dates_render`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("datetime2", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
