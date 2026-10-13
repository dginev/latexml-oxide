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
  // `\newlength`s, i.e. skips (titling.sty:76-77, :112): `\setlength` scans by
  // the register's type (OXIDIZED_DESIGN #317).
  DefRegister!("\\droptitle" =>       Glue::new(0));
  DefRegister!("\\thanksmarkwidth" => Glue!("1.8em"));
  DefRegister!("\\thanksmargin" =>    Glue!("-1.8em"));
  // titling.sty:245-268 appends to `\title`, `\author` and `\date` the setting of `\thetitle`, `\theauthor` and
  // `\thedate`: the argument expanded once (`\protected@xdef`) with its `\thanks` dropped and `\and` as a comma, so
  // they exist from the call on (a custom title page's `\LARGE\thetitle` before or without `\maketitle`). The kernel
  // `\title` and `\author` are locked; their after-slots take the appended code. As in titling, nothing defines them
  // before the call, so a paper's own `\thetitle` stands until its `\title`, and `\title{\thetitle}` takes that
  // definition's text instead of looping. Perl (titling.sty.ltxml:85) `\let`s them to `\@title` at `\maketitle`
  // instead, a loop for `\title{\thetitle}` (KNOWN_PERL_ERRORS #636). Run 340: 62 first errors (witnesses
  // 1904.02630, 2008.03410, 2008.01842).
  RawTeX!(r"\def\lx@titling@nothanks{\def\thanks##1{}\def\thanksmark##1{}\def\thanksgap##1{}}
\def\lx@titling@title#1{\begingroup\lx@titling@nothanks\protected@xdef\thetitle{#1}\endgroup}
\def\lx@titling@author#1{\begingroup\lx@titling@nothanks\def\and{\unskip, }\protected@xdef\theauthor{#1}\endgroup}
\def\lx@titling@date#1{\begingroup\lx@titling@nothanks\protected@xdef\thedate{#1}\endgroup}
\let\lx@titling@title@slot\lx@title@after
\def\lx@title@after#1{\lx@titling@title@slot{#1}\gdef\lx@titling@fired@title{}\lx@titling@title{#1}}
\let\lx@titling@author@slot\lx@author@after
\def\lx@author@after#1{\lx@titling@author@slot{#1}\gdef\lx@titling@fired@author{}\lx@titling@author{#1}}
\let\lx@titling@date@slot\lx@date@after
\def\lx@date@after#1{\lx@titling@date@slot{#1}\gdef\lx@titling@fired@date{}\lx@titling@date{#1}}");
  // A class or package binding whose own setter reaches no slot (titling's `\appendiargdef` wraps whatever setter
  // exists when it loads; amsart, mnras, IEEEtran) has the name set from what the setter stored (`\@title`,
  // `\@author`, `\@date`) at `\begin{document}` and again at `\maketitle` (amsart's `\title`/`\author` go in the
  // body), replacing a class's placeholder (IEEEtran's no-op `\theauthor`); nothing stored, an undefined name is
  // defined empty, as Perl and the kernel before this left it after `\maketitle`. A name whose slot fired is the
  // slot's; the `\maketitle` run acts only on a field changed since `\begin{document}` (`\lx@titling@seen@<x>`), so a
  // paper's own `\thetitle` — with no `\title`, or redefined in the body — is kept.
  RawTeX!(r"\def\lx@titling@fallback#1#2{\@ifundefined{lx@titling@fired@#2}{%
\expandafter\ifx\csname @#2\expandafter\endcsname\csname lx@titling@seen@#2\endcsname\else
\global\expandafter\let\csname lx@titling@seen@#2\expandafter\endcsname\csname @#2\endcsname
\protected@edef\lx@titling@tmp{\csname @#2\endcsname}%
\ifx\lx@titling@tmp\@empty\@ifundefined{#1}{\global\expandafter\let\csname #1\endcsname\@empty}{}%
\else\csname lx@titling@#2\endcsname{\csname @#2\endcsname}\fi\fi}{}}
\def\lx@titling@fallbacks{\lx@titling@fallback{thetitle}{title}\lx@titling@fallback{theauthor}{author}%
\lx@titling@fallback{thedate}{date}}
\AtBeginDocument{\lx@titling@fallbacks}");
  AddToMacro!("\\lx@maketitle@begin", "\\lx@titling@fallbacks");
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
