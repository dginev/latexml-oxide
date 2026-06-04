//! mhchem.sty — chemical formula typesetting.
//!
//! TODO(strict-perl-parity): once `latexml_engine` can faithfully
//! handle the expl3 / xparse / chemgreek raw-load chain, DELETE this
//! binding so that `\usepackage{mhchem}` raw-loads the actual TL
//! `mhchem.sty`, matching Perl LaTeXML's behavior (Perl has no
//! `mhchem.sty.ltxml`).
//! Driver paper: arXiv:1806.06448 (3 errors → 0 errors with this
//! stub; full chemistry rendering needs the engine fix).
//!
//! **Current blocker (diagnosed 2026-05-12):** `\ce{H}` with raw-load
//! produces 77 errors in Rust vs 0 in Perl. The cascade starts at
//! `\int_value:w` seeing `;` with no preceding digit — the
//! digit-producing expansion returned nothing. Root-cause hypothesis:
//! `read_x_token` returns PA-aliased CS tokens as opaque
//! `Stored::Token(\let-target)`, causing the csname-reader to error
//! because the let-target is a CS, not a character. Every subsequent
//! expl3 token (`\__int_eval_end:`, `\fi:`, `\else:`, `\s__tl`, …)
//! shifts one slot and surfaces where it shouldn't. The chain is
//! `chemgreek` → `xparse` → expl3 (`\__file_tmp:w`, l3regex,
//! l3tl-analysis). Tracked in `docs/SYNC_STATUS.md` §"mhchem
//! retirement (deferred R36 long-tail)". Next step: instrument
//! `read_x_token` around line 6 col 1 of the minimal repro to
//! narrow the first wrong return.
//!
//! Perl LaTeXML has no `mhchem.sty.ltxml` and raw-loads the actual
//! TL `mhchem.sty` (which `\RequirePackage{chemgreek}` →
//! `\RequirePackage{xparse}` → heavy expl3 machinery). Perl's expl3
//! emulation is mature enough that this works.
//!
//! The specific gap: see "Current blocker" above.
//!
//! Until the expl3 cluster is fixed, this binding intercepts the
//! mhchem load and provides a minimal stub: `\ce{...}` typesets its
//! argument as roman text, no chemistry layout. This is a documented
//! divergence from Perl LaTeXML — the full chemistry rendering needs
//! a real port. Driver paper: 1806.06448 (3 errors → 0 errors).
//!
//! Stubs cover the public mhchem v3/v4 surface most papers actually
//! use: `\ce`, `\cee`, `\cf`, plus `\mhchemoptions` (no-op).

