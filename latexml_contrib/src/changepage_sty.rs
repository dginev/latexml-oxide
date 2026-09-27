use latexml_package::prelude::*;

/// `\ch@ngetext` in LaTeXML's one column. changepage's (changepage.sty:80-88), chngpage's
/// (chngpage.sty:79-88) and memoir's (memoir.cls:11237-11245), which `\changetext`/`\changepage`
/// end with, halve `\columnwidth` under `\if@twocolumn`; LaTeXML sets one column, so
/// `\columnwidth` is the changed `\textwidth` and `\linewidth` follows it (every later
/// `width=\linewidth` graphic at half width otherwise). OXIDIZED_DESIGN_DIVERGENCES #341.
pub const ONE_COLUMN_CH_NGETEXT: &str = r"\DeclareRobustCommand{\ch@ngetext}{%
  \setlength{\@colht}{\textheight}\setlength{\@colroom}{\textheight}%
  \setlength{\vsize}{\textheight}\setlength{\columnwidth}{\textwidth}%
  \setlength{\hsize}{\columnwidth}\setlength{\linewidth}{\hsize}}";

LoadDefinitions!({
  // memoir provides changepage's commands itself (memoir.cls:12216 `\EmulatedPackage{changepage}`,
  // which a later `\RequirePackage` then skips), and changepage.sty stops under memoir
  // (changepage.sty:8-11): memoir's `\checkoddpage` and page macros stay. Its `{adjustwidth}`
  // (memoir.cls:11267-11297) is the same list as changepage's, so the environments below are
  // defined either way.
  if !(lookup_bool("memoir.cls_loaded") || lookup_bool("memoir.cls_raw_loaded")) {
    // The page-check and page-layout macros are the package's own TeX: `[strict]` (:22),
    // `\ifoddpage`, the `cp@cntr` counter and `\cp@tempcnt` (:29-31; the dgruyter.sty that arXiv
    // 2605.13539 and 2605.16066 ship reads it), `\cplabel` (:32), `\checkoddpage` (:59-67; odd, on
    // LaTeXML's one page).
    InputDefinitions!("changepage", noltxml => true, extension => Some(Cow::Borrowed("sty")));
    RawTeX!(ONE_COLUMN_CH_NGETEXT);
  }
  // `{adjustwidth}` and `{adjustwidth*}` are lists (:110-139): the list's end closes the paragraph
  // (`\endtrivlist`'s `\ifhmode\unskip\par`, latex.ltx:15915-15926) and the text after it goes on in
  // the same logical paragraph (`\@doendpe`, :15939). The body stays in the flow — loaded raw it is
  // a tagless one-item <itemize>, in which a bibliography is malformed — and its last paragraph
  // closes before the end. `internal_vertical` keeps `$$` inside a display (witness 2305.09826);
  // `{adjustwidth*}` is its own environment (:122; witness 2006.09676). The margins are read, not
  // typeset: digested, `{-0.005\linewidth}` assigned `\linewidth` = 0pt (witness 2605.02723, a
  // figure at zero width). The ar5iv binding this replaced stubbed all of it (KPE #318); the
  // transparent body is a divergence from the raw list (OXIDIZED_DESIGN_DIVERGENCES #341).
  DefEnvironment!("{adjustwidth} Undigested Undigested", "#body",
    mode => "internal_vertical", before_digest_end => { leave_horizontal()?; });
  DefEnvironment!("{adjustwidth*} Undigested Undigested", "#body",
    mode => "internal_vertical", before_digest_end => { leave_horizontal()?; });
});
