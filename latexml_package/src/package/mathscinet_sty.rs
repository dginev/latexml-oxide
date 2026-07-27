//! mathscinet.sty — the AMS's transliteration vocabulary for MathSciNet output.
//!
//! Real source: `mathscinet.sty` v1.05 (2002/04/17), American Mathematical
//! Society, LPPL, shipped in TeX Live as part of the **amsrefs** bundle
//! (`texmf-dist/tex/latex/amsrefs/mathscinet.sty`). Perl LaTeXML has no binding
//! for it — `\Dbar` and `\cprime` appear in no `.ltxml` file — so this is
//! Rust-only, not a port of a Perl binding; the `.sty` itself is the ground truth.
//!
//! [MathSciNet](https://mathscinet.ams.org) is the AMS reviewing database, and
//! its BibTeX export assembles a large share of pure-mathematics bibliographies.
//! The macros here are how those records transliterate Cyrillic and South-Slavic
//! names: `\cprime` for the soft sign ь, `\Dbar`/`\dbar` for Đ/đ.
//!
//! **Loaded the normal way, and only the normal way.** `\usepackage{mathscinet}`
//! is the trigger, and `amsrefs.sty` L217 does
//! `\RequirePackage{mathscinet}[2002/01/01]`. All three `\cprime` witnesses name
//! the package themselves (2508.13753 L7, 2508.20226 L3, 2509.07628 L13) — they
//! errored only because no binding existed.
//!
//! Nothing loads it on a document's behalf, and that is deliberate. The
//! recursive `.bib` session does exactly that for `url.sty`, on the grounds that
//! a real `.bst` writes `\providecommand{\url}` into the `.bbl`; the same
//! argument does NOT hold here. Checked on witness 2605.11579: it never mentions
//! mathscinet or amsrefs, and it uses `\bibliographystyle{alpha}` — `alpha.bst`
//! contains no `Dbar` at all. So `\Dbar` is undefined in the author's own build
//! too, real pdflatex raises the same undefined control sequence, and leaving it
//! undefined is PARITY. Supplying it would push our error count below what the
//! author's toolchain produces. Guard:
//! `06_cluster_bibliography::bib_mathscinet_macro_yields_to_the_authors_own_definition`.
//!
//! **`\Provide…` semantics are load-bearing, not incidental.** The upstream file
//! uses `\ProvideTextCommand`/`\ProvideTextCommandDefault` for the character
//! commands, so an author who defined the name first keeps their meaning. That
//! matters concretely: in a 4,000-paper scan of arXiv 2605, six papers define
//! `\Dbar` themselves (four with `\newcommand`) and twelve define `\dbar` — and
//! every one of those twelve means the inexact-differential đ of thermodynamics
//! or a barred derivative, not a Croatian letter. Being a *package* rather than
//! an always-present kernel definition is what keeps us out of their way, and
//! `provide` below reproduces the deferral for the case where the package IS
//! loaded.
//!
//! Each glyph binds the Unicode character carrying the author's visual intent
//! (WISDOM #50). The upstream definitions build their glyphs by overprinting —
//! `\Dbar` is `\leavevmode\lower.5ex\rlap{\hskip-.07em\accent"16}D` — which would
//! reach the XML as a bare "D"; the mappings below are taken from the file's own
//! **T1 branches**, which say what each glyph IS (`\Dbar`→`\DJ`, `\dbar`→`\dj`,
//! `\polhk`→`\k`, `\soft`→`\v`, `\udot`→`\d`).
//!
//! The `\cprime` family additionally keeps an always-on stub in
//! `latex_constructs_rust_only.rs` §5, for `.bib`-borne use in a document that
//! loads no package; `provide` below then finds it defined, the correct no-op.
//! That stub is NOT extended to `\Dbar` — see the collision counts above.
//!
//! Witnesses: 2508.13753, 2508.20226, 2509.07628 (`\cprime`); 2605.11579
//! (`\Dbar` in `MRREVIEWER = {Dragomir \v{Z}. \Dbar okovi\'{c}}`, which stays a
//! parity error because that paper loads no package and its `.bst` has no
//! `Dbar`).
use crate::prelude::*;

