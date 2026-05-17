use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\pretitle{}",   "\\def\\@bspretitle{#1}");
  DefMacro!("\\posttitle{}",  "\\def\\@bsposttitle{#1}");
  DefMacro!("\\preauthor{}",  "\\def\\@bspreauthor{#1}");
  DefMacro!("\\postauthor{}", "\\def\\@bspostauthor{#1}");
  DefMacro!("\\predate{}",    "\\def\\@bspredate{#1}");
  DefMacro!("\\postdate{}",   "\\def\\@bspostdate{#1}");
  DefMacro!("\\maketitlehooka",   "");
  DefMacro!("\\maketitlehookb",   "");
  DefMacro!("\\maketitlehookc",   "");
  DefMacro!("\\maketitlehookd",   "");
  DefMacro!("\\thanksmarkseries{}",  "");
  DefMacro!("\\symbolthanksmark",    "");
  DefMacro!("\\@bscontmark",         "");
  DefMacro!("\\continuousmarks",     "");
  DefMacro!("\\thanksheadextra{}{}", "");
  DefMacro!("\\thanksfootextra{}{}", "");
  DefMacro!("\\thanksmark{}",        "\\footnotemark[#1]");
  DefMacro!("\\thanksgap{}",         "\\hspace{#1}");
  DefMacro!("\\tamark",              "\\footnotemark");
  DefMacro!("\\thanksscript{}",      "\\textsuperscript{#1}");
  DefMacro!("\\makethanksmarkhook",  "");
  DefMacro!("\\thanksfootmark",      "\\tamark");
  DefMacro!("\\makethanksmark",      "\\thanksfootmark");
  DefMacro!("\\usethanksrule",       "");
  DefMacro!("\\cancelthanksrule",    "");
  DefMacro!("\\calccentering{}{}",   "");
  DefRegister!("\\droptitle" =>       Dimension::new(0));
  DefRegister!("\\thanksmarkwidth" => Dimension::from_str("1.8em")?);
  DefRegister!("\\thanksmargin" =>    Dimension::from_str("-1.8em")?);
  Let!("\\lx@titling@maketitle", "\\maketitle");
  DefMacro!("\\maketitle",
    "\\global\\let\\theauthor\\@author\\global\\let\\thedate\\@date\\global\\let\\thetitle\\@title\\lx@titling@maketitle");
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

  DefMacro!("\\killtitle",         "");
  DefMacro!("\\keepthetitle",      "");
  DefMacro!("\\emptythanks",       "");
  DefMacro!("\\@bsmtitlempty",     "");
  DefMacro!("\\appendiargdef{}{}", "");
});
