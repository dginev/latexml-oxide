use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: booktabs.sty.ltxml
  // Adjust thickness of rules? Currently no support for variable thickness.
  // The widths and spaces are therefore read untyped: they are unused, and a
  // typed read hands what follows the value back to the input (OXIDIZED_DESIGN
  // #317), which between rows opens a cell — `\cmidrule[lr]{1-2}` for `(lr)`
  // put "lr" in the next row and lost the next rule (2605.27476, 2605.25272),
  // `\addlinespace[2pt plus 1pt]` put "plus 1pt" there (booktabs.sty:80-82 reads
  // a skip). Perl reads `[Dimension]` (booktabs.sty.ltxml) and drops the rest.

  // The width is still a <dimen> to TeX (booktabs.sty:85-114 `\@BTrule[#1]`
  // assigns it), so one that is not reports what Perl's `[Dimension]` read
  // reports — "Missing number (Dimension)" / "Illegal unit" Warnings
  // (Gullet.pm:969-972), at the rule — judged on its text (`width_shape`, the
  // shapes of tex.web §448-458) at expansion time, so nothing is left between the
  // rows (witnesses 2605.27476, 2605.25272: `\cmidrule[lr]`). pdflatex: the same as Errors. Under
  // tabularray (its `booktabs` library, `\toprule[2pt,red]`) the brackets hold
  // tabularray's keys, not a width.
  DefMacro!("\\lx@booktabs@width{}", sub[(width)] {
    if !width.is_empty() && !(lookup_bool("tabularray.sty_loaded") || lookup_bool("tabularray.sty_raw_loaded")) {
      match width_shape(&width.to_string()) {
        WidthShape::Dimension => {},
        WidthShape::NoUnit => Warn!(
          "expected",
          "<unit>",
          "Illegal unit of measure (pt inserted).",
          s!("in the rule width [{width}]")
        ),
        WidthShape::NoNumber => Warn!(
          "expected",
          "<number>",
          "Missing number (Dimension), treated as zero.",
          s!("in the rule width [{width}]")
        ),
      }
    }
    Ok(Tokens!())
  });

  // \toprule[thickness]  doubled
  DefMacro!("\\toprule[]", "\\lx@booktabs@width{#1}\\hline\\hline");
  // \midrule[thickness]
  DefMacro!("\\midrule[]", "\\lx@booktabs@width{#1}\\hline");
  // \bottomrule[thickness] doubled
  DefMacro!("\\bottomrule[]", "\\lx@booktabs@width{#1}\\hline\\hline");

  // \cmidrule[thickness](trim){col-col}
  DefMacro!("\\@afterfi Until:\\fi", "\\fi#1");
  // Whether a width was given: `\detokenize`'s emptiness test, as `\ifx.#1.` is true for `[.5pt]`, whose leading `.`
  // it compares with its own, so "5pt." was typeset into the next cell (2405.01314; Perl booktabs.sty.ltxml:31-33).
  DefMacro!("\\cmidrule[]",
    r"\lx@booktabs@width{#1}\@ifnextchar({\if\relax\detokenize{#1}\relax\expandafter\ltx@@cmidrule\else\@afterfi\ltx@@cmidrule[#1]\fi}{\if\relax\detokenize{#1}\relax\expandafter\ltx@cmidrule\else\@afterfi\ltx@cmidrule[#1]\fi}"
  );
  // The cmidrule helpers draw the partial rule via `\cline`. They route through
  // a PRIVATE saved copy (`\ltx@saved@cline`, captured at load below) rather than
  // the public `\cline`, so a document `\let\cline\cmidrule` (a common idiom to
  // make `\cline` render like a booktabs rule) does NOT create a
  // `\cmidrule`→`\cline`→`\cmidrule` infinite expansion. Real LaTeX avoids the
  // cycle because its `\cmidrule` draws the rule directly; LaTeXML's simplified
  // `\cmidrule`→`\cline` binding (shared with Perl, which hangs on this — see
  // KNOWN_PERL_ERRORS) would otherwise loop until the conditional limit.
  // Witnesses: arXiv 2506.23179, 2511.17056 (both `\let\cline\cmidrule`).
  // Output-neutral for ordinary `\cmidrule` (saved CS == `\cline` at load).
  DefMacro!("\\ltx@@cmidrule[] SkipMatch:( Until:){}", "\\ltx@saved@cline{#3}");
  DefMacro!("\\ltx@cmidrule[]{}", "\\ltx@saved@cline{#2}");

  // add vspace
  def_macro_noop("\\addlinespace[]")?;
  // adjust spacing to make double line
  def_macro_noop("\\morecmidrules")?;
  // \specialrule{thickness}{above}{below}
  DefMacro!("\\specialrule{}{}{}", "\\hline");

  // Capture the real `\cline` at load time (before any document redefinition)
  // so `\cmidrule` can draw its rule without depending on the live `\cline`.
  TeX!(r"\let\ltx@saved@cline\cline");

  TeX!(r"\newdimen\heavyrulewidth
\newdimen\lightrulewidth
\newdimen\cmidrulewidth
\newdimen\belowrulesep
\newdimen\belowbottomsep
\newdimen\aboverulesep
\newdimen\abovetopsep
\newdimen\cmidrulesep
\newdimen\cmidrulekern
\newdimen\defaultaddspace
\heavyrulewidth=.08em
\lightrulewidth=.05em
\cmidrulewidth=.03em
\belowrulesep=.65ex
\belowbottomsep=0pt
\aboverulesep=.4ex
\abovetopsep=0pt
\cmidrulesep=\doublerulesep
\cmidrulekern=.5em
\defaultaddspace=.5em
\newcount\@cmidla
\newcount\@cmidlb
\newdimen\@aboverulesep
\newdimen\@belowrulesep
\newcount\@thisruleclass
\newcount\@lastruleclass
");
  // booktabs.sty:53-118 — the rule machinery a document reaches when it
  // copies the REAL `\midrule`/`\toprule` verbatim over the binding's
  // simplified ones (l2kurz.tex:58-65 = booktabs.sty:67-71): the `\noalign{`
  // opened by `\ifnum0=`}\fi` is closed only by `\@BTendrule`'s
  // `\ifnum0=`{\fi}`, so with `\@BTrule` undefined the noalign body over-ran
  // to the end of the table and the alignment frame leaked through the whole
  // document (lshort-german l2kurz, 41 errors; Perl shares the gap). The
  // binding's own `\toprule`/`\midrule`/`\bottomrule` stay `\hline`-based;
  // longtable's `\@BLTrule` branch (booktabs.sty:103-108, cmidrule kerning
  // internals) reduces to `\@BTnormal`. Guard:
  // `perfect_kernel_batch56::booktabs_rule_machinery_closes_its_noalign`.
  TeX!(r"\@lastruleclass=0
\@ifundefined{@thisrulewidth}{\newdimen\@thisrulewidth}{}
\def\futurenonspacelet#1{\def\@BTcs{#1}\afterassignment\@BTfnslone\let\nexttoken= }
\def\@BTfnslone{\expandafter\futurelet\@BTcs\@BTfnsltwo}
\def\@BTfnsltwo{\expandafter\ifx\@BTcs\@sptoken\let\next=\@BTfnslthree
   \else\let\next=\nexttoken\fi \next}
\def\@BTfnslthree{\afterassignment\@BTfnslone\let\next= }
\def\@addspace[#1]{\global\@belowrulesep=#1\global\@thisruleclass=\tw@
  \futurelet\@tempa\@BTendrule}
\def\@BTrule[#1]{\let\@BTswitch\@BTnormal
  \global\@thisrulewidth=#1\relax
  \ifnum\@thisruleclass=\tw@\vskip\@aboverulesep\else
  \ifnum\@lastruleclass=\z@\vskip\@aboverulesep\else
  \ifnum\@lastruleclass=\@ne\vskip\doublerulesep\fi\fi\fi
  \@BTswitch}
\providecommand*\CT@arc@{}
\def\@BTnormal{{\CT@arc@\hrule\@height\@thisrulewidth}\futurenonspacelet\@tempa\@BTendrule}
\let\@BLTrule\@BTnormal
\let\@BTswitch\@BTnormal
\def\@BTendrule{\ifx\@tempa\toprule\global\@lastruleclass=\@thisruleclass
  \else\ifx\@tempa\midrule\global\@lastruleclass=\@thisruleclass
  \else\ifx\@tempa\bottomrule\global\@lastruleclass=\@thisruleclass
  \else\ifx\@tempa\cmidrule\global\@lastruleclass=\@thisruleclass
  \else\ifx\@tempa\specialrule\global\@lastruleclass=\@thisruleclass
  \else\ifx\@tempa\addlinespace\global\@lastruleclass=\@thisruleclass
  \else\global\@lastruleclass=\z@\fi\fi\fi\fi\fi\fi
  \ifnum\@lastruleclass=\@ne\relax\else\vskip\@belowrulesep\fi
  \ifnum0=`{\fi}}
");
});

/// How a booktabs rule width reads as a TeX `<dimen>` (tex.web §448-458).
enum WidthShape {
  /// A factor and a unit (`.5pt`, `2.5\arrayrulewidth`), or a register or macro (`\heavyrulewidth`).
  Dimension,
  /// A number with no unit (`0`): "Illegal unit of measure (pt inserted)".
  NoUnit,
  /// No number (`lr`, a typo for `(lr)`): "Missing number, treated as zero".
  NoNumber,
}

fn width_shape(width: &str) -> WidthShape {
  static DIMENSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
      // TeX's unit keywords ignore case (tex.web §407 scan_keyword: `\toprule[1PT]`).
      r"(?i)^\s*[-+\s]*(?:(?:\d+[.,]?\d*|[.,]\d+)\s*(?:true\s*)?(?:pt|pc|in|bp|cm|mm|dd|cc|sp|em|ex|px|nd|nc)|(?:\d+[.,]?\d*|[.,]\d+)?\s*\\[A-Za-z@])",
    )
    .unwrap()
  });
  static NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*[-+\s]*(?:\d|[.,]\d)").unwrap());
  if DIMENSION.is_match(width) {
    WidthShape::Dimension
  } else if NUMBER.is_match(width) {
    WidthShape::NoUnit
  } else {
    WidthShape::NoNumber
  }
}
