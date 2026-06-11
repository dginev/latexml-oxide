use latexml_package::prelude::*;

//**********************************************************************
// LaTeXML Declaration for David Carlisle's xii.tex
//**********************************************************************
LoadDefinitions!({
  // Simple reuse of Whatsit
  DefConstructor!(
    "\\BoxDup{}",
    "<ltx:XMWrap><ltx:XMWrap>#1</ltx:XMWrap><ltx:XMWrap>#1</ltx:XMWrap></ltx:XMWrap>"
  );

  // Deferred reuse of Whatsit
  DefConstructor!("\\SaveBox{}", "#1",
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      assign_value("SAVED_WHATSIT", whatsit.get_arg(1), Some(Scope::Global)); });
  DefConstructor!("\\UseBox", "#savedbox",
    properties => { Ok(stored_map!("savedbox" => lookup_value("SAVED_WHATSIT"))) });

  // Some math macros that create ltx:XMDual's for testing
  DefMath!("\\parens{}",   "(#1)", meaning => "parentheses");
  DefMath!("\\brackets{}", "[#1]", meaning => "brackets");
});
