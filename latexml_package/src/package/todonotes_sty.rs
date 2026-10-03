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
  DefMacro!("\\todotyperefname", "\\lx@latin@name{ToDo}");
  DefMacro!("\\todo",              "\\lx@note{todo}");
  DefMacro!("\\missingfigure[]{}", "[Missing Figure: #2]");
  DefMacro!("\\todototoc",         None);
  // todonotes.sty:323 `\listoftodos[1][\@todonotes@todolistname]`: the optional is the list's
  // heading; the list itself is not built, as in Perl (L38), whose no-argument macro printed the
  // `[heading]` (KPE #372).
  DefMacro!("\\listoftodos[]",     None);
  DefMacro!("\\@todo[]{}",         None);
  DefMacro!("\\setuptodonotes{}",  None);

  DeclareOption!("disable", {
    // Perl todonotes.sty.ltxml L22-23: `disable` replaces \todo with
    // an empty macro and locks it so that subsequent \renewcommand
    // \todo from user code or further loads can't re-enable the todo
    // notes. Without the lock the suppression silently un-applies.
    DefMacro!("\\todo[]{}", None, locked => true);
  });

  ProcessOptions!();
});
