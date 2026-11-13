use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: amsart.cls.ltxml — AMS journal article class

  //======================================================================
  // Document structure.

  // amsart.cls:58-60 (amsproc.cls, amsbook.cls alike): `\@classname` is the file that loaded this class, the top
  // of `\@currnamestack` (`\@pushfilename` prepends the loader, latex.ltx). Under a class that `\LoadClass`es it,
  // that is the loading class's name (quantumdust.cls:278-286 `\ClassWarning{\@classname}`, 2606.20943); loaded by
  // `\documentclass`, that entry is empty and the class names itself. Read here, in the class's own load, not in a file it loads.
  RawTeX!(r"\def\@tempa#1#2\@nil{\edef\@classname{#1}}
\expandafter\@tempa\@currnamestack{}{}{}\@nil
\ifx\@classname\@empty \edef\@classname{\@currname}\fi");
  // LoadClass('ams_core', withoptions => 1);
  LoadClass!("ams_core", withoptions => true);
  RequireResource!("ltx-amsart.css");

  // amsart.cls.ltxml : journal article
  // amsproc.cls.ltxml : article in book proceedings
  // amsbook.cls.ltxml : monograph
  // gen-j-l.cls.ltxml "generic journal article" => amsart
  // gen-p-l.cls.ltxml "generic proceedings article" => amsproc
  // gen-m-l.cls.ltxml "monograph" => amsbook

  //======================================================================
  // Sec. 5. Document Body
  // Perl `Let('\specialsection', '\chapter')`: the kernel's chapter, which `\documentclass` takes away while the class
  // loads (amsart has no `\chapter`; OXIDIZED_DESIGN #179), so read under its own name.
  Let!("\\specialsection", "\\lx@kernel@chapter"); // Close enough?
});
