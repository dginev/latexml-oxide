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
  InputDefinitions!("adjustbox", noltxml => true, extension => Some(Cow::Borrowed("sty")));

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
