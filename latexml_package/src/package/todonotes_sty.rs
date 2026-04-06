use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Mostly because this loads packages which are then expected
  RequirePackage!("ifthen");
  RequirePackage!("xkeyval");
  RequirePackage!("xcolor");
  RequirePackage!("tikz");
  RequirePackage!("calc");

  DefMacro!("\\ext@todo", "todo");
  NewCounter!("todo");
  DefMacro!("\\todotyperefname",   "ToDo");
  DefMacro!("\\todo",              "\\lx@note{todo}");
  DefMacro!("\\missingfigure[]{}", "[Missing Figure: #2]");
  DefMacro!("\\todototoc",         None);
  DefMacro!("\\listoftodos",       None);
  DefMacro!("\\@todo[]{}",         None);
  DefMacro!("\\setuptodonotes{}",  None);

  DeclareOption!("disable", {
    DefMacro!("\\todo[]{}", None);
  });

  ProcessOptions!();
});
