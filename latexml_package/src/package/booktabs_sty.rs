use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: booktabs.sty.ltxml
  // Adjust thickness of rules? Currently no support for variable thickness.

  // \toprule[thickness]  doubled
  DefMacro!("\\toprule[Dimension]", "\\hline\\hline");
  // \midrule[thickness]
  DefMacro!("\\midrule[Dimension]", "\\hline");
  // \bottomrule[thickness] doubled
  DefMacro!("\\bottomrule[Dimension]", "\\hline\\hline");

  // \cmidrule[thickness](trim){col-col}
  DefMacro!("\\@afterfi Until:\\fi", "\\fi#1");
  DefMacro!("\\cmidrule[]",
    r"\@ifnextchar({\ifx.#1.\expandafter\ltx@@cmidrule\else\@afterfi\ltx@@cmidrule[#1]\fi}{\ifx.#1.\expandafter\ltx@cmidrule\else\@afterfi\ltx@cmidrule[#1]\fi}"
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
  DefMacro!("\\ltx@@cmidrule[Dimension] SkipMatch:( Until:){}", "\\ltx@saved@cline{#3}");
  DefMacro!("\\ltx@cmidrule[Dimension]{}", "\\ltx@saved@cline{#2}");

  // add vspace
  def_macro_noop("\\addlinespace[Dimension]")?;
  // adjust spacing to make double line
  def_macro_noop("\\morecmidrules")?;
  // \specialrule{thickness}{above}{below}
  DefMacro!("\\specialrule{Dimension}{Dimension}{Dimension}", "\\hline");

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
