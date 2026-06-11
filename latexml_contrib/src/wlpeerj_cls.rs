//! Stub for wlpeerj.cls (Wiley PeerJ template).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // Real wlpeerj.cls: `\RequirePackage{amsmath,amsfonts,amssymb}`.
  // We omitted amsfonts + amssymb before, leaving common math glyphs
  // like \gtrsim, \lesssim, \mathbb undefined for PeerJ-using papers.
  // Witness 2305.10817 (Trieste causality paper).
  RequirePackage!("amsmath");
  RequirePackage!("amsfonts");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  // wlpeerj.cls L23: `\RequirePackage{lineno}` unconditional.
  RequirePackage!("lineno");
  // wlpeerj.cls L55: `\RequirePackage{fancyhdr}` (custom headers/footers).
  // PeerJ-template papers set `\pagestyle{fancy}` and use the legacy fancyhdr
  // interface (`\lhead`/`\chead`/`\rhead`/`\cfoot`/`\fancyplain`) directly in
  // the preamble. Our binding intercepts wlpeerj (OmniBus) and mirrored most of
  // its RequirePackage list but omitted fancyhdr, so those macros were
  // undefined where Perl — which raw-dep-scans the .cls and loads fancyhdr — is
  // clean. Witness 1507.06496.
  RequirePackage!("fancyhdr");

  // Many PeerJ-template papers (witness 2305.10817) use
  // `\printbibliography[…]` without loading biblatex, expecting the
  // class to wire it up. Provide a `\bibliography`-style fallback so
  // we don't fire `Error:undefined:\printbibliography`. If the user
  // actually loads biblatex, our biblatex_sty binding redefines this
  // with the real semantics.
  DefMacro!("\\printbibliography[]", None);

  // Frontmatter — preserve author content.
  // \corrauthor[mark]{name}{email} — emit name as author + email note.
  DefMacro!(
    "\\corrauthor[]{}{}",
    "\\author{#2}\\@add@frontmatter{ltx:note}[role=email]{#3}"
  );
  // \authoraffiliation[mark]{name}{affil} — emit name + affil note.
  DefMacro!(
    "\\authoraffiliation[]{}{}",
    "\\author{#2}\\@add@frontmatter{ltx:note}[role=affiliation]{#3}"
  );
  DefMacro!(
    "\\affil[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
});
