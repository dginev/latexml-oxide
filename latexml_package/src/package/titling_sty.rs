use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\pretitle{}",   "\\def\\@bspretitle{#1}");
  DefMacro!("\\posttitle{}",  "\\def\\@bsposttitle{#1}");
  DefMacro!("\\preauthor{}",  "\\def\\@bspreauthor{#1}");
  DefMacro!("\\postauthor{}", "\\def\\@bspostauthor{#1}");
  DefMacro!("\\predate{}",    "\\def\\@bspredate{#1}");
  DefMacro!("\\postdate{}",   "\\def\\@bspostdate{#1}");
  def_macro_noop("\\maketitlehooka")?;
  def_macro_noop("\\maketitlehookb")?;
  def_macro_noop("\\maketitlehookc")?;
  def_macro_noop("\\maketitlehookd")?;
  def_macro_noop("\\thanksmarkseries{}")?;
  def_macro_noop("\\symbolthanksmark")?;
  def_macro_noop("\\@bscontmark")?;
  def_macro_noop("\\continuousmarks")?;
  def_macro_noop("\\thanksheadextra{}{}")?;
  def_macro_noop("\\thanksfootextra{}{}")?;
  DefMacro!("\\thanksmark{}",        "\\footnotemark[#1]");
  DefMacro!("\\thanksgap{}",         "\\hspace{#1}");
  DefMacro!("\\tamark",              "\\footnotemark");
  DefMacro!("\\thanksscript{}",      "\\textsuperscript{#1}");
  def_macro_noop("\\makethanksmarkhook")?;
  DefMacro!("\\thanksfootmark",      "\\tamark");
  DefMacro!("\\makethanksmark",      "\\thanksfootmark");
  def_macro_noop("\\usethanksrule")?;
  def_macro_noop("\\cancelthanksrule")?;
  def_macro_noop("\\calccentering{}{}")?;
  DefRegister!("\\droptitle" =>       Dimension::new(0));
  DefRegister!("\\thanksmarkwidth" => Dimension::from_str("1.8em")?);
  DefRegister!("\\thanksmargin" =>    Dimension::from_str("-1.8em")?);
  Let!("\\lx@titling@maketitle", "\\maketitle");
  DefMacro!("\\maketitle",
    "\\global\\let\\theauthor\\@author\\global\\let\\thedate\\@date\\global\\let\\thetitle\\@title\\lx@titling@maketitle");
  // titling.sty:166-180 — the title block titling's `\maketitle` typesets: each
  // field between its pre/post hooks. Perl's binding keeps the hooks but never
  // reads them (titling.sty.ltxml:28-33, "we'll just punt"), so a class that puts
  // its title page in them (lion-msc.cls:232-318: the thesis type, degree, major,
  // advisors, the abstract) lost it all. The kernel `\maketitle` deposits this
  // `\@maketitle` with the fields emptied (they are in the frontmatter) and keeps
  // it only when it typesets something, so the default hooks (layout only) add
  // nothing. The page layout around the block (`\newpage\null\vskip 2em
  // \vspace*{\droptitle}` … `\par\vskip 1.5em`) is left out: it typesets no
  // content, and `\newpage\null` would leave a page break and an empty
  // paragraph. Surpass; OXIDIZED_DESIGN #275.
  DefMacro!("\\@maketitle",
    "\\maketitlehooka{\\@bspretitle \\@title \\@bsposttitle}\\maketitlehookb{\\@bspreauthor \\@author \\@bspostauthor}\\maketitlehookc{\\@bspredate \\@date \\@bspostdate}\\maketitlehookd\\par");
  // Default formatting — Perl L57-64
  RawTeX!("\\pretitle{\\begin{center}\\LARGE}");
  RawTeX!("\\posttitle{\\par\\end{center}\\vskip 0.5em}");
  RawTeX!("\\preauthor{\\begin{center}\\large\\lineskip 0.5em\\begin{tabular}[t]{c}}");
  RawTeX!("\\postauthor{\\end{tabular}\\end{center}}");
  RawTeX!("\\predate{\\begin{center}\\large}");
  RawTeX!("\\postdate{\\par\\end{center}}");

  // Titling page environment — Perl L88. Preserve body so author-
  // typed alternate-title-page content (institution, abstract,
  // dedication, etc.) reaches the XML (content-preserving). The
  // previous empty replacement silently dropped the entire page.
  DefEnvironment!("{titlingpage}", "#body");

  def_macro_noop("\\killtitle")?;
  def_macro_noop("\\keepthetitle")?;
  def_macro_noop("\\emptythanks")?;
  def_macro_noop("\\@bsmtitlempty")?;
  def_macro_noop("\\appendiargdef{}{}")?;
});
