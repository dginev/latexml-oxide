use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\address[]{}", "\\lx@add@address{#2}");
  // espcrc1.sty:159-162 `\addressmark` takes an optional key, the `\address[<key>]` whose mark it prints
  // (`\@markedaddressmark[#1]`); the binding prints no address marks, so the key is read and dropped. Perl's
  // espcrc.sty.ltxml:22 takes none, and "[ETHZ]" was printed in the author's name (hep-lat0409072; KNOWN_PERL_ERRORS
  // #1122). Guard `perfect_kernel_batch78s::espcrc_addressmark_key_not_printed`.
  def_macro_noop("\\addressmark[]")?;
  DefMacro!("\\addresstext{}{}", "#2");
  DefMacro!("\\filedate",    "24 November 1993");
  DefMacro!("\\fileversion", "v2.6");
  NewCounter!("address");
  DefMacro!("\\theaddress", "\\alph{address}");
  // The 2004 Elsevier CRC styles (espcrc1.sty / espcrc2.sty v1.2, 2004/02/24, spepping; the copy 0906.2868 and
  // hep-ph0610015 ship, espcrc1.sty:27-31) read the version stamp off an RCS `$Id$` keyword: `\readRCS
  // $Id: espcrc2.tex,v 1.2 2004/02/24 11:22:11 spepping Exp $` sets `\filename`, `\fileversion` and `\filedate`, which
  // the template's `\ProvidesFile{espcrc2.tex}[\filedate \space v\fileversion …]` then prints into the log. Delimited
  // by math shifts, so without it the `$Id …$` became preamble math (hep-ex0506002, hep-ph0506127, hep-ex0610011).
  RawTeX!(r"\def\readRCS$#1: #2 #3 #4 #5${\def\filename{#2}\def\fileversion{#3}\def\filedate{#4}}");
  // espcrc1.sty:156-159: `\runtitle` / `\runauthor` store the running heads `\ps@crcheadings` (:232-236, the
  // `headings` option) prints; `\maketitle` falls back to `\@title` / `\@author` (:133-134). Print furniture, so
  // stored and never typeset (astro-ph0610654, cond-mat0610259, hep-ex0506002, hep-ex0610011).
  RawTeX!(r"\def\runtitle#1{\gdef\@runtitle{#1}}
\def\runauthor#1{\gdef\@runauthor{#1}}
\let\@runauthor\relax
\let\@runtitle\relax");
});
