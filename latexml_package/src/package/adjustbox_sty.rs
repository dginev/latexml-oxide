use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: adjustbox.sty.ltxml
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

  // \lx@RE@BOXCONTENT: complex sub{} body — stub as no-op
  // In Perl: looks up \collectedbox register, reverts & re-digests the box
  DefMacro!("\\lx@RE@BOXCONTENT", None);
});