use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // [mhchem retirement probe, 2026-05-19] When env var
  // LATEXML_MHCHEM_NOLTXML is set, bypass this stub and force a
  // raw load of the actual TL mhchem.sty (mirroring Perl
  // LaTeXML's behaviour — Perl has no mhchem.sty.ltxml). Lets us
  // measure the engine gap (expected:<relationaltoken>,
  // unexpected:\fi, etc. — see SYNC_STATUS Cluster E / Task #22).
  // No-op when unset — production users keep the stub.
  if std::env::var("LATEXML_MHCHEM_NOLTXML").is_ok() {
    InputDefinitions!("mhchem", noltxml => true, extension => Some(Cow::Borrowed("sty")));
    return Ok(());
  }
  // Perl LaTeXML auto-scans mhchem.sty for `\RequirePackage` calls
  // and brings in ifthen, calc, twoopt, amsmath, keyval, graphics, pgf,
  // tikz as transitive deps. Since this Rust stub intercepts the load
  // (so the raw RequirePackage chain never fires), papers that rely on
  // those deps via mhchem alone hit undefined-CS errors. Pull in the
  // ones most commonly needed: amsmath (for \boldsymbol, \eqref,
  // \text, align*, etc.) and graphicx (for figure handling). Witness:
  // 1311.6762 (stage 15 RUST-REGRESSION) — paper loads mhchem but
  // not amsmath, then uses `\boldsymbol` / `\eqref`. Perl's auto-dep
  // scan loads amsmath → 0 errors; Rust stub didn't → 2 errors.
  RequirePackage!("amsmath");
  RequirePackage!("graphicx");

  // Accept both v3 and v4: the package option is `version=N` — handled
  // at \usepackage time but irrelevant to our stub.
  def_macro_noop("\\mhchemoptions RequiredKeyVals")?;

  // \ce{<formula>} — chemistry mode. Real mhchem renders subscripts,
  // charges, arrows, etc. Papers invoke \ce{H_2O} / \ce{N_2} both in
  // math context (equation*) AND in text context (paragraphs).
  // \ensuremath wraps body in math mode if not already in math, so
  // `_`/`^` parse as scripts in both contexts. Loses roman-text
  // rendering of plain text chemistry, but avoids cascading errors.
  //
  // Strip embedded `$` toggles from the body before re-entering math:
  // mhchem v3 papers commonly write `\ce{Cs$_x$MA$_{1-x}$PbI3}` where
  // the `$` pairs are mhchem's own subscript-grouping hint, NOT real
  // math toggles. Without stripping, `\ensuremath{...$_x$...}` re-toggles
  // out of math at the first `$`, leaving `_x` in text mode — which
  // errors with "Script _ can only appear in math mode".
  // Witnesses: 1908.05236 (\ce{MAPb(I_{1-x}Br_x)3}), 0907.1390 (\ce{N_2}).
  //
  // Also convert `#` (PARAM-catcode) tokens to `\equiv` CS: mhchem v3
  // uses `#` for triple bond (e.g. `\ce{-C#C-}` renders as `-C≡C-`).
  // Without conversion, the bare `#` reaches the Stomach as a PARAM
  // token and triggers "should never reach Stomach!". Witness:
  // arXiv:2508.11040 (`\ce{-C#C-}`).
  fn strip_math_toggles(arg: &Tokens) -> Tokens {
    let mut out: Vec<Token> = Vec::with_capacity(arg.unlist_ref().len());
    for tok in arg.unlist_ref().iter().copied() {
      match tok.get_catcode() {
        Catcode::MATH => continue,
        Catcode::PARAM => out.push(T_CS!("\\equiv")),
        _ => out.push(tok),
      }
    }
    Tokens::new(out)
  }
  // Wrap the `\ensuremath{…}` in an explicit `{ }` group so that `\ce` used
  // as a sub/superscript operand — `E_\ce{M_{bcc}}` (witness 1709.05523) —
  // binds as ONE math atom. `\ensuremath{X}` strips its OWN braces in math
  // mode (it is `\def\ensuremath#1{\ifmmode#1\else$#1$\fi}`), so a bare
  // `E_\ensuremath{M_{bcc}}` expands to `E_M_{bcc}` → the inner `_` becomes a
  // SECOND subscript on `E` ("Double subscript"). The extra group is
  // transparent for rendering. Real mhchem (Perl) produces a single boxed
  // unit, so `E_\ce{…}` is one atom there.
  fn ce_expand(body: &Tokens) -> Tokens {
    let stripped = strip_math_toggles(body);
    let mut result = vec![T_BEGIN!(), T_CS!("\\ensuremath"), T_BEGIN!()];
    result.extend(stripped.unlist());
    result.push(T_END!());
    result.push(T_END!());
    Tokens::new(result)
  }
  DefMacro!("\\ce{}",  sub[(body)] { Ok(ce_expand(&body)) });
  DefMacro!("\\cee{}", sub[(body)] { Ok(ce_expand(&body)) });
  DefMacro!("\\cf{}",  sub[(body)] { Ok(ce_expand(&body)) });

  // \arrow / \chemarrow — used inside \ce arguments. Stub as small text
  // arrow so a `\ce{A \arrow B}` doesn't error if it leaks out.
  DefMacro!("\\chemarrow", "\\rightarrow");

  // \bond{<type>} — mhchem bond operator, used inside \ce, e.g.
  // `\ce{H2O\bond{...}H2O}` (hydrogen bond) or bare `\ce{HC#CH\bond}`
  // (trailing single bond). Real mhchem (mhchem.sty L3217-3243)
  // `\mhchem@bond{#1}` str_case-maps the type to a `\resizebox`-rendered
  // bond line; the layout is moot in our XML paradigm, so map each type to
  // the corresponding math glyph. `\ce` already runs us inside `\ensuremath`.
  // `\bond` may appear bare (no following `{...}`) for a single bond — peek
  // with `\@ifnextchar\bgroup` so the bare form doesn't swallow the closing
  // brace. Witness 1608.02559 (`\ce{H2O\bond{...}H2O}`, `\ce{HC#CH\bond}`).
  RawTeX!(r"\def\bond{\@ifnextchar\bgroup\lx@mhchem@bond@typed\lx@mhchem@bond@single}");
  DefMacro!("\\lx@mhchem@bond@single", "{-}");
  DefMacro!("\\lx@mhchem@bond@typed{}", sub[(typ)] {
    // mhchem.sty L3223-3237 type table. Unknown → single bond (mhchem
    // raises an error; we render a single bond, staying error-free).
    let glyph = match typ.to_string().trim() {
      "-" | "1"            => r"{-}",
      "=" | "2"            => r"{=}",
      "#" | "##" | "3"     => r"{\equiv}",
      "~"                  => r"{\sim}",
      "~-"                 => r"{\sim\!\!-}",
      "~--" | "~=" | "-~-" => r"{\sim\!\!=}",
      "..." | "...."       => r"{\cdots}",
      "->"                 => r"{\rightarrow}",
      "<-"                 => r"{\leftarrow}",
      _                    => r"{-}",
    };
    Ok(Tokenize!(glyph))
  });
});
