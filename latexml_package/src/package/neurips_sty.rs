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
  // neurips_2026.sty L72 adds \newif\if@anonymous\@anonymoustrue (toggled false by
  // preprint/final). The binding intercepts the versioned name and never creates it,
  // so papers copying the style's \@maketitle (which branches on \if@anonymous) hit
  // Error:undefined:\if@anonymous — a Rust-only divergence: Perl 0.8.8 converts the
  // same paper without it. Default false = authors shown (correct for preprint/final
  // arXiv uploads). Witness 2605.17249.
  DefConditional!("\\if@anonymous");
  // The style's year, from the name it was requested as: neurips_YYYY.sty (2026: :105-106) names the (YYYY-1986)th
  // conference, and its notice, tracks and location follow that year's style. A versioned request under a directory
  // loads through the fallback, which records it (`Styles/neurips_2026`, 2609.20831). The bare `neurips` name keeps
  // the values Perl's neurips.sty.ltxml sets (2022's).
  let fallback_request = lookup_string("fallback_request");
  let request = match fallback_request.rsplit('/').next() {
    Some(name) if name.starts_with("neurips") => fallback_request.clone(),
    _ => do_expand(T_CS!("\\@currname"))?.to_string(),
  };
  // `neurips_2025_custom` is 2025's.
  let year = request
    .rsplit('/')
    .next()
    .and_then(|name| name.strip_prefix("neurips_"))
    .map(|name| name.chars().take_while(char::is_ascii_digit).collect::<String>())
    .and_then(|year| year.parse::<u32>().ok())
    .filter(|year| *year > 1986);
  let (ordinal, year_text) = match year {
    Some(year) => {
      let n = year - 1986;
      let suffix = match (n % 100, n % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
      };
      (format!("{n}{suffix}"), year.to_string())
    },
    None => (s!("36th"), s!("2022")),
  };
  raw_tex(&format!(r"\def\@neuripsordinal{{{ordinal}}}\def\@neuripsyear{{{year_text}}}"))?;
  // neurips_2016.sty:39 … neurips_2026.sty:107; Perl's binding: New Orleans. (2016-2018 shipped as `nips_YYYY.sty`, a name
  // this binding does not answer to; a copy renamed `neurips_YYYY` reads them.)
  let location = match year {
    Some(2016) => "Barcelona, Spain",
    Some(2017) => "Long Beach, CA, USA",
    Some(2018) => "Montr\\'eal, Canada",
    Some(2019 | 2020) => "Vancouver, Canada",
    Some(2021) => "Sydney, Australia",
    Some(2024) => "Vancouver, USA",
    Some(2025) => "San Diego",
    Some(2026) => "Sydney",
    _ => "New Orleans",
  };
  raw_tex(&format!(r"\def\@neuripslocation{{{location}}}"))?;
  // neurips_2025.sty:47-86, neurips_2026.sty:47-93: the camera-ready tracks, each an option setting its conditional
  // (main is the default) and naming the track in `\@trackname` — 2025's after "(NeurIPS 2025)", 2026's after a
  // period — and the workshop tracks' `\workshoptitle` (:81-82), whose name that track name carries. Papers copying
  // the style's `\@maketitle` branch on the conditionals and print `\@noticestring`, in camera-ready mode the
  // `\@trackname` (2026: :391-403); 113 2609 papers call `\workshoptitle` (2609.04556, 2609.00038).
  RawTeX!(r"\newif\if@neuripsfinal\newif\if@natbib\@natbibtrue\newif\if@main\@maintrue\newif\if@position
\newif\if@dandb\newif\if@eandd\newif\if@creativeai\newif\if@education\newif\if@workshop");
  if year.is_none_or(|year| year >= 2025) {
    RawTeX!(r"\newcommand{\@workshoptitle}{}\newcommand{\workshoptitle}[1]{\renewcommand{\@workshoptitle}{#1}}");
  }
  let track_after_year = if year.is_some_and(|year| year >= 2026) { ". " } else { " " };
  let define_trackname = move |track: &str| {
    let track = if track.is_empty() { s!(".") } else { s!("{track_after_year}{track}") };
    raw_tex(&format!(
      r"\newcommand{{\@trackname}}{{\@neuripsordinal\ Conference on Neural Information Processing Systems (NeurIPS \@neuripsyear){track}}}"
    ))
  };
  DeclareOption!("main", {
    Let!("\\if@main", "\\iftrue");
    define_trackname("")?;
  });
  DeclareOption!("position", {
    Let!("\\if@position", "\\iftrue");
    define_trackname("Position Paper Track.")?;
  });
  DeclareOption!("dandb", {
    Let!("\\if@dandb", "\\iftrue");
    Let!("\\if@anonymous", "\\iffalse");
    define_trackname("Track on Datasets and Benchmarks.")?;
  });
  DeclareOption!("eandd", {
    Let!("\\if@eandd", "\\iftrue");
    define_trackname("Track on Evaluations and Datasets.")?;
  });
  DeclareOption!("creativeai", {
    Let!("\\if@creativeai", "\\iftrue");
    Let!("\\if@anonymous", "\\iffalse");
    define_trackname("Creative AI Track.")?;
  });
  DeclareOption!("education", {
    Let!("\\if@education", "\\iftrue");
    Let!("\\if@anonymous", "\\iffalse");
    define_trackname("Education Track.")?;
  });
  DeclareOption!("sglblindworkshop", {
    Let!("\\if@workshop", "\\iftrue");
    Let!("\\if@anonymous", "\\iffalse");
    define_trackname(r"Workshop: \@workshoptitle.")?;
  });
  DeclareOption!("dblblindworkshop", {
    Let!("\\if@workshop", "\\iftrue");
    define_trackname(r"Workshop: \@workshoptitle.")?;
  });
  DeclareOption!("nonanonymous", { Let!("\\if@anonymous", "\\iffalse"); });
  // neurips_2026.sty:231-232: the caption skips the style sets `\abovecaptionskip`/`\belowcaptionskip` from
  // (2609.20831, 2609.29545 read them).
  RawTeX!(r"\newlength{\@neuripsabovecaptionskip}\setlength{\@neuripsabovecaptionskip}{7\p@}\newlength{\@neuripsbelowcaptionskip}\setlength{\@neuripsbelowcaptionskip}{\z@}");
  DeclareOption!("final", {
    assign_value("neurips_final", Stored::from(1), Some(Scope::Global));
    Let!("\\if@neuripsfinal", "\\iftrue");
  });
  DeclareOption!("preprint", {
    assign_value("neurips_preprint", Stored::from(1), Some(Scope::Global));
    Let!("\\if@preprint", "\\iftrue");
    Let!("\\if@anonymous", "\\iffalse");
  });
  DeclareOption!("nonatbib", {
    assign_value("neurips_nonatbib", Stored::from(1), Some(Scope::Global));
    Let!("\\if@natbib", "\\iffalse");
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
  // The style's `\@noticestring` (neurips_2026.sty:391-403; 2019: :41-55), for papers whose own `\@maketitle` prints
  // it: "Preprint." (2020-2024: "Preprint. Under review."), in camera-ready mode the venue — from 2025 the track's
  // `\@trackname`, before the conference (2016-2021 with its location) — and otherwise "Submitted to … Do not
  // distribute.". No venue note of its own: the paper's copy of the style decides what its first page prints, and
  // copies empty, edit or drop the notice (DIVERGENCES #448).
  let series = if year.is_some_and(|year| year < 2018) { "NIPS" } else { "NeurIPS" };
  let notice = if with_value("neurips_preprint", |v| v.is_some()) {
    if year.is_some_and(|year| (2020..=2024).contains(&year)) { "Preprint. Under review." } else { "Preprint." }.to_string()
  } else if with_value("neurips_final", |v| v.is_some()) {
    match year {
      Some(year) if year >= 2025 => s!(r"\@trackname"),
      Some(year) if year <= 2021 => s!(
        r"\@neuripsordinal\/ Conference on Neural Information Processing Systems ({series} \@neuripsyear), \@neuripslocation."
      ),
      _ => s!(r"\@neuripsordinal\/ Conference on Neural Information Processing Systems ({series} \@neuripsyear)."),
    }
  } else {
    s!(r"Submitted to \@neuripsordinal\/ Conference on Neural Information Processing Systems ({series} \@neuripsyear). Do not distribute.")
  };
  raw_tex(&format!(r"\def\@noticestring{{{notice}}}"))?;
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

  // {hide} environment. Perl (neurips.sty.ltxml L59) defines it
  // UNCONDITIONALLY, but the raw neurips_2023.sty (L336-390) only runs
  // `\NewEnviron{hide}{}` in the SUBMISSION branch —
  // `\if@preprint … \else \if@neuripsfinal … \else <here> \fi \fi` — so in
  // preprint/final mode `\hide` is left undefined. That matters: papers define
  // their own `\newcommand{\hide}[1]{}` (a brace-gobbling "comment this out"
  // helper) and use it as `\hide{ … }`. Defining `{hide}` unconditionally
  // shadows that `\newcommand` (silently ignored as a redefinition), so `\hide{`
  // is parsed as the environment opener and runs away to `\end{document}`
  // looking for `\endhide` — swallowing the whole body (everything after the
  // abstract). Gate on submission mode (neither preprint nor final) to match the
  // real class. SURPASSES Perl, which drops the body identically here.
  // arXiv/html_feedback#861, witness 2403.15796.
  if with_value("neurips_preprint", |v| v.is_none())
    && with_value("neurips_final", |v| v.is_none())
  {
    DefEnvironment!("{hide}", "");
  }

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
