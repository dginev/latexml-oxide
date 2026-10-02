//! showexpl.sty: `LTXexample` and `\LTXinputExample` typeset LaTeX source as a listing AND run it as the result.
//!
//! The raw package writes the body to `\SX@codefile` through listings' write-file layer and reads it back twice
//! (showexpl.sty:189-267); its `\lst@newenvironment` end-group parse failed in the raw-load path (a readBalanced
//! `Expected opening '{'`), so `\SX@put@code@result` never registered. The binding reproduces the semantics
//! instead: the body is captured raw, typeset by the listings display engine with the environment's keys (showexpl's
//! keys are listings keys, :46-68: `caption=`/`label=` caption the code listing), and run as the result in
//! showexpl's preset group (`\SX@@preset`, :88-114 verbatim; `\SX@resultInput`, :348-366) — a whole-document
//! example's `\documentclass`, `\usepackage`, `document` environment, `\label`s do nothing (showexpl-test: the
//! conversion ended at an example's `\end{document}`, 50 → 95 % recall with the preset). What is not modelled is
//! recorded in OXIDIZED_DESIGN_DIVERGENCES #167. Perl has no binding and skips the package as a missing file.
//!
//! Witnesses (package loaded, CONVERR_7/CONVERR_3 → OK): 1604.00381, 1606.01035, 1706.03232, 1804.02704,
//! 1804.07221, 1612.01022, 1905.12059, 1706.09226, 1701.01402, 1812.06820, 1801.01025, 1806.10927, 2001.08314,
//! 2002.09910, 1901.08750.

use crate::{
  package::listings_sty::{
    listings_read_raw_lines, lst_activate, lst_get_tokens, lst_group_opener,
    lst_process_display_scoped, lst_run_body_via_input,
  },
  prelude::*,
};

/// showexpl.sty:348-366 `\SX@resultInput`: the example's result is read back in a group with `%` a comment
/// character, `@` other, a line end a space, and showexpl's preset (`\SX@@preset`, then the `preset=` key) in force,
/// so a whole-document example's `\documentclass`, `\usepackage`, `document` environment, `\label`, `\index`, … do
/// nothing (`\footnote` and `\cite` are locked here and still act).
fn result_in_preset(input: Tokens, preset: Tokens) -> Vec<Token> {
  let mut out = TokenizeInternal!(
    r"\begingroup\MakePercentComment\makeatother\catcode`\^^M=5\relax\SX@@preset"
  )
  .unlist();
  out.extend(preset.unlist());
  // :97-103 nulls `\cref@old@label` under cleveref, which the cleveref binding's `\label` never calls.
  out.extend(TokenizeInternal!(r"\@ifpackageloaded{cleveref}{\let\label\@gobble}{}").unlist());
  out.extend(input.unlist());
  out.extend(TokenizeInternal!(r"\par\endgroup").unlist());
  out
}

