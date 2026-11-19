//! The PASJ classes (Publications of the Astronomical Society of Japan) — `pasj00`, `pasj01`, `pasj02`: shipped with
//! every paper, not in TeX Live. The shipped class is interpreted raw (in every profile), for its own macros, and what
//! LaTeXML builds semantically is put back over it: the kernel `\caption`, and the frontmatter its title page only
//! typesets.
use latexml_package::prelude::*;

use crate::and_by_meaning::and_runs;

/// Interpret the shipped class `name` raw, then restore the semantic definitions over it:
/// - the kernel `\caption` (`\lx@caption`): PASJ's own (pasj00.cls:814-835, pasj01.cls:898-919, pasj02.cls:887-908)
///   steps the counter and calls the class's `\@makecaption` directly, bypassing `\@caption`, the one path that builds
///   `<caption>`, so its captions were paragraphs without their number and their labels dangled (1310.7069,
///   1505.02769, 2504.06663);
/// - `\@maketitle` empty: everything its title page lays out (pasj01.cls:1912-1950) is in the frontmatter, and the
///   deposit of the rest left an empty "[ Received ; Accepted ]" frame in the body (1505.02769, 2005.13750).
///
/// A paper whose source lacks the class falls to OmniBus, as before the binding, with the class's own `{longtable}`
/// ([`pasj_unshipped_longtable`]) and the class's own commands papers call ([`pasj_unshipped_macros`]). Either way the
/// frontmatter macros the classes store for `\maketitle` to typeset (pasj01.cls:84-159) are the frontmatter's: they were
/// lost under OmniBus and raw alike.
pub(crate) fn pasj_class(name: &str) -> Result<()> {
  let shipped = find_file(
    name,
    Some(FindFileOptions {
      ext_type: Some(Cow::Borrowed("cls")),
      forbid_ltxml: true,
      ..Default::default()
    }),
  )
  .is_some();
  if shipped {
    InputDefinitions!(name, noltxml => true, extension => Some(Cow::Borrowed("cls")));
    Let!("\\caption", "\\lx@caption");
    Let!("\\@maketitle", "\\@empty");
    // pasj00.cls:4025-4035, pasj01.cls:4098-4110 ("%%% fleqn"): `\equation` is fleqn.clo's flush-left `\hbox to
    // \linewidth\bgroup $\m@th\displaystyle`, which the raw class cannot give the kernel's `{equation}`, so its hook is
    // set here (latexml_package fleqn_sty.rs `math_displays_in_hbox`): an author's `$` in it is text in the math
    // (2311.01721; KNOWN_PERL_ERRORS #907). pasj02.cls:4070-4083 leaves `\equation` commented out.
    if crate::shipped_class::shipped_class_text(name)?
      .is_some_and(|text| crate::shipped_class::class_code_contains(&text, "\\def\\equation{"))
    {
      fleqn_sty::math_displays_in_hbox()?;
    }
  } else {
    LoadClass!("OmniBus");
    pasj_unshipped_longtable()?;
    pasj_unshipped_macros()?;
    pasj_unshipped_timeform()?;
  }
  pasj_frontmatter()
}

