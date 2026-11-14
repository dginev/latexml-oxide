use crate::prelude::*;

/// The AMS environments empheq's `overload` option gives an optional `[<empheq options>]` (empheq.sty:961-1066, at
/// `\AtBeginDocument`; `multline` under the default `newmultline`). `alignat` reads its column count first.
const OVERLOADED: [&str; 12] = [
  "equation",
  "equation*",
  "gather",
  "gather*",
  "align",
  "align*",
  "flalign",
  "flalign*",
  "multline",
  "multline*",
  "alignat",
  "alignat*",
];

#[rustfmt::skip]
LoadDefinitions!({
  // empheq.sty:37-55: `overload` (and `overload2`, which adds `\[…\]`) gives the AMS environments empheq's options;
  // the rest go to mathtools.
  DeclareOption!("overload", { assign_value("empheq_overload", true, None); });
  DeclareOption!("overload2", { assign_value("empheq_overload", true, None); });
  DeclareOption!("oldmultline", None);
  DeclareOption!("newmultline", None);
  DeclareOption!("ntheorem", None);
  DeclareOption!(None, {
    Digest!("\\PassOptionsToPackage{\\CurrentOption}{mathtools}")?;
  });
  ProcessOptions!();
  RequirePackage!("amsmath");
  RequirePackage!("mathtools");
  RequirePackage!("keyval");

  DefMacro!("\\empheqset{}", None);

  // The options of `{empheq}` and of an overloaded AMS environment (empheq.sty:785-838 `\EQ_options_name:`): `left`
  // and `right` are math set before and after the display, which stays as written — the first and the last cell of
  // the alignment receive them (`\begin{align}[left={F(x)=\empheqlbrace}]`, 1410.0933, 1906.02657, 2210.09120); the
  // boxes are styling, not kept. Perl's binding (empheq.sty.ltxml) reads and drops them all, the left-hand side too.
  RawTeX!(r"\def\lx@empheq@left{}\def\lx@empheq@right{}%
    \define@key{lx@EmphEq}{left}{\def\lx@empheq@left{#1}}%
    \define@key{lx@EmphEq}{right}{\def\lx@empheq@right{#1}}%
    \define@key{lx@EmphEq}{box}{}\define@key{lx@EmphEq}{innerbox}{}%
    \define@key{lx@EmphEq}{outerbox}{}\define@key{lx@EmphEq}{marginbox}{}%
    \def\lx@empheq@setkeys#1{\def\lx@empheq@left{}\def\lx@empheq@right{}\setkeys{lx@EmphEq}{#1}}%
    \def\lx@empheq@go#1#2#3{\csname lx@empheq@AmS@#2\endcsname#3#1}%
    \def\lx@empheq@ov#1#2[#3]{\lx@empheq@setkeys{#3}\expandafter\lx@empheq@go\expandafter{\lx@empheq@left}{#1}{#2}}%
    \def\lx@empheq@peek#1#2{\@ifnextchar[{\lx@empheq@ov{#1}{#2}}{\lx@empheq@ov{#1}{#2}[]}}%
    \def\lx@empheq@peekarg#1#2{\lx@empheq@peek{#1}{{#2}}}");

  // The AMS environments as amsmath left them, which `{empheq}` runs (and, under `overload`, the environments of the
  // same name): `\lx@empheq@AmS@<env>`/`\lx@empheq@AmS@end<env>`, after `\<env>`/`\end<env>`.
  let overload = lookup_bool("empheq_overload");
  for env in OVERLOADED {
    let (begin, end) = (s!("\\{env}"), s!("\\end{env}"));
    if !is_defined(&begin) || !is_defined(&end) {
      continue;
    }
    Let!(T_CS!(s!("\\lx@empheq@AmS@{env}")), T_CS!(begin.clone()));
    Let!(T_CS!(s!("\\lx@empheq@AmS@end{env}")), T_CS!(end.clone()));
    if overload {
      // empheq.sty:964-975 `\EQ_renew_ams_environment:n`: `\renewenvironment{<env>}[1][]`; alignat's count first
      // (:1013-1029). An environment that is a constructor (`\begin{equation}`, which `\begin` runs before
      // `\equation`) takes the option there, and keeps its own end (no `right`). The definitions keep their lock.
      let constructor_cs = s!("\\begin{{{env}}}");
      let constructor = is_defined(&constructor_cs);
      let (begin, target) = if constructor {
        Let!(T_CS!(s!("\\lx@empheq@AmS@begin@{env}")), T_CS!(constructor_cs.clone()));
        (constructor_cs, s!("begin@{env}"))
      } else {
        (begin, env.to_string())
      };
      let peek = if env.starts_with("alignat") {
        s!("\\lx@empheq@peekarg{{{target}}}")
      } else {
        s!("\\lx@empheq@peek{{{target}}}{{}}")
      };
      let lock = |cs: &str| Some(ExpandableOptions { locked: lookup_bool(&s!("{cs}:locked")), ..Default::default() });
      let begin_lock = lock(&begin);
      def_macro(T_CS!(begin), None, TokenizeInternal!(TeXString::assembled(peek)), begin_lock)?;
      if !constructor {
        let end_lock = lock(&end);
        let right = TokenizeInternal!(TeXString::assembled(s!("\\lx@empheq@right\\csname lx@empheq@AmS@end{env}\\endcsname")));
        def_macro(T_CS!(end), None, right, end_lock)?;
      }
    }
  }

  // Just pass on to the ams environment
  // Usually \begin{empheq}{amsenv} ==> \begin{amsenv}
  // but note specialcase: \begin{empheq}{alignat=2} ==> \begin{alignat}{2} !!!
  // The options' `left` opens the environment's first cell, `right` closes its last (above).
  DefMacro!("\\empheq[]{}",
    "\\lx@empheq@setkeys{#1}\\lx@empheq#2==\\end");

  RawTeX!(r"\def\lx@empheq #1=#2=#3\end{%
    \expandafter\def\expandafter\endempheq\expandafter{\expandafter\lx@empheq@right
      \csname lx@empheq@AmS@end#1\endcsname}%
    \if.#2.\def\lx@empheq@args{}\else\def\lx@empheq@args{{#2}}\fi
    \expandafter\lx@empheq@go\expandafter{\lx@empheq@left}{#1}{\lx@empheq@args}}");

  // empheq.sty:839-882 `\DeclareLeftDelimiter`/`\DeclareRightDelimiter`: `\empheq<delim>` and `\empheqbig<delim>`
  // (`\empheqlbrace`, `\empheqbigrangle`, …), the delimiter grown to the display's height (`\left#2…\right.`, its
  // phantom and kerns presentational), then the optional material.
  RawTeX!(r"\newcommand*\DeclareLeftDelimiter[2][]{%
      \@namedef{empheq\expandafter\@gobble\string#2}{\left#2\right.#1}%
      \@namedef{empheqbig\expandafter\@gobble\string#2}{\left#2\right.#1}}%
    \newcommand*\DeclareRightDelimiter[2][]{%
      \@namedef{empheq\expandafter\@gobble\string#2}{#1\left.\right#2}%
      \@namedef{empheqbig\expandafter\@gobble\string#2}{#1\left.\right#2}}%
    \DeclareLeftDelimiter{\lbrace}\DeclareRightDelimiter{\rbrace}%
    \DeclareLeftDelimiter{\lbrack}\DeclareRightDelimiter{\rbrack}%
    \DeclareLeftDelimiter{\langle}\DeclareRightDelimiter{\rangle}%
    \DeclareLeftDelimiter{\lvert}\DeclareRightDelimiter{\rvert}%
    \DeclareLeftDelimiter{\lVert}\DeclareRightDelimiter{\rVert}%
    \DeclareLeftDelimiter{\lfloor}\DeclareRightDelimiter{\rfloor}%
    \DeclareLeftDelimiter{\lceil}\DeclareRightDelimiter{\rceil}%
    \DeclareLeftDelimiter{\lparen}\DeclareRightDelimiter{\rparen}");
});
