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
  DefRegister!("\\thanksmarkwidth" => Dimension("1.8em"));
  DefRegister!("\\thanksmargin" =>    Dimension("-1.8em"));
  Let!("\\lx@titling@maketitle", "\\maketitle");
  DefMacro!("\\maketitle",
    "\\global\\let\\theauthor\\@author\\global\\let\\thedate\\@date\\global\\let\\thetitle\\@title\\lx@titling@maketitle");
  DefMacro!("\\killtitle",         "");
  DefMacro!("\\keepthetitle",      "");
  DefMacro!("\\emptythanks",       "");
});