/// `\ProvideTextCommandDefault` semantics: define only if the name is free.
fn provide(proto: &str, body: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  if lookup_meaning(&cs_tok).is_some() {
    return Ok(());
  }
  def_macro(
    cs_tok,
    params,
    ExpansionBody::Tokens(mouth::tokenize_internal(body)),
    None,
  )?;
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  // L36 is `\RequirePackage{textcmds}`, deliberately NOT reproduced: textcmds
  // has no binding, so requiring it would raw-load a docstrip `.sty` that plays
  // `\pcatcode` games, to obtain exactly two commands (`\tprime`, `\tsup`) whose
  // results are inlined below as the character and `\textsuperscript`.

  // ---- Math alphabet shorthands (L38-49, verbatim) ----
  provide("\\bold", "\\mathbf")?;
  provide("\\scr", "\\mathcal")?;
  // L41-49 defers \germ to \begin{document} only to diagnose a missing
  // amsfonts; \mathfrak is always available here, so bind it directly.
  provide("\\germ", "\\mathfrak")?;
  // L50-52: superscript shorthands, both \tsup (textcmds).
  provide("\\romsup{}", "\\textsuperscript{#1}")?;
  provide("\\asup{}", "\\textsuperscript{#1}")?;
  provide("\\hslash", "\\hbar")?;

  // ---- Cyrillic / South-Slavic transliteration glyphs ----
  // L57-58: \ProvideTextCommand{\Dbar}{T1}{\DJ}. T1's \DJ IS U+0110 LATIN
  // CAPITAL LETTER D WITH STROKE, so the encoding branch names the character
  // outright; the Default branch just draws it. Witness 2605.11579's
  // `MRREVIEWER = {Dragomir \v{Z}. \Dbar okovi\'{c}}` — Đoković.
  provide("\\Dbar", "\u{0110}")?;
  // L63: \ProvideTextCommand{\dbar}{T1}{\dj} — U+0111, the lowercase đ.
  // Safe to bind HERE and nowhere else: twelve of the twelve papers in the
  // 2605 scan that define `\dbar` themselves mean a math differential, and a
  // package binding cannot reach them unless they load the package.
  provide("\\dbar", "\u{0111}")?;
  // L75-77: \cprime is \tprime (U+02B9 MODIFIER LETTER PRIME), \cdprime is two
  // of them (U+02BA MODIFIER LETTER DOUBLE PRIME), \bud is \cdprime.
  // These render the Russian soft/hard signs in transliterated names —
  // `Gel\cprime fand` is Gelʹfand. Witnesses 2508.13753 (body prose, L2131),
  // 2508.20226, 2509.07628 (a pre-compiled `.bbl`).
  provide("\\cprime", "\u{02B9}")?;
  provide("\\cdprime", "\u{02BA}")?;
  provide("\\bud", "\u{02BA}")?;
  // L78: \cydot — a raised dot used in Cyrillic transliteration.
  provide("\\cydot", "\u{00B7}")?;

  // NOT in the real `mathscinet.sty`: `\Cprime`/`\Cdprime` are the capitalized
  // spellings that `cyracc.def` (L53-55) and MathSciNet `.bst` preambles emit,
  // and they arrive with exactly the same data as `\cprime`. Kept beside it so
  // the four move together; the always-on stub in
  // `latex_constructs_rust_only.rs` §5 covers the same four, so in practice
  // `provide` finds them defined and these are the no-op they should be.
  provide("\\Cprime", "\u{02B9}")?;
  provide("\\Cdprime", "\u{02BA}")?;

  // ---- Under-accents (L104-110) ----
  // These use \DeclareTextCommandDefault, i.e. unconditional, and the file
  // draws them with \@underaccent box kerning. The Unicode combining marks
  // below are the same accents as characters, applied AFTER the base letter.
  // \udot's own default IS the kernel \d (L109), and \polhk/\soft name kernel
  // accents in every encoding branch they define (L111-113, L156-158).
  DefMacro!("\\udot{}", "\\d{#1}");
  DefMacro!("\\polhk{}", "\\k{#1}");
  DefMacro!("\\soft{}", "\\v{#1}");
  DefMacro!("\\utilde{}", "#1\u{0330}");  // combining tilde below
  DefMacro!("\\uarc{}",   "#1\u{032E}");  // combining breve below
  DefMacro!("\\lfhook{}", "#1\u{0326}");  // combining comma below
  DefMacro!("\\dudot{}",  "#1\u{0324}");  // combining diaeresis below
});
