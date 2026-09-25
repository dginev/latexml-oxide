//! Batch 56co: two RUST-ONLY residues of latex-via-exemplos (root-causer
//! `~/data/pk_agents/w23/regr83/latex-via-exemplos/NOTES.md`).

/// tabto.sty:183 `\newcommand\NumTabs[1]`: the binding's former
/// `OptionalMatch:* {}{}` signature ate the token after `\NumTabs{5}` — the
/// following `\begin{itemize}` lost its `\begin`, the environment never
/// opened and its `\end` errored.
#[test]
fn numtabs_takes_one_argument() {
  let tex = "\\documentclass{article}\n\\usepackage{tabto}\n\\begin{document}\n\\NumTabs{5}\n\\begin{itemize}\\item First\\item Second\\end{itemize}\nAfter.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "itemize",
    &[],
    r##"<itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>First</p></para></item><item xml:id="S0.I1.i2"><tags><tag>•</tag><tag role="typerefnum">2nd item</tag></tags><para xml:id="S0.I1.i2.p1"><p>Second</p></para></item></itemize>"##,
  );
}

/// wrapstuff.sty:2382-2384 declares its box's float type through caption's
/// internals (`\caption@settype{table}`, `\caption@clearmargin`,
/// `\caption@setoptions{wrap table}`); with the binding replacing raw
/// caption.sty they were undefined (two errors per box). The internals are
/// exercised directly: the type reaches `\@captype` and the caption is its
/// table (batch 56gs: "Table 1", no longer an inline caption text). (wrapstuff's own box is still dropped whole — a separate
/// content-loss finding, DIFFICULT_CASES D15.)
#[test]
fn caption_settype_declares_the_float_type() {
  let tex = "\\documentclass{article}\n\\usepackage{caption}\n\\makeatletter\n\\begin{document}\n\\begin{minipage}{5cm}\\caption@settype{table}\\caption@clearmargin\\caption@setoptions{wrap table}\\caption{A wrapped table caption}\\end{minipage}\nAfter.\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "table",
    &[r#"class="ltx_minipage""#],
    r##"<table class="ltx_minipage" inlist="lot" vattach="middle" width="142.3pt" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><caption><tag close=": ">Table 1</tag>A wrapped table caption</caption></table>"##,
  );
}
