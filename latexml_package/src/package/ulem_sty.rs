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
});
