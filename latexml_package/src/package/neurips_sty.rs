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
    state::assign_value("neurips_final", Stored::from(1), Some(Scope::Global));
  });
  DeclareOption!("preprint", {
    state::assign_value("neurips_preprint", Stored::from(1), Some(Scope::Global));
  });
  DeclareOption!("nonatbib", {
    state::assign_value("neurips_nonatbib", Stored::from(1), Some(Scope::Global));
  });
  ProcessOptions!();
  if state::with_value("neurips_nonatbib", |v| v.is_none()) {
    RequirePackage!("natbib");
  }
  DefMacro!("\\AND",                                   "");
  DefMacro!("\\And",                                   "");
  DefMacro!("\\bottomfraction",                        "");
  DefMacro!("\\patchAmsMathEnvironmentForLineno",      "");
  DefMacro!("\\patchBothAmsMathEnvironmentsForLineno", "");
  // Perl L37: DefMacroI('\subsubsubsection', …, locked => 1). The lock
  // prevents well-meaning user-level \renewcommand{\subsubsubsection}{…}
  // from clobbering the @startsection trampoline.
  DefMacro!("\\subsubsubsection",
    "\\@startsection{subsubsubsection}{4}{}{}{}{}",
    locked => true);
  DefMacro!("\\textfraction", "");
  DefMacro!("\\topfraction",  "");
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
    before_digest => { gullet::unread_one(T_CS!("\\acksection")); });

  // {hide} environment — Perl L59
  DefEnvironment!("{hide}", "");

  // Theorem-likes — neurips_2024.sty L451-460 (and similar in 2022-2025).
  // Real templates define a `theorem` counter and a small set of named
  // envs sharing/cascading it. Mirror that defensively so neurips papers
  // that use `\begin{theorem}…\end{theorem}` without a manual
  // `\newtheorem` block render cleanly. Witness 2406.18814.
  RawTeX!(
    r"\newtheorem{theorem}{Theorem}[section]
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{propo}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{remark}[theorem]{Remark}
\newtheorem{example}[theorem]{Example}
\newtheorem{claim}[theorem]{Claim}
\newtheorem{assumption}[theorem]{Assumption}
\newtheorem{question}[theorem]{Question}
\newtheorem{problem}[theorem]{Problem}
\newtheorem{result}[theorem]{Result}"
  );
});
