use crate::package::*;

LoadDefinitions!(_state, {
  // Mostly ignorable, although it could add an attribute to an ancestor
  // to record the desired justification.
  // Spacing stuff
  DefConstructor!("\\@", "");
  // Math spacing.

  // Math style.
  // Also record that this explicitly sets the mathstyle (support for \over, etal)
  DefPrimitive!("\\displaystyle", sub[_stomach,_args,state] {
    MergeFont!(mathstyle => "display");
    Tbox::new(*EMPTY_SYM, None, None, Tokens!(T_CS!("\\displaystyle")),
      stored_map!("explicit_mathstyle" => true), state) });
  DefPrimitive!("\\textstyle", sub[_stomach,_args,state] {
    MergeFont!(mathstyle => "text");
    Tbox::new(*EMPTY_SYM, None, None, Tokens!(T_CS!("\\textstyle")),
      stored_map!("explicit_mathstyle" => true), state) });
  DefPrimitive!("\\scriptstyle", sub[_stomach,_args,state] {
    MergeFont!(mathstyle => "script");
    Tbox::new(*EMPTY_SYM, None, None, Tokens!(T_CS!("\\scriptstyle")),
      stored_map!("explicit_mathstyle" => true), state) });
  DefPrimitive!("\\scriptscriptstyle", sub[_stomach,_args,state] {
    MergeFont!(mathstyle => "scriptscript");
    Tbox::new(*EMPTY_SYM, None, None, Tokens!(T_CS!("\\scriptscriptstyle")),
      stored_map!("explicit_mathstyle" => true), state) });

});
