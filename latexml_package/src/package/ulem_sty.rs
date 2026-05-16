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