/// The class's own commands a paper whose source lacks the class calls, which OmniBus does not define (identical in
/// pasj00.cls of 0704.3654, pasj01.cls of 1505.02769 and pasj02.cls of 2504.06663):
/// - `\SetRunningHead` (pasj00.cls:52-54, pasj01.cls:68-70, pasj02.cls:53-55) stores the running head, no content;
/// - `\FigureFile(w,h){file}` and `\RelativeFigurePath` (pasj00.cls:4122-4164, pasj01.cls:4179-4221,
///   pasj02.cls:4174-4216): the file, under the figure path, at the width `w` (in mm when it has no unit), through
///   graphicx's `\includegraphics`; the class's `\vtop` and `\kern` around it are print layout, and its framed
///   file name for a missing file is the binding's missing-graphic warning;
/// - amssymb for `\lesssim`/`\gtrsim` (the class's `useamsfonts` branch, pasj00.cls:3445-3446; its other branch
///   composes the same relations), and the solar symbols `\Sol`, `\solar`, `\MO`, `\LO`, `\RO` (pasj00.cls:2303,
///   3433-3444) with `\odot`, the glyph the class's `\@solar` takes in script styles;
/// - `\authorcite` and `\yearcite` (pasj00.cls:1636-1637, pasj01.cls:1704-1705, pasj02.cls:1695-1696), the class's
///   author-only and year-only citations: natbib's `\citeauthor` and `\citeyear`, which OmniBus autoloads.
///
/// Without them astro-ph0108490, astro-ph0201077 and astro-ph0204110 had an undefined-macro error per command, their
/// figures were the error's text, and each `\authorcite{M31_Makishima}` printed its key with an Error per `_`
/// (astro-ph0108490). Guard `perfect_kernel_batchclustfix::pasj00_without_its_class_keeps_its_figures`.
fn pasj_unshipped_macros() -> Result<()> {
  RequirePackage!("amssymb");
  RawTeX!(
    r"\def\SetRunningHead#1#2{\def\@Author{#1}\def\@runninghead{#2}}
\def\RelativeFigurePath#1{\def\@figurepath{#1}\ifx\@figurepath\@empty\def\@figurepath{.}\fi}
\RelativeFigurePath{}
\def\FigureFile{\@ifstar\@FigureFile\@FigureFile}
\def\@FigureFile(#1,#2)#3{\def\@figfilename{#3}\@onelevel@sanitize\@figfilename\begingroup
\@defaultunits\@tempdima#1mm\relax\@nnil\@tempdima.35146\@tempdima
\edef\@tempa{\noexpand\includegraphics[width=\strip@pt\@tempdima mm,keepaspectratio]{\@figurepath/\@figfilename}}%
\@tempa\endgroup}
\DeclareRobustCommand*\Sol{\ensuremath{\mathbin{\odot}}}
\let\solar\Sol
\DeclareRobustCommand*\MO{\relax\ifmmode\@math@sub@solar{M}\else\@text@sub@solar{M}\fi}
\let\Mo\MO
\DeclareRobustCommand*\LO{\relax\ifmmode\@math@sub@solar{L}\else\@text@sub@solar{L}\fi}
\DeclareRobustCommand*\RO{\relax\ifmmode\@math@sub@solar{R}\else\@text@sub@solar{R}\fi}
\def\@math@sub@solar#1{{{#1}_{\solar}}}
\def\@text@sub@solar#1{\ifvmode\leavevmode\fi\hbox{$\m@th{#1}_{\solar}$}}
\DeclareRobustCommand\authorcite{\citeauthor}
\DeclareRobustCommand\yearcite{\citeyear}"
  );
  Ok(())
}

