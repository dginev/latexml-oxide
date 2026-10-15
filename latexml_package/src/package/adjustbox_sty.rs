use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: adjustbox.sty.ltxml
  // adjustbox.sty `\RequirePackage{adjcalc}` + `\RequirePackage{calc}` at its top —
  // calc supplies \widthof/\heightof/\depthof. The raw InputDefinitions below does not
  // reliably propagate adjustbox's own `\RequirePackage{calc}`, so docs that use
  // \widthof (e.g. via enumitem `labelwidth=\widthof{...}`) without explicitly loading
  // calc get \widthof undefined. Load calc explicitly first (faithful — adjustbox
  // requires it). Witness 2401.06320. RUST-ONLY (Perl loads calc via adjustbox).
  RequirePackage!("calc");
  // Its trimclip finds no clipping driver for the empty `\Gin@driver` here (trimclip.sty:24, 252-258) and loads its
  // fall-back, tc-pgf.def, so pgf is read now, where pdflatex reads tc-pdftex.def and no pgf. pgf's code needs `|` other
  // (pgfmathparser.code.tex:912 `\pgfmathdeclareoperator{||}`), as its own library loader sets it (pgfutil-common.tex:
  // 372-384); a class that leaves `|` active (sn-jnl's program.sty:521, until the paper's `\normalbaroutside`) broke it,
  // 6 errors (2202.10115, 2306.09847, 2507.09618). The load reads `|` as other, its catcode put back after.
  RawTeX!(r"\expandafter\edef\csname lx@adjustbox@barcode\endcsname{\the\catcode`\|}\catcode`\|=12\relax");
  InputDefinitions!("adjustbox", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(r"\catcode`\|=\csname lx@adjustbox@barcode\endcsname\relax");

  // Some strategic redefinitions
  // collectbox's approach to starting a block for environments isn't quite working; Force a \par
  Let!("\\lx@save@@adjustbox", "\\@adjustbox");
  DefMacro!("\\@adjustbox", "\\ifcollectboxenv\\par\\fi\\lx@save@@adjustbox");

  // Redefined so the frame contains \BOXCONTENT, rather than (attempted) \hskip overlap
  // \adjbox@@frame{setframecolor}{fboxrule}{fboxsep}{???}
  DefMacro!("\\adjbox@@frame{}{}{}{}",
    "\\ifx\\@nnil#2\\@nnil\\else\\adjsetlength\\fboxrule{#2}\\fi\\ifx\\@nnil#3\\@nnil\\else\\adjsetlength\\fboxsep{#3}\\fi\\@framebox{\\BOXCONTENT}");

  // Since adjustbox is adapting the already digested content in \BOXCONTENT,
  // and we encode color & bgcolor in the font, which is already incorporated into the box
  // we need to RE-digest the box, to apply the changed color!
  DefMacro!("\\@bgcolorbox{}", "{\\let\\color\\pagecolor\\hbox{#1\\lx@RE@BOXCONTENT}}");

  // Perl L44-48: looks up \collectedbox register, reverts & re-digests the box.
  DefPrimitive!("\\lx@RE@BOXCONTENT", sub[_args] {
    if let Ok(Some(cbox_val)) = lookup_register("\\collectedbox", Vec::new()) {
      let box_name = s!("box{}", cbox_val.value_of());
      if let Some(Stored::Digested(d)) = lookup_value(&box_name) {
        let reverted = d.revert()?;
        return digest(reverted).map(|d| vec![d]);
      }
    }
    Ok(Vec::new())
  });
});
