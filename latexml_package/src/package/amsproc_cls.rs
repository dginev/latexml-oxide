use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsproc.cls.ltxml — AMS proceedings article class
  // amsart.cls:58-60 (amsproc.cls, amsbook.cls alike): `\@classname` is the file that loaded this class, the top
  // of `\@currnamestack` (`\@pushfilename` prepends the loader, latex.ltx). Under a class that `\LoadClass`es it,
  // that is the loading class's name (quantumdust.cls:278-286 `\ClassWarning{\@classname}`, 2606.20943); loaded by
  // `\documentclass`, that entry is empty and the class names itself. Read here, in the class's own load, not in a file it loads.
  RawTeX!(r"\def\@tempa#1#2\@nil{\edef\@classname{#1}}
\expandafter\@tempa\@currnamestack{}{}{}\@nil
\ifx\@classname\@empty \edef\@classname{\@currname}\fi");
  // LoadClass('ams_core', withoptions => 1);
  load_class_with_options("ams_core", Tokens!())?;
});
