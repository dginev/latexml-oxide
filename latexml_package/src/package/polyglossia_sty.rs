//! polyglossia.sty — raw load, then the font-script check answers TRUE.
//!
//! polyglossia.sty:719-721 `\__xpg_setup_font:nn {rm}/{sf}/{tt}` redefines
//! `\rmfamily`/`\sffamily`/`\ttfamily` (:690-718) so that a font switch inside
//! a non-Latin language calls `\__xpg_add_font_feature_script:nnn` (:632),
//! whose `\xpg_if_script:nTF {script}` (:641) either adds
//! `\addfontfeature{Script=…}` or raises "The current main roman font, cmr10,
//! does not contain the "Greek" script!" (:677 — a counted `\errmessage` since
//! batch 56g). `\xpg_if_script:n` is `\fontspec_if_script:n` (:47, pdfTeX
//! profile) or is gated by `\fontspec_if_fontspec_font:TF` (:31-45, luatex
//! profile); fontspec_sty.rs stubs both constant-FALSE because LaTeXML has no
//! OpenType font model — the "current" font is always cmr10, so the check can
//! never truthfully pass, while a lualatex-clean document did load a
//! script-capable font. The faithful answer is TRUE: the `\addfontfeature`
//! branch is a no-op here. Perl raw-loads polyglossia too (no `.ltxml`) but
//! fails earlier on the fontspec conditionals it never defines.
//! Witnesses: fontsetup/fspsample ×2 (11 → 0, lualatex clean), greektonoi
//! (16 → 0), latex-mr (98 + Fatal → 3).
//! Guard: `perfect_kernel_batch56::polyglossia_script_check_passes_without_font`.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("polyglossia", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\ExplSyntaxOn
\cs_if_exist:cT { xpg_if_script:nTF }
  { \prg_set_conditional:Nnn \xpg_if_script:n { TF, T, F } { \prg_return_true: } }
\ExplSyntaxOff"
  );
});
