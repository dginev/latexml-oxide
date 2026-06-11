//! mathpartir.sty — math paragraphs for typesetting inference rules
//! by Didier Remy.
//!
//! Provides `mathpar`/`mathparpagebreakable` environments and the
//! `\inferrule[label]{premises}{conclusion}` command for
//! horizontally-laid-out inference rules.
//!
//! Perl LaTeXML has no mathpartir binding. With its default
//! `INCLUDE_STYLES=false` Perl skips the raw .sty file, emits a
//! single "missing binding" warning, and lets the user document
//! continue — any `\inferrule`/`mathpar` use surfaces as one
//! "undefined" error per use site.
//!
//! Our `INCLUDE_STYLES=true` default raw-loads mathpartir.sty,
//! which hits `\halign \bgroup \hfil $##$\hfil\cr` patterns in
//! `\mathvbox@` — the `##` PARAM tokens cascade as `# should
//! never reach Stomach` errors and the `\halign` outside an
//! alignment template trips `Missing \halign box` (witness
//! arXiv:1310.8644: amsart + mathpartir → 100+ errors + fatal,
//! while Perl converts with the single missing-binding warning).
//!
//! Stub the public API as faithful enough wrappers — inference
//! rules render as a vertical `numerator / line / denominator`
//! frac-style — so the raw file is skipped and the document
//! continues. Lost fidelity: precise label placement, multi-rule
//! `\and` separation. Gained: error-free conversion + readable
//! inference rules.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "mathpartir.sty",
    "mathpartir.sty is minimally stubbed — \\inferrule renders as a frac; mathpar / mathparpagebreakable wrap their body in a display equation."
  );

  // \inferrule[label]{premises}{conclusion}
  //   → \ensuremath{\frac{premises}{conclusion}} with the label
  //     appended parenthetically when present.
  // The `\frac` is math-mode-only, but mathpartir's `\inferrule` is
  // routinely used in *text* mode (e.g. bare inside `\begin{tabular}{c}`
  // — witness arXiv:1404.0085 §3 Fig.3, the π-calculus reduction rules).
  // Emitting a bare `\frac` there drops the math `XMApp` straight into
  // the `<ltx:td>` with no `<ltx:Math>` wrapper → "ltx:XMApp isn't
  // allowed in <ltx:td>" (Perl, which raw-loads the real mathpartir,
  // wraps it). `\ensuremath` enters math mode only when not already in
  // it, so this is correct in BOTH text-mode (tabular) and math-mode
  // (`mathpar` = display equation) use sites.
  // Use OptionalMatch:* to consume the starred form `\inferrule*`
  // (mathpartir's \mpr@inferstar branch). Optional `[label]`,
  // then two required {} args.
  DefMacro!("\\inferrule OptionalMatch:* [] {} {}", sub[(_star, label, prem, conc)] {
    let mut out: Vec<Token> = Vec::new();
    out.push(T_CS!("\\ensuremath"));
    out.push(T_BEGIN!());
    out.push(T_CS!("\\frac"));
    out.push(T_BEGIN!());
    // mathpartir separates premises with `\\`, laying them out SIDE BY SIDE
    // above the rule line. We render via `\frac`, so the `\\` must become a
    // horizontal separator (`\quad`) — emitting it raw inside `\frac{…}` is a
    // hard error in BOTH Perl and Rust ("\\ in \frac"), and inside a display
    // alignment (`gather*`/`align`) the leaked `\\` starts a spurious row,
    // desyncing math mode → `\lx@end@inline@math … end mode math in math`
    // cascade (witness 1702.02972: llncs + `\begin{gather*}\inferrule{A \\ B}{C}`,
    // Perl 0, raw-mathpartir lays the premises out without leaking `\\`).
    // Skip a `\\[dim]` optional spacing arg so it doesn't render literally.
    let prem_toks = prem.unlist();
    let mut i = 0;
    while i < prem_toks.len() {
      if prem_toks[i] == T_CS!("\\\\") {
        out.push(T_CS!("\\quad"));
        i += 1;
        // Skip an optional `[dim]` spacing argument following `\\`.
        if i < prem_toks.len() && prem_toks[i].get_catcode() == Catcode::OTHER
          && prem_toks[i].to_string() == "["
        {
          while i < prem_toks.len()
            && !(prem_toks[i].get_catcode() == Catcode::OTHER && prem_toks[i].to_string() == "]")
          {
            i += 1;
          }
          if i < prem_toks.len() {
            i += 1; // consume the `]`
          }
        }
      } else {
        out.push(prem_toks[i]);
        i += 1;
      }
    }
    out.push(T_END!());
    out.push(T_BEGIN!());
    out.extend(conc.unlist());
    out.push(T_END!());
    out.push(T_END!());
    if let Some(lab) = label
      && !lab.is_empty() {
        out.push(T_CS!("\\quad"));
        out.push(T_CS!("\\textsc"));
        out.push(T_BEGIN!());
        out.extend(lab.unlist());
        out.push(T_END!());
      }
    Ok(Tokens::new(out))
  });
  // `\infer` is a deprecated alias — install it ONLY when `\infer` is not
  // already defined. The real mathpartir.sty does not define `\infer` at all
  // (it is the `proof` package's command, whose premise syntax splits on `&`:
  // `\infer{concl}{prem1 & prem2}`). An UNCONDITIONAL `\let\infer\inferrule`
  // clobbered proof's `\infer` whenever `proof` was loaded BEFORE `mathpartir`
  // — then `\infer{C}{A & B}` ran `\inferrule` (premises split on `\\`, not `&`)
  // and every premise `&` errored "Stray alignment &" during Building (witness
  // 1801.08114: `\usepackage{proof}` then `\usepackage{mathpartir}` → 15 errors,
  // Perl 0). Guarding with `\@ifundefined` preserves proof's `\infer` and still
  // offers the alias to mathpartir-only documents.
  RawTeX!(r"\@ifundefined{infer}{\let\infer\inferrule}{}");

  // `mathpar` environment: wrap body in a display equation.
  // Optional `[keys]` consumed and discarded — we don't honour
  // mathpartir's lineskip / column-width keys, those are
  // typesetting hints with no HTML equivalent.
  DefMacro!("\\mathpar []", "\\begin{equation*}");
  DefMacro!("\\endmathpar", "\\end{equation*}");
  // mathparpagebreakable is identical for our rendering purposes —
  // it differs from `mathpar` only in page-break behaviour, which
  // is irrelevant in HTML.
  DefMacro!("\\mathparpagebreakable []", "\\begin{equation*}");
  DefMacro!("\\endmathparpagebreakable", "\\end{equation*}");

  // Configuration setters: silently consume their keyval argument.
  DefMacro!("\\mprset {}", "");
  DefMacro!("\\MathparLineskip", "");
  DefMacro!("\\MathparBindings", "");
  // Label-style hooks: pass-through `#1` so the label still renders.
  DefMacro!("\\TirName {}", "\\textsc{#1}");
  DefMacro!("\\LeftTirName {}", "\\textsc{#1}");
  DefMacro!("\\RightTirName {}", "\\textsc{#1}");
  DefMacro!("\\TirNameStyle {}", "\\textsc{#1}");
  DefMacro!("\\LeftTirNameStyle {}", "\\textsc{#1}");
  DefMacro!("\\RightTirNameStyle {}", "\\textsc{#1}");
});
