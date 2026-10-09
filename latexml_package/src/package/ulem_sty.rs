use crate::prelude::*;

LoadDefinitions!({
  // ulem.sty:46 `\global\let\UL@protected\csname protected\endcsname`: packages
  // that build on ulem read it (oblivoir-misc.sty:51-54 defines its own only
  // when ulem is absent). Perl's ulem.sty.ltxml omits it (SHARED).
  // Guard: `perfect_kernel_batch56::sweep46_single_name_gaps`.
  Let!("\\UL@protected", "\\protected");
  RequireResource!("ltx-ulem.css");

  DefConstructor!("\\uline{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_uline'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_uline'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\uuline{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_uuline'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_uuline'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\uwave{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_uwave'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_uwave'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\sout{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_sout'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_sout'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\xout{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_xout'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_xout'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\dashuline{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_dashuline'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_dashuline'>#1</ltx:text>)",
    enter_horizontal => true);
  DefConstructor!("\\dotuline{}",
    "?#isMath(<ltx:XMWrap class='ltx_ulem_dotuline'>#1</ltx:XMWrap>)(<ltx:text class='ltx_ulem_dotuline'>#1</ltx:text>)",
    enter_horizontal => true);

  DefMacro!("\\normalem", None, "");

  // ulem L343: \newdimen\ULdepth (initialized to \maxdimen for auto-mode).
  // Witnesses 2406.02021, 2406.18999.
  DefRegister!("\\ULdepth" => Dimension!("0pt"));
  DefRegister!("\\ULthickness" => Dimension!("0.4pt"));
  DefRegister!("\\UL@height" => Dimension!("0pt"));

  // Real-ulem word-machinery internals: our high-level constructors never
  // call them, but RAW add-on code wires into them by name — xeCJKfntef.sty
  // L102-134 installs `\xeCJK_ulem_word:nw` over `\UL@word` and probes
  // `\UL@stop`/`\UL@hook`/`\UL@end`/`\UL@start` (the
  // `wisdom_latexml_reimpl_internal_name_mismatch` shape; fixdif-zh-cn's
  // digestion loop). Provide the surface inertly: identity word processor,
  // no-op start/stop, the real `\UL@end *` delimited shape (ulem.sty L59).
  DefRegister!("\\UL@hook", Tokens!());
  def_macro_noop("\\UL@start")?;
  def_macro_noop("\\UL@stop")?;
  def_macro_noop("\\UL@putbox")?;
  def_macro_noop("\\UL@ender")?;
  def_macro_identity("\\UL@word{}")?;
  // ulem's documented extension contract (ulem.sty:232-233): a special
  // underline is `\def\command{\bgroup \markoverwith{<leader>} \ULon}`, and
  // the `\bgroup` is closed by the word machinery — `\UL@on#1` ends in
  // `\UL@end *` whose real body (:59) closes the group, `\UL@onin`/`\UL@onmath`
  // (:90-102) end in `\egroup`. Keep that contract with the identity word
  // processor: CJKfntef.sty:258-283 `\CJK@UL` (`\CJKunderline`, raw) left the
  // group open and unbalanced the enclosing list (jnuexam exam sheets, 8
  // CJKfntef manuals). `\markoverwith` only builds the leader box.
  def_macro_noop("\\markoverwith{}")?;
  DefMacro!(
    "\\ULon",
    r"\ifmmode\expandafter\UL@onmath\else\expandafter\UL@on\fi"
  );
  DefMacro!("\\UL@on{}", r"#1\UL@end *");
  DefMacro!("\\UL@onin{}", r"#1\egroup");
  DefMacro!("\\UL@onmath{}", r"#1\egroup");
  // ulem.sty:59 `\def\UL@end *{\relax\relax}`: `*` is a delimiter, nothing after it is read. An
  // `OptionalMatch:*` then skips spaces (Perl Parameter.pm:98-100), so a custom underline built on
  // `\ULon` (`\bgroup\markoverwith{…}\ULon`, ulem.sty:232-233; witnesses 2605.15048 `\redout`,
  // 2605.16437 `\Erase`) ate the space after its argument. The mark itself is not rendered yet (RED
  // `fonts-nfss/ulem_custom_mark_is_rendered`).
  DefMacro!("\\UL@end Match:*", r"\egroup");
  DefMacro!("\\UL@spfactor", None, "1000");

  // ulem.sty:286-293 `\useunder{ucmd}{decl}{argcmd}`: `argcmd` is `ucmd`, and `decl` a declaration that applies `ucmd`
  // to the rest of its group — `{\ul some words}` underlines "some words" (ulem.sty:288-289 opens `ucmd`'s argument with
  // an unmatched brace, the group's own `}` closing it, and `\UL@swender`, :107, puts the group's `}` back after). An
  // empty slot (papers write `\useunder{\uline}{\ul}{}`) defines nothing. Witnesses 2406.08270 (`\useunder` with
  // `\uline{…}`), 2405.20343 and 2406.03441 (`{\ul 18.8262}`, `{\ul …}`: one character underlined while `\ul` was let
  // to `\uline`). A `\ul{x}` outside any group, a fatal error in pdflatex, reads to the end of the input.
  DefMacro!("\\useunder{}{}{}", sub[(ucmd, decl, argcmd)] {
    let mut out: Vec<Token> = Vec::new();
    let ucmd_v = ucmd.unlist();
    if !ucmd_v.is_empty() {
      let target = ucmd_v[0];
      let decl_v = decl.unlist();
      if !decl_v.is_empty() {
        out.extend([T_CS!("\\def"), decl_v[0], T_BEGIN!(), T_CS!("\\lx@ulem@declaration"), target, T_END!()]);
      }
      let arg_v = argcmd.unlist();
      if !arg_v.is_empty() {
        out.extend([T_CS!("\\let"), arg_v[0], target]);
      }
    }
    Ok(Tokens::new(out))
  });
  // `\lx@ulem@declaration\uline` opens `\uline`'s argument, which the declaration's group closes; the argument read,
  // the group's own `}` is put back after it.
  DefMacro!("\\lx@ulem@declaration{}", sub[(ucmd)] {
    let mut out = vec![T_CS!("\\lx@ulem@declared")];
    out.extend(ucmd.unlist());
    out.push(T_BEGIN!());
    Ok(Tokens::new(out))
  });
  DefMacro!("\\lx@ulem@declared{}{}", sub[(ucmd, content)] {
    let mut out = ucmd.unlist();
    out.push(T_BEGIN!());
    out.extend(content.unlist());
    out.extend([T_END!(), T_END!()]);
    Ok(Tokens::new(out))
  });
  // ulem.sty:221 `\newbox\ULC@box`, the box the `\sout`-family builders measure in;
  // xeCJKfntef.sty sets it directly (xdupgthesis, xduugthesis, xduugtp; class
  // census 2026-09-24).
  RawTeX!(r"\newbox\ULC@box");
});
