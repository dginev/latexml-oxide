use crate::prelude::*;

LoadDefinitions!({
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
  DefMacro!("\\UL@end OptionalMatch:*", r"\egroup");
  DefMacro!("\\UL@spfactor", None, "1000");

  // ulem.sty L286: \useunder{ucmd}{decl}{argcmd} aliases `decl` and
  // `argcmd` to forms that apply `ucmd{...}` to content. We \let both
  // straight to `ucmd` so `\ul{foo}` -> `\uline{foo}` works. Empty
  // declaration/argument-command slots (papers write `\useunder{\uline}{\ul}{}`)
  // produce no let.
  // Witnesses 2405.20343, 2406.08270.
  DefMacro!("\\useunder{}{}{}", sub[(ucmd, decl, argcmd)] {
    let mut out: Vec<Token> = Vec::new();
    let ucmd_v = ucmd.unlist();
    if !ucmd_v.is_empty() {
      let target = ucmd_v[0];
      let decl_v = decl.unlist();
      if !decl_v.is_empty() {
        out.extend([T_CS!("\\let"), decl_v[0], target]);
      }
      let arg_v = argcmd.unlist();
      if !arg_v.is_empty() {
        out.extend([T_CS!("\\let"), arg_v[0], target]);
      }
    }
    Ok(Tokens::new(out))
  });
});
