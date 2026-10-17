use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Mostly because this loads packages which are then expected
  RequirePackage!("ifthen");
  RequirePackage!("xkeyval");
  RequirePackage!("xcolor");
  RequirePackage!("tikz");
  // todonotes.sty:32 (Perl's binding has it commented out, todonotes.sty.ltxml:29, KNOWN_PERL_ERRORS #943): its
  // documents draw with positioning's keys (`on grid`: 1807.00512, 2107.03711). Guard
  // `perfect_kernel_batch71a::todonotes_loads_positioning`.
  RawTeX!(r"\usetikzlibrary{positioning}");
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
  // todonotes.sty:581-583 `\todostyle{<name>}{<keys>}` defines the `\todo` key `<name>` as those keys (27 papers:
  // 2407.14906, 2409.03694, 2410.02385); `\todo` reads its keys as a note's options here, so the style key is
  // declared and unused. :46-49: at the document's start todonotes provides an empty `\phantomsection` when hyperref
  // has not defined one (12 papers: 1308.6029, 2103.09181, 2208.10521). Perl's binding defines neither
  // (KNOWN_PERL_ERRORS #805).
  RawTeX!(r"\newcommand*{\todostyle}[2]{\define@key{todonotes}{#1}[]{\setkeys{todonotes}{#2}}}
\AtBeginDocument{\ifx\undefined\phantomsection\newcommand{\phantomsection}{}\fi}");

  // todonotes.sty:159-160 `\if@todonotes@disabled`, set by `disable`, which papers test to define their own note
  // commands (2606.13443); the `obeyDraft`/`obeyFinal` modes that also set it (:206-218) are not modeled.
  RawTeX!(r"\newif\if@todonotes@disabled");
  DeclareOption!("disable", {
    Digest!(r"\@todonotes@disabledtrue")?;
    // Perl todonotes.sty.ltxml L22-23: `disable` replaces \todo with
    // an empty macro and locks it so that subsequent \renewcommand
    // \todo from user code or further loads can't re-enable the todo
    // notes. Without the lock the suppression silently un-applies.
    DefMacro!("\\todo[]{}", None, locked => true);
  });

  // todonotes.sty:206 reads its options with `\ProcessOptionsX`: `\@curroptions` stays as it was.
  ProcessKeyOptions!();
});
