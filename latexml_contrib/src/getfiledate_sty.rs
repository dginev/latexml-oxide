//! Stub for getfiledate.sty — prints a file's last-modification date
//! (a boxed/inline date string). Output-irrelevant for our purposes.
//!
//! Why a stub: getfiledate.sty L20 `\RequirePackage{ltxnew}` and uses
//! its `\new` allocator prefix at load time —
//! `\new\dimen\gfd@width@tmp\gfd@width@tmp=\z@` (L29). ltxnew builds
//! `\new` via a `\futurelet`-based prefix scanner (a long `\ifx\x\dimen
//! → \newdimen` chain) that our raw-load doesn't execute faithfully, so
//! `\new\dimen\gfd@width@tmp` never allocates the register and the
//! immediately-following `\gfd@width@tmp=\z@` raises
//! `Error:undefined:\gfd@width@tmp` at package-load time.
//!
//! Perl LaTeXML matches the EFFECTIVE skip: it reports getfiledate as a
//! missing-file (`Can't find binding for package getfiledate`) and only
//! deps-scans it, never executing the body — so the paper converts
//! cleanly with `\getfiledate` simply absent. We mirror that by stubbing
//! the public `\getfiledate` as a no-op (the 4 witness papers
//! 1503.08338/1503.08341/1709.04899/1803.07118 all `\usepackage` it but
//! never call it). A future paper that DOES call `\getfiledate` loses
//! only the decorative file-date string — same as Perl today.
use latexml_package::prelude::*;

LoadDefinitions!({
  // getfiledate.sty L22 does `\RequirePackage[table]{xcolor}` (guarded by
  // \@ifpackageloaded). Perl's deps-scan loads xcolor too, so a paper
  // relying on getfiledate to transitively provide \textcolor/\color
  // still works. Mirror that — without it, stubbing getfiledate drops
  // xcolor and `\textcolor` goes undefined (witness 1803.07118).
  RequirePackage!("xcolor", options => vec!["table".to_string()]);
  // \getfiledate[<keyvals>]{<file>} — newcommand\getfiledate[2][] in the
  // real package (one optional arg, one required). Consume and emit
  // nothing, matching Perl's effective skip.
  def_macro_noop("\\getfiledate[]{}")?;
});