/// The class's notation commands a paper whose source lacks the class calls, as pasj00.cls of 0704.3654 defines them,
/// verbatim: `\emissiontype{…}` (:3578-3598, the line's name set small after a thin space), the abbreviations
/// (:3684-3713: `\arcsec`, `\micron`, `\fp` … `\fdg` with the decimal point under a unit and the class's
/// `\ambiguouswarn` note), and `\timeform{…}`/`\TF{…}` (:3716-3980), whose parser sets the units of `12h34m56.7s` or
/// `2.5'` as superscripts with a decimal point under the last (2'.5); its `\@csc@t` and `\@@lineno` helpers are
/// :2169-2178. `\draft` (:284-300) and the size options `\tenpoint`/`\elevenpoint`/`\twelvepoint` (:236-283) set the
/// page layout, no content (empty). Without them astro-ph0201169,
/// astro-ph0204505, 0706.0253 (`\arcsec`, `\micron`, `\draft`) and 0708.1829 had an undefined-macro error per command
/// and lost the time or angle to its text.
/// Guard `perfect_kernel_batchclustfix2::pasj00_without_its_class_sets_timeform`.
fn pasj_unshipped_timeform() -> Result<()> {
  RawTeX!(
    r#"\def\@csc@t#1#2#3{%
   \expandafter\expandafter\expandafter\def
   \expandafter\expandafter\expandafter#1%
   \expandafter\expandafter\expandafter{\expandafter#2#3}}
\let\@@lineno\@empty
\expandafter\ifx\csname inputlineno\endcsname\relax\else
   \ifnum\inputlineno<\z@\else
      \def\@@lineno{l.\the\inputlineno\space}
   \fi
\fi
\DeclareRobustCommand*\emissiontype{%
   \relax
   \ifmmode \expandafter\@math@emissiontype
   \else    \expandafter\@text@emissiontype
   \fi}
\def\@text@emissiontype#1{%
   \ifvmode \leavevmode \fi
   \unskip\nobreak\,\nobreak
   \@@emissiontype\f@size{#1}}
\def\@math@emissiontype#1{%
   \nobreak\,\nobreak
   \mathchoice
      {\@@emissiontype\tf@size{#1}}%
      {\@@emissiontype\tf@size{#1}}%
      {\@@emissiontype\sf@size{#1}}%
      {\@@emissiontype\ssf@size{#1}}}
\def\@@emissiontype#1#2{%
   \hbox{\dimen@#1\p@ \dimen@.8\dimen@
      \ifdim\dimen@<4\p@ \dimen@4\p@ \fi
      \fontsize\dimen@\f@baselineskip\selectfont
      #2}}
\def\arcdeg{\ensuremath{{}^{\circ}}}
\let\degree\arcdeg
\def\arcmin{\ensuremath{{}^{\prime}}}
\def\arcsec{\ensuremath{{}^{\prime\prime}}}
\def\farcm@mss{\ensuremath{.\mkern-4mu{}^{\prime}}}%
\let\farcm\farcm@mss
\def\farcs@mss{\ensuremath{.\!\!{}^{\prime\prime}}}%
\let\farcs\farcs@mss
\def\fp{\ensuremath{.\!\!^{\scriptscriptstyle\mathrm{p}}}%
   \ambiguouswarn{fp}{timeform}}
\def\fd{\ensuremath{.\!\!^{\mathrm{d}}}\ambiguouswarn{fd}{timeform}}
\def\fh{\ensuremath{.\!\!^{\mathrm{h}}}\ambiguouswarn{fh}{timeform}}
\def\fm{\ensuremath{.\!\!^{\mathrm{m}}}\ambiguouswarn{fs}{timeform}}
\def\fs{\ensuremath{.\!\!^{\mathrm{s}}}\ambiguouswarn{fm}{timeform}}
\def\fdg{\ensuremath{.\!\!^{\circ}}\ambiguouswarn{fdg}{timeform}}
\def\ubvr{\mbox{$U\!BV\!R$}\ambiguouswarn{ubvr}{ensuremath{UBVR}}}
\def\ub{\mbox{$U\!-\!B$}\ambiguouswarn{ub}{ensuremath{U-B}}}
\def\bv{\mbox{$B\!-\!V$}\ambiguouswarn{bv}{ensuremath{B-V}}}
\def\vr{\mbox{$V\!-\!R$}\ambiguouswarn{vr}{ensuremath{V-R}}}
\def\ur{\mbox{$U\!-\!R$}\ambiguouswarn{ur}{ensuremath{U-R}}}
\def\onehalf{\ensuremath{\frac{1}{2}}}
\def\onethird{\ensuremath{\frac{1}{3}}}
\def\twothirds{\ensuremath{\frac{2}{3}}}
\def\onequarter{\ensuremath{\frac{1}{4}}}
\def\threequarters{\ensuremath{\frac{3}{4}}}
\def\micron{\ensuremath{\mu\mathrm{m}}}
\DeclareRobustCommand\ambiguouswarn[2]{%
   \typeout{\@@lineno Warning [pasj]: ^^J\@spaces
The use of ambiguous command `\@backslashchar#1' is not recommended.^^J%
\@spaces Use `\@backslashchar#2' in place of `\@backslashchar#1'.}}
\DeclareRobustCommand\timeform[1]{\@timeform\timeform{#1}}
\DeclareRobustCommand\TF[1]{\@timeform\TF{#1}}
\def\@timeform#1#2{{%
   \def\@timeform@csname{#1}%
   \begingroup
   \def\circ{D}\def\deg{D}\def\prime{'}%
   \let\rm\@empty \let\mathrm\@empty
   \let~\@empty
   \def\pm{\@other@tilde P}%
   \def\mp{\@other@tilde M}%
   \let\protect\noexpand
   \edef\@tempa{\def\noexpand\@tempb{#2}}%
   \expandafter\endgroup\@tempa
   \@onelevel@sanitize\@tempb
   \let\@tempa\@empty
   \expandafter\@timeform@preprocess\@tempb\@nil
   \let\@timeform@body\@empty
   \let\@timeform@prevdigits\@empty
   \let\@timeform@lastdigits\@empty
   \let\@timeform@lastunit\@empty
   \@chclass\m@ne
   \@lastchclass\m@ne
   \let\@templist\@empty
   \let\protect\@unexpandable@protect
   \expandafter\@timeform@loop@\@tempa\@nil
   \ifx\@timeform@lastunit\@empty \def\@timeform@lastunit{\ }\fi
   \let\protect\relax
   \@timeform@body}}
\def\@timeform@preprocess#1{%
   \def\@tempb{#1}%
   \ifx\@tempb\@nnil\else
      \if \@charlb#1\relax \let\@tempb\@empty \fi
      \if \@charrb#1\relax \let\@tempb\@empty \fi
      \if ^#1\relax        \let\@tempb\@empty \fi
      \edef\@tempa{\@tempa\@tempb}%
      \expandafter\@timeform@preprocess
   \fi}
\begingroup
\catcode`\~=12
\def\x{\def\@other@tilde{~}}
\expandafter\endgroup\x
\def\@timeform@loop@#1{%
   \ifnum\@chclass=\z@ \ifx\@currchar\@empty\else
      \typeout{%
         \@@lineno Warning [pasj]: Invalid character `\@currchar' in
         \expandafter\string\@timeform@csname.}%
   \fi\fi
   \def\@currchar{#1}%
   \@timeform@check@char
   \@timeform@loop}
\def\@timeform@loop{%
   \ifnum\@chclass<\z@
      \ifx\@timeform@prevdigits\@empty
         \let\@tempa\@timeform@lastdigits
      \else
         \toks@\expandafter{\@timeform@lastunit}%
         \edef\@tempa{%
            \@timeform@prevdigits
            \noexpand\@timeform@unit{\the\toks@}%
            \@timeform@lastdigits}%
      \fi
      \@csc@t\@timeform@body\@timeform@body\@tempa
   \else
      \expandafter\expandafter
      \csname @timeform@loop@\romannumeral\@chclass\endcsname
   \fi}
\def\@timeform@check@char{%
   \ifx\@currchar\@nnil
      \@chclass\m@ne
   \else
      \expandafter\count@\expandafter=\expandafter`\@currchar\relax
      \@chclass\z@
      \ifnum`0>\count@\else \ifnum`9<\count@\else
         \@chclass\@ne
      \fi\fi
      \if +\@currchar\relax
         \@chclass\@ne
         \def\@currchar{\protect\ensuremath{+}}%
      \fi
      \if -\@currchar\relax
         \@chclass\@ne
         \def\@currchar{\protect\ensuremath{-}}%
      \fi
      \if D\@currchar\relax \@chclass\tw@ \def\@currchar{\circ}\fi
      \if d\@currchar\relax \@chclass\tw@ \fi
      \if h\@currchar\relax \@chclass\tw@ \fi
      \if m\@currchar\relax \@chclass\tw@ \fi
      \if p\@currchar\relax
         \@chclass\tw@
         \def\@currchar{\scriptscriptstyle p}%
      \fi
      \if s\@currchar\relax \@chclass\tw@ \fi
      \if '\@currchar\relax \@chclass\thr@@ \fi
      \if "\@currchar\relax \@chclass\thr@@ \fi
      \if .\@currchar\relax \@chclass4\relax \fi
      \if \@other@tilde\@currchar\relax \@chclass5\relax \fi
   \fi}
\def\@timeform@loop@i#1{%
   \edef\@templist{\@templist\@currchar}%
   \def\@currchar{#1}%
   \@timeform@check@char
   \ifnum\@chclass<\z@
      \ifx\@timeform@lastdigits\@empty
         \let\@tempa\@templist
      \else
         \toks@\expandafter{\@timeform@lastunit}%
         \edef\@tempa{%
            \@timeform@lastdigits
            \noexpand\@timeform@unit{\the\toks@}%
            \@templist}%
      \fi
      \@csc@t\@timeform@body\@timeform@body\@tempa
      \let\@timeform@next\relax
   \else
      \ifnum\@chclass=\@ne
         \def\@timeform@next{\@timeform@loop@i}%
      \else
         \@lastchclass\@ne
         \ifnum 5=\@chclass\else
            \let\@timeform@prevdigits\@timeform@lastdigits
            \let\@timeform@lastdigits\@templist
            \let\@templist\@empty
         \fi
         \def\@timeform@next{\@timeform@loop}%
      \fi
   \fi
   \@timeform@next}
\def\@timeform@loop@ii#1{%
   \ifx\@timeform@lastunit\@empty\else
      \ifnum\@lastchclass=\tw@
         \toks@\expandafter{\@timeform@lastunit}%
         \edef\@tempa{%
            \@timeform@lastdigits
            \noexpand\@timeform@unit{\the\toks@}}%
         \let\@timeform@lastdigits\@empty
      \else
         \toks@\expandafter{\@timeform@lastunit}%
         \edef\@tempa{%
            \@timeform@prevdigits
            \noexpand\@timeform@unit{\the\toks@}}%
         \let\@timeform@prevdigits\@empty
      \fi
      \@csc@t\@timeform@body\@timeform@body\@tempa
   \fi
   \let\@timeform@lastunit\@currchar
   \def\@currchar{#1}%
   \@timeform@check@char
   \ifnum\@chclass<\z@
      \edef\@tempa{%
         \@timeform@lastdigits
         \noexpand\@timeform@unit{\noexpand\@timeform@lastunit}}%
      \@csc@t\@timeform@body\@timeform@body\@tempa
   \else
      \@lastchclass\tw@
      \expandafter\@timeform@loop
   \fi}
\def\@timeform@loop@iii#1{%
   \ifnum\@lastchclass=\thr@@
      \if\@currchar'\relax \def\@tempa{\prime}%
      \else                \def\@tempa{\prime\prime}%
      \fi
      \@csc@t\@templist\@templist\@tempa
   \else
      \ifnum\@lastchclass=\tw@
         \toks@\expandafter{\@timeform@lastunit}%
         \edef\@tempa{%
            \@timeform@lastdigits
            \noexpand\@timeform@unit{\the\toks@}}%
         \let\@timeform@lastdigits\@empty
         \let\@timeform@lastunit\@empty
         \@csc@t\@timeform@body\@timeform@body\@tempa
      \fi
      \if\@currchar'\relax
         \def\@templist{\prime}%
      \else
         \def\@templist{\prime\prime}%
      \fi
   \fi
   \def\@currchar{#1}%
   \@timeform@check@char
   \@lastchclass=\thr@@
   \ifnum\@chclass=\thr@@
      \def\@timeform@next{\@timeform@loop@iii}%
   \else
      \let\@currchar\@templist
      \let\@templist\@empty
      \def\@timeform@next{\@timeform@loop@ii#1}%
   \fi
   \@timeform@next}
\def\@timeform@loop@iv#1{%
   \ifx\@timeform@prevdigits\@empty
      \edef\@tempa{%
         \@timeform@lastdigits
         \noexpand\@timeform@dot}%
   \else
      \toks@\expandafter{\@timeform@lastunit}%
      \edef\@tempa{%
         \@timeform@prevdigits
         \noexpand\@timeform@unit{\the\toks@}%
         \@timeform@lastdigits
         \noexpand\@timeform@dot}%
      \let\@timeform@lastunit\@empty
   \fi
   \@csc@t\@timeform@body\@timeform@body\@tempa
   \let\@timeform@prevdigits\@empty
   \let\@timeform@lastdigits\@empty
   \def\@currchar{#1}%
   \@timeform@check@char
   \ifnum\@chclass<\z@\else
      \@lastchclass4\relax
      \expandafter\@timeform@loop
   \fi}
\def\@timeform@loop@v#1{%
   \def\@currchar{#1}%
   \ifx\@currchar\@nnil
      \ifx\@timeform@lastdigits\@empty
         \let\@tempa\@templist
      \else
         \toks@\expandafter{\@timeform@lastunit}%
         \edef\@tempa{%
            \@timeform@lastdigits
            \noexpand\@timeform@unit{\the\toks@}%
            \@templist}%
      \fi
      \@csc@t\@timeform@body\@timeform@body\@tempa
      \let\@timeform@next\relax
   \else
      \expandafter\ifx\csname @timeform@cmdlist@\@currchar\endcsname\relax
         \let\@currchar\@empty
      \else
         \edef\@currchar{%
            \noexpand\protect\expandafter\noexpand
            \csname @timeform@cmdlist@\@currchar\endcsname}%
      \fi
      \def\@timeform@next{\@timeform@loop}%
      \@lastchclass\@ne
      \@chclass\@ne
   \fi
   \@timeform@next}
\def\@timeform@cmdlist@P{\ensuremath{\pm}}
\def\@timeform@cmdlist@M{\ensuremath{\mp}}
\def\@timeform@unit#1{%
   \ensuremath{%
      \mathchoice
         {\@timeform@unit@\scriptstyle{#1}}%
         {\@timeform@unit@\scriptstyle{#1}}%
         {\@timeform@unit@\scriptscriptstyle{#1}}%
         {\@timeform@unit@\scriptscriptstyle{#1}}}}
\def\@timeform@unit@#1#2{%
   \setbox\z@\hbox{$\m@th#1\mathrm{#2}$}%
   \setbox\tw@\hbox{$\m@th#1\mathrm{m}$}%
   \dimen@\ht\tw@ \advance\dimen@-\ht\z@
   \ifdim\dimen@<\z@ \dimen@\z@ \fi
   {}^{\raise\dimen@\box\z@}}
\def\@timeform@dot{%
   \ensuremath{%
      \mathchoice
         {\@timeform@dot@\displaystyle\scriptstyle}%
         {\@timeform@dot@\textstyle\scriptstyle}%
         {\@timeform@dot@\scriptstyle\scriptscriptstyle}%
         {\@timeform@dot@\scriptscriptstyle\scriptscriptstyle}}%
   \def\@timeform@lastunit{}}
\def\@timeform@dot@#1#2{%
   \rlap{$\m@th#1.$}%
   \mkern1mu\@timeform@unit@#2{\@timeform@lastunit}}
\def\draft{}\def\tenpoint{}\def\elevenpoint{}\def\twelvepoint{}"#
  );
  Ok(())
}

/// The class's own `{longtable}`/`{longtable*}` (pasj00.cls:884-916, pasj01.cls:967-1000, pasj02.cls:956-989) for a
/// paper whose source lacks the class: the longtable binding's table, which keeps the interface the class shares with
/// longtable.sty (`\endfirsthead`/`\endhead`/`\endfoot`/`\endlastfoot`, pasj00.cls:1241-1244), with the class's own
/// `\caption` and `\label`. Its `\@lt@init` lets them to `\@lt@setcaption` and `\@lt@label` (pasj00.cls:1068-1070,
/// pasj01.cls:1152-1154, pasj02.cls:1141-1143): each is a `\noalign` that gobbles a following `\\` if there is one
/// (`\@lt@remove@cr`; pasj00.cls:1245-1270, pasj01.cls:1329-1354, pasj02.cls:1318-1343), so a caption is no row and
/// PASJ papers write `\caption{…}\label{…}`, then `\hline` and the head row. The optional argument
/// (`\@lt@captionnumber`, a number in place of the counter's) is the list entry's, as the binding's kernel `\caption`
/// reads it everywhere else; the star changes nothing, as in the class. Without the class's `{longtable}` the tables of
/// astro-ph0407419, 0704.1713, 0705.1365, 1108.4491, astro-ph0511689, astro-ph0605217, astro-ph0612753 and
/// astro-ph0701701 were text with a misplaced-alignment error per row; with longtable.sty's caption rule (a row ended
/// by `\\`), the head row after each of their captions was lost.
/// Guard `perfect_kernel_batch65h::pasj00_without_its_class_keeps_longtable`.
fn pasj_unshipped_longtable() -> Result<()> {
  RequirePackage!("longtable");
  // The binding lets `\caption`/`\label` to these two at the table's start; the class's `\@lt@init` comes after.
  DefMacro!(
    "\\lx@pasj@lt@init",
    r"\let\lx@longtable@caption\lx@pasj@lt@setcaption\let\lx@longtable@label\lx@pasj@lt@label"
  );
  DefMacro!("\\longtable", r"\begingroup\lx@pasj@lt@init\LT@array");
  DefMacro!(
    "\\csname longtable*\\endcsname",
    r"\begingroup\lx@pasj@lt@init\LT@array"
  );
  Let!("\\lx@pasj@lt@@label", "\\lx@longtable@label");
  DefMacro!("\\lx@pasj@lt@setcaption OptionalMatch:* []{}", sub[(_star, number, caption)] {
    pasj_lt_remove_cr()?;
    Ok(Invocation!(r"\noalign{\lx@longtable@caption@text{#1}{#2}}", vec![number, Some(caption)]))
  });
  DefMacro!("\\lx@pasj@lt@label Semiverbatim", sub[(label)] {
    pasj_lt_remove_cr()?;
    Ok(Invocation!(r"\noalign{\lx@pasj@lt@@label{#1}}", vec![Some(label)]))
  });
  Ok(())
}

/// pasj00.cls `\@lt@remove@cr` (:1254-1261): past spaces (`\@ifnextchar`), a `\\` is gobbled; any other token stays.
/// Read where the class reads it, before its `\noalign` closes; the tokens read are not expanded.
fn pasj_lt_remove_cr() -> Result<()> {
  if let Some(next) = read_non_space()?
    && !x_equals(&next, &T_CS!("\\\\"))
  {
    unread_one(next);
  }
  Ok(())
}

/// PASJ's frontmatter macros as the frontmatter. One `\author` lists every author, separated by commas and `\&`, its
/// rows broken by `\\` (pasj00.cls:73-104), and the template puts each author's marks after the comma that ends the
/// name (`Name,\altaffilmark{1}\orcid{…}`, pasj02's usage). The kernel would read each line after a `\\` as an
/// affiliation (most of 0707.3867's 26 authors) and hand a mark after a comma to the next author (2504.06663,
/// 0704.3654), so the binding splits the list itself ([`pasj_author_list`]). Documents write `\altaffiltext{n}{text}`
/// and `\affil{text}` under all three versions, whatever pasj00's argument shapes; pasj00's `\email` takes an optional
/// name; pasj02's `\altemailmark` only marks the author `\email` belongs to.
fn pasj_frontmatter() -> Result<()> {
  Let!("\\lx@pasj@author", "\\author");
  DefMacro!("\\author[]{}", sub[(label, author)] {
    let mut call = vec![T_CS!("\\lx@pasj@author")];
    if let Some(label) = label {
      call.push(T_OTHER!("["));
      call.extend(label.unlist());
      call.push(T_OTHER!("]"));
    }
    call.push(T_BEGIN!());
    call.extend(pasj_author_list(&author.unlist())?);
    call.push(T_END!());
    Ok(Tokens::new(call))
  });
  DefMacro!("\\KeyWords{}", "\\lx@add@keywords{#1}");
  Let!("\\kword", "\\KeyWords");
  DefMacro!("\\affil{}", "\\lx@add@affiliation{#1}");
  DefMacro!(
    "\\altaffilmark Semiverbatim",
    "\\lx@request@frontmatter@annotation[altaffil]{#1}"
  );
  // An empty text (a template's placeholder) makes no contact, which would stand as an empty creator (0704.3654).
  DefMacro!("\\altaffiltext Semiverbatim {}", sub[(label, text)] {
    if text.unlist_ref().iter().all(|t| t.get_catcode() == Catcode::SPACE) {
      Ok(Tokens!())
    } else {
      Ok(Invocation!("\\lx@add@contact[role=altaffiliation,label={altaffil:#1}]{#2}", vec![Some(label), Some(text)]))
    }
  });
  // pasj00.cls:68-69, the address in `\small\itshape`; the binding keeps the current font, not that size and shape
  // (1001.5133)
  DefMacro!("\\email[]{}", "\\lx@add@email{\\lx@address@textfont{#2}}");
  def_macro_noop("\\altemailmark")?;
  DefMacro!("\\orcid Semiverbatim", "\\lx@add@orcid{#1}");
  DefMacro!("\\Received{}", "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\Accepted{}", "\\lx@add@date[role=accepted]{#1}");
  Ok(())
}

/// The author marks a PASJ name carries, with their arguments: (control sequence, takes an optional argument, braced
/// arguments) — `\altaffilmark{…}`, `\orcid{…}`, `\thanks[…]{…}` (pasj01.cls:2019), `\email[…]{…}`, `\altemailmark`.
const PASJ_AUTHOR_MARKS: [(&str, bool, usize); 5] = [
  ("\\altaffilmark", false, 1),
  ("\\orcid", false, 1),
  ("\\thanks", true, 1),
  ("\\email", true, 1),
  ("\\altemailmark", false, 0),
];

/// A PASJ author list as `\and`-separated names: it splits at depth-0 `,`, `\&`, `\and` and `\\` (with the `*` and a
/// closed `[<skip>]` that follow it), gives a piece's leading marks (and math superscripts) back to the name before it,
/// drops a piece's leading "and", and keeps no empty piece. The `\and`s: each written, as before (pasj00.cls:1878
/// prints its own), and each held in a macro of the document's, which the separator query reads as the kernel's
/// (`and_by_meaning`, Option C P2-5).
fn pasj_author_list(toks: &[Token]) -> Result<Vec<Token>> {
  let ands = and_runs(toks, &[T_CS!("\\and")])?.top_level(toks);
  let mut pieces: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0usize;
  let mut i = 0;
  while i < toks.len() {
    if let Some(end) = ands.get(&i) {
      i = *end;
      pieces.push(Vec::new());
      continue;
    }
    let t = toks[i];
    i += 1;
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    let separator = depth == 0 && (t == T_OTHER!(",") || t == T_CS!("\\&") || t == T_CS!("\\\\"));
    if !separator {
      pieces.last_mut().unwrap().push(t);
      continue;
    }
    if t == T_CS!("\\\\") {
      let mut k = i;
      while k < toks.len() && toks[k].get_catcode() == Catcode::SPACE {
        k += 1;
      }
      if k < toks.len() && toks[k] == T_OTHER!("*") {
        k += 1;
      }
      if k < toks.len()
        && toks[k] == T_OTHER!("[")
        && let Some(close) = closing_bracket(&toks[k..])
      {
        k += close + 1;
      }
      i = k;
    }
    pieces.push(Vec::new());
  }
  let mut names: Vec<Vec<Token>> = Vec::new();
  for mut piece in pieces {
    let marks = leading_marks(&piece);
    if marks > 0
      && let Some(prev) = names.last_mut()
    {
      prev.extend(piece.drain(..marks));
    }
    let piece = strip_leading_and(piece);
    if piece.iter().any(|t| t.get_catcode() != Catcode::SPACE) {
      names.push(piece);
    }
  }
  let mut list = Vec::new();
  for (n, name) in names.into_iter().enumerate() {
    if n > 0 {
      list.push(T_CS!("\\and"));
    }
    list.extend(name);
  }
  Ok(list)
}

/// The length of a piece's leading run of author marks (with their arguments), math superscripts (`$^{1}$`) and
/// spaces — 0 when it starts with none: the marks that follow the comma ending the previous name.
fn leading_marks(piece: &[Token]) -> usize {
  let mut end = 0;
  let mut i = 0;
  while i < piece.len() {
    let t = piece[i];
    match t.get_catcode() {
      Catcode::SPACE => i += 1,
      Catcode::MATH => {
        let Some(len) = piece[i + 1..]
          .iter()
          .position(|t| t.get_catcode() == Catcode::MATH)
        else {
          break;
        };
        if piece.get(i + 1).map(|t| t.get_catcode()) != Some(Catcode::SUPER) {
          break;
        }
        i += len + 2;
        end = i;
      },
      Catcode::CS => {
        let Some((_, optional, braced)) = PASJ_AUTHOR_MARKS
          .iter()
          .find(|(cs, ..)| t.with_str(|s| s == *cs))
        else {
          break;
        };
        i += 1;
        // Exactly its own arguments: a following brace group belongs to the next name (`{\'A}lvaro`).
        let skip_spaces = |mut k: usize| {
          while k < piece.len() && piece[k].get_catcode() == Catcode::SPACE {
            k += 1;
          }
          k
        };
        if *optional {
          let k = skip_spaces(i);
          if piece.get(k) == Some(&T_OTHER!("[")) {
            match closing_bracket(&piece[k..]) {
              Some(close) => i = k + close + 1,
              None => return end,
            }
          }
        }
        for _ in 0..*braced {
          let k = skip_spaces(i);
          match piece.get(k) {
            Some(b) if b.get_catcode() == Catcode::BEGIN => {
              let mut level = 0usize;
              i = k;
              while i < piece.len() {
                match piece[i].get_catcode() {
                  Catcode::BEGIN => level += 1,
                  Catcode::END => {
                    level -= 1;
                    if level == 0 {
                      i += 1;
                      break;
                    }
                  },
                  _ => {},
                }
                i += 1;
              }
            },
            // An undelimited argument is one token (`\altaffilmark1`).
            Some(_) => i = k + 1,
            None => return end,
          }
        }
        end = i;
      },
      _ => break,
    }
  }
  end
}

/// The offset of the `]` closing the `[` that `toks` starts with, at brace depth 0 (`\thanks[{a]b}]{…}`).
fn closing_bracket(toks: &[Token]) -> Option<usize> {
  let mut depth = 0usize;
  for (n, t) in toks.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && *t == T_OTHER!("]") => return Some(n),
      _ => {},
    }
  }
  None
}

/// A piece without a leading "and" — the last name of a comma list ("…, and D. Four").
fn strip_leading_and(piece: Vec<Token>) -> Vec<Token> {
  let start = piece
    .iter()
    .position(|t| t.get_catcode() != Catcode::SPACE)
    .unwrap_or(piece.len());
  let word: String = piece[start..]
    .iter()
    .take(3)
    .map(|t| t.to_string())
    .collect();
  let after = piece.get(start + 3);
  if word == "and" && after.is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
    piece[start + 3..].to_vec()
  } else {
    piece
  }
}

LoadDefinitions!({
  pasj_class("pasj00")?;
});
