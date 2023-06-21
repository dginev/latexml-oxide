use crate::package::*;

LoadDefinitions!({
  // Mostly ignorable, although it could add an attribute to an ancestor
  // to record the desired justification.
  // Spacing stuff
  DefConstructor!("\\@", "");
  // Math spacing.

  // Math style.
  // Also record that this explicitly sets the mathstyle (support for \over, etal)
  DefPrimitive!("\\displaystyle", sub[_args] {
    MergeFont!(mathstyle => "display");
    Tbox::new(*EMPTY_SYM, None, None, Tokens!(T_CS!("\\displaystyle")),
      stored_map!("explicit_mathstyle" => true)) });
  DefPrimitive!("\\textstyle", sub[_args] {
    MergeFont!(mathstyle => "text");
    Tbox::new(*EMPTY_SYM, None, None, Tokens!(T_CS!("\\textstyle")),
      stored_map!("explicit_mathstyle" => true)) });
  DefPrimitive!("\\scriptstyle", sub[_args] {
    MergeFont!(mathstyle => "script");
    Tbox::new(*EMPTY_SYM, None, None, Tokens!(T_CS!("\\scriptstyle")),
      stored_map!("explicit_mathstyle" => true)) });
  DefPrimitive!("\\scriptscriptstyle", sub[_args] {
    MergeFont!(mathstyle => "scriptscript");
    Tbox::new(*EMPTY_SYM, None, None, Tokens!(T_CS!("\\scriptscriptstyle")),
      stored_map!("explicit_mathstyle" => true)) });

});