#[rustfmt::skip]
LoadDefinitions!({
  // showexpl.sty L1-13 \RequirePackage chain (attachfile is loaded
  // conditionally via `\IfFileExists`; we load it unconditionally —
  // harmless, has a binding).
  RequirePackage!("listings");
  RequirePackage!("refcount");
  RequirePackage!("varwidth");
  RequirePackage!("float");
  // {LTXexample}[keys] — showexpl's whole point: typeset the SOURCE as a
  // listing AND its RESULT (real showexpl routes the body through listings'
  // write-file layer into \jobname.tmp and \input's it back). Reproduce that
  // semantic directly: capture the body raw, emit the code listing through
  // the listings display engine, then re-tokenize the body so it EXECUTES
  // as the result. `[pos=…]`-style keys arrange the two blocks on the page —
  // presentation. (The TL doc corpus — koma/babel/gauss manuals — uses this
  // env heavily; the earlier noop stub predates OXIDIZED_DESIGN #161, whose
  // DefPlain fix unblocked raw showexpl parsing, and dropped example bodies
  // entirely.)
  // showexpl.sty:46-68: its keys are listings keys (`\lst@Key`); `preset` is read back below, the layout keys
  // (`pos`, `width`, `hsep`, `rframe`, …) place the two blocks on the page, and `graphic` (a precompiled image of the
  // result) is not modelled: the result is always typeset.
  for key in ["pos", "width", "hsep", "vsep", "overhang", "rframe", "preset", "explpreset", "codefile",
              "justification", "graphic"] {
    DefKeyVal!("LST", key, "");
  }
  DefKeyVal!("LST", "scaled", "", "!");
  for key in ["wide", "rangeaccept", "varwidth", "attachfile"] {
    DefKeyVal!("LST", key, "", "true");
  }
  // showexpl.sty:88-114 (`\SX@@preset`, `\SX@eat@version`) and :346-347, verbatim.
  RawTeX!(r"\newcommand*\SX@@preset{%
  \renewcommand\documentclass[2][]{\SX@eat@version}%
  \renewcommand\usepackage[2][]{\SX@eat@version}%
  \renewenvironment{document}{}{}%
  \renewcommand\cite[1][]{}%
  \let\tableofcontens\relax \let\listoffigures\relax
  \let\listoftables\relax \let\printindex\relax
  \let\listfiles\relax \let\nofiles\relax
  \let\index\@gobble
  \expandafter\ifx\csname ver@cleveref.sty\endcsname\relax
    \let\refstepcounter=\stepcounter
    \let\label\@gobble
  \else
    \let\cref@old@refstepcounter=\stepcounter
    \let\cref@old@label=\@gobble
  \fi
  \let\bibliography\@gobble
  \let\pagestyle\@gobble \let\thispagestyle\@gobble
  \renewcommand\marginpar[2][]{}%
  \renewcommand\footnote[2][]{}%
  \let\@footnotetext\@gobble
}
\newcommand*\SX@eat@version[1][]{}
\providecommand*\MakePercentIgnore{\catcode`\%9\relax}
\providecommand*\MakePercentComment{\catcode`\%14\relax}");
  // {LTXexample}[keys] — showexpl's whole point: the SOURCE as a listing AND its RESULT. Real showexpl writes the
  // body to `\SX@codefile` and reads it back twice (showexpl.sty:189-267): typeset as code with the keys
  // (`\lstset{\SX@explpreset,#1}`), and run as the result in the preset group. The keys are activated as
  // `lstlisting`'s are, so `caption=`/`label=` caption the code listing (showexpl puts one caption around the pair,
  // :245/:261). The result was run without the preset: a whole-document example ended the conversion at its own
  // `\end{document}` and its `\label`s leaked (showexpl-test, 50 % recall; RUST-ONLY).
  {
    let cs = T_CS!("\\begin{LTXexample}");
    let params = parse_parameters("OptionalKeyVals:LST", &cs, true)?;
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        let kv: Option<KeyVals> = args.into_iter().next().unwrap_or_default().into();
        bgroup();
        assign_value("current_environment", Stored::String(pin("LTXexample")), None);
        lst_activate(kv.as_ref());
        let text = listings_read_raw_lines("LTXexample");
        let name = lst_get_tokens("name");
        let code = lst_process_display_scoped(if name.is_empty() { None } else { Some(name) }, &text);
        let preset = lst_get_tokens("preset");
        egroup()?;
        let mut out = lst_group_opener("LTXexample", kv.as_ref())?;
        out.extend(code);
        out.extend(result_in_preset(lst_run_body_via_input("LTXexample", &text)?, preset));
        let mut end_tokens = vec![T_CS!("\\end"), T_BEGIN!()];
        end_tokens.extend(ExplodeText!("LTXexample"));
        end_tokens.push(T_END!());
        unread_expansion(Tokens::new(end_tokens));
        Ok(Tokens::new(out))
      },
    )));
    def_macro(cs, params, expansion, None)?;
    DefMacro!(T_CS!("\\end{LTXexample}"), None, Tokens!());
  }
  def_macro_noop("\\endLTXexample")?;
  // \LTXinputExample[keys]{file} (showexpl.sty:396-398): the file as code, then run in the preset group (its
  // `preset=` key is not read back here).
  DefMacro!("\\LTXinputExample[]{}", "\\lstinputlisting[#1]{#2}\\lx@SX@resultinput{#2}");
  DefMacro!("\\lx@SX@resultinput{}", sub[(file)] {
    let mut input = vec![T_CS!("\\input"), T_BEGIN!()];
    input.extend(file.clone().unwrap_or_default().unlist());
    input.push(T_END!());
    Tokens::new(result_in_preset(Tokens::new(input), Tokens!()))
  });
  def_macro_noop("\\setupSXfiles")?;
  def_macro_noop("\\setupLZfiles")?;
  // showexpl.sty:66-86 load-time state that documents rebuilding
  // `LTXexample` from the internals read back (lshort-german l2kurz.tex:73-100
  // `\edef\x{\endgroup\def\noexpand\SX@codefile{\SX@codefile}…}\x`: with the
  // macros undefined the self-reference `\def\SX@codefile{\SX@codefile}`
  // "expands into itself" 96 times). The counter is showexpl.sty:57.
  RawTeX!(concat!(
    r"\newcommand*\SX@graphicname{}\newcommand*\SX@graphicparam{}",
    r"\newcommand\ResultBox{}\let\ResultBox=\fbox",
    r"\newdimen\ResultBoxSep\ResultBoxSep=\fboxsep\newdimen\ResultBoxRule\ResultBoxRule=\fboxrule",
    r"\newcommand*\SX@pos{}\newcommand*\SX@width{}\newcommand*\SX@hsep{}\newcommand*\SX@vsep{}",
    r"\newcommand*\SX@overhang{}\newcommand*\SX@rframe{}\newcommand\SX@preset{}",
    r"\newcommand*\SX@explpreset{}\newcommand*\SX@@explpreset{}",
    r"\newcommand*\SX@codefile{}\edef\SX@codefile{\jobname.tmp}",
    r"\newcommand*\SX@justification{\raggedright}",
    r"\@ifundefined{c@ltxexample}{\newcounter{ltxexample}}{}",
    // showexpl.sty:58-61,115 — the switches raw dependants skip over
    // (pst-exa.sty:104 `\if@SX@rangeaccept` inside a false `\ifpstexa@swpl`
    // branch: an UNDEFINED conditional is not counted by the skip, tex.web
    // §510, so the nested `\else`/`\fi` desync it — pst-exa-doc). Guard:
    // `perfect_kernel_batch56::showexpl_switches_balance_a_skipped_branch`.
    r"\newif\if@SX@rangeaccept\newif\if@SX@varwidth\newif\if@SX@wide",
    r"\newif\if@SX@attachfile\newif\ifSX@wasodd"
  ));
  // showexpl.sty:208 `\SX@put@code@result`: typeset the listing written to
  // `\SX@codefile` and run it as the result — the binding's own display path.
  DefMacro!("\\SX@put@code@result", "\\lstinputlisting{\\SX@codefile}\\lx@SX@resultinput{\\SX@codefile}");
});
