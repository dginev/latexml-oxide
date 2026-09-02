//! scrlayer.sty — KOMA-Script's page-layer engine, raw-interpreted.
//! A registered binding so the raw load also happens under the default
//! (arXiv) configuration (see `tocbasic_sty.rs`). Raw-loading used to die
//! with `Fatal:Timeout:PushbackLimit` at `\begin{document}` because the
//! kernel `\pagestyle` was a non-expandable primitive that scrlayer's
//! `\expandafter`-freeze redefinition (scrlayer.sty L2183-2196) could not
//! inline — fixed at the kernel (`latex_constructs.rs`, KNOWN_PERL_ERRORS
//! #121). The layers it declares are never shipped out (no page output), so
//! `\DeclareNewLayer{…contents=…}` bodies stay unexecuted, exactly as in a
//! run that never reaches `\shipout`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("scrlayer", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
