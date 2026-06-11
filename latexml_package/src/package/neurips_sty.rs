use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("geometry");
  RequirePackage!("lineno");
  // neurips_2025.sty L39 defines \newif\if@preprint. Our binding
  // intercepts \DeclareOption{preprint} and never actually creates
  // the conditional. Provide it defensively so user code that does
  // \if@preprint ... \fi outside the preamble works.
  // Witness 2406.00153 (neurips_2025).
  DefConditional!("\\if@preprint");
  DefConditional!("\\if@submission");
  DefConditional!("\\if@final");
  DeclareOption!("final", {
    assign_value("neurips_final", Stored::from(1), Some(Scope::Global));
  });
  DeclareOption!("preprint", {
    assign_value("neurips_preprint", Stored::from(1), Some(Scope::Global));
  });
  DeclareOption!("nonatbib", {
    assign_value("neurips_nonatbib", Stored::from(1), Some(Scope::Global));
  });
  ProcessOptions!();
  if with_value("neurips_nonatbib", |v| v.is_none()) {
    RequirePackage!("natbib");
  }
  def_macro_noop("\\AND")?;
  def_macro_noop("\\And")?;
  def_macro_noop("\\bottomfraction")?;
  // neurips_*.sty L301/307: \@toptitlebar / \@bottomtitlebar draw the
  // decorative \hrule + \vskip box around the title — purely visual, moot in
  // our XML paradigm (WISDOM #50). Our binding intercepts neurips_*.sty (so the
  // real raw defs never run), and downstream styles build their own title using
  // them — e.g. the bundled `arxiv.sty` `\@maketitle`:
  // `\@toptitlebar{\Large\bf #1}\@bottomtitlebar`. Provide 0-arg no-ops so the
  // title text survives and `\maketitle` doesn't hit undefined-CS errors.
  // Witness arXiv:2007.04825 (`\usepackage{arxiv}` → neurips_2020 title bars).
  def_macro_noop("\\@toptitlebar")?;
  def_macro_noop("\\@bottomtitlebar")?;
  def_macro_noop("\\patchAmsMathEnvironmentForLineno")?;
  def_macro_noop("\\patchBothAmsMathEnvironmentsForLineno")?;
  // Perl L37: DefMacroI('\subsubsubsection', …, locked => 1). The lock
  // prevents well-meaning user-level \renewcommand{\subsubsubsection}{…}
  // from clobbering the @startsection trampoline.
  DefMacro!("\\subsubsubsection",
    "\\@startsection{subsubsubsection}{4}{}{}{}{}",
    locked => true);
  def_macro_noop("\\textfraction")?;
  def_macro_noop("\\topfraction")?;
  DefMacro!("\\@neuripsordinal",  "36th");
  DefMacro!("\\@neuripsyear",     "2022");
  DefMacro!("\\@neuripslocation", "New Orleans");
  DefMacro!("\\acksection", "\\section*{Acknowledgments and Disclosure of Funding}");
  DefMacro!("\\answerYes[]",  "\\textcolor{blue}{[Yes] #1}");
  DefMacro!("\\answerNo[]",   "\\textcolor{orange}{[No] #1}");
  DefMacro!("\\answerNA[]",   "\\textcolor{gray}{[N/A] #1}");
  DefMacro!("\\answerTODO[]", "\\textcolor{red}{\\bf [TODO]}");

  // {ack} environment — Perl L51-52 unreads `\acksection` before the body
  // digests so the "Acknowledgments and Disclosure of Funding" title
  // header fires without the author having to write it. Without the
  // unread, `\begin{ack}…\end{ack}` produces a bare body block with no
  // heading.
  DefEnvironment!("{ack}", "#body",
    before_digest => { unread_one(T_CS!("\\acksection")); });

  // {hide} environment — Perl L59
  DefEnvironment!("{hide}", "");

  // Theorem-likes — neurips_2024.sty L451-460 (and similar in 2022-2025).
  // Real templates define a `theorem` counter and a small set of named
  // envs sharing/cascading it. Mirror that defensively so neurips papers
  // that use `\begin{theorem}…\end{theorem}` without a manual
  // `\newtheorem` block render cleanly. Witness 2406.18814.
  //
  // \AtBeginDocument-defer + \@ifundefined-guard: defer until after the
  // user preamble runs, so a user-provided helper (e.g. mymath.sty doing
  // `\ifx\lemma\undefined \newtheorem{lemma} \newtheorem*{lemma*} \fi`)
  // wins. Without deferral our unconditional defs run at .sty-load time,
  // pre-define `\lemma`, and silently suppress the user's `\newtheorem*
  // {lemma*}` branch. Witness 2305.11788 (neurips paper + mymath.sty).
  RawTeX!(
    r"\AtBeginDocument{%
\@ifundefined{theorem}{\newtheorem{theorem}{Theorem}[section]}{}%
\@ifundefined{lemma}{\newtheorem{lemma}[theorem]{Lemma}}{}%
\@ifundefined{corollary}{\newtheorem{corollary}[theorem]{Corollary}}{}%
\@ifundefined{proposition}{\newtheorem{proposition}[theorem]{Proposition}}{}%
\@ifundefined{propo}{\newtheorem{propo}[theorem]{Proposition}}{}%
\@ifundefined{definition}{\newtheorem{definition}[theorem]{Definition}}{}%
\@ifundefined{remark}{\newtheorem{remark}[theorem]{Remark}}{}%
\@ifundefined{example}{\newtheorem{example}[theorem]{Example}}{}%
\@ifundefined{claim}{\newtheorem{claim}[theorem]{Claim}}{}%
\@ifundefined{assumption}{\newtheorem{assumption}[theorem]{Assumption}}{}%
\@ifundefined{question}{\newtheorem{question}[theorem]{Question}}{}%
\@ifundefined{problem}{\newtheorem{problem}[theorem]{Problem}}{}%
\@ifundefined{result}{\newtheorem{result}[theorem]{Result}}{}}"
  );
});
