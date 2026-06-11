use latexml_package::prelude::*;

LoadDefinitions!({
  // ar5iv-bindings/bindings/CJK.sty.ltxml L17-24: CJK environment is a
  // transparent wrapper that passes body through. `leaveHorizontal` +
  // `internal_vertical` mode ensures paragraph breaks inside CJK blocks
  // feed the line-wrapping / height-estimation code correctly instead of
  // accumulating inside an implicit horizontal list.
  DefEnvironment!("{CJK}{}{}", "#body",
    before_digest => { leave_horizontal()?; },
    mode => "internal_vertical"
  );
  DefEnvironment!("{CJK*}{}{}", "#body",
    before_digest => { leave_horizontal()?; },
    mode => "internal_vertical"
  );
  DefMacro!("\\CJKfamily{}", "#1");
});
