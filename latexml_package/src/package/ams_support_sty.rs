use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ams_support.sty.ltxml — common support for AMS document classes

  //======================================================================
  // Document structure.

  Let!("\\@xp", "\\expandafter");
  Let!("\\@nx", "\\noexpand");

  // None of the options are vital, I think; deferred.
  // [though loading an unwanted amsfonts (noamsfonts) could be an issue]
  for option in [
    "a4paper", "letterpaper", "landscape", "portrait",
    "oneside", "twoside", "draft", "final", "e-only",
    "titlepage", "notitlepage",
    "openright", "openany", "onecolumn", "twocolumn",
    "nomath", "noamsfonts", "psamsfonts",
    "leqno", "reqno", "centertags", "tbtags", "fleqn",
    "8pt", "9pt", "10pt", "11pt", "12pt",
    "makeidx",
  ].iter() {
    DeclareOption!(*option, None);
  }
  ProcessOptions!();

  //======================================================================
  // Font size commands:

  DefPrimitive!("\\larger",  None, font => { scale => 1.2 });
  DefPrimitive!("\\smaller", None, font => { size => 0.8333333333333334 }); // 1/1.2

  // \@xsetfontize
  DefPrimitive!("\\TINY", None, font => { size => 3 });
  DefPrimitive!("\\Tiny", None, font => { size => 4 });
  Let!("\\SMALL", "\\scriptsize");
  Let!("\\Small", "\\footnotesize");
  DefPrimitive!("\\HUGE", None, font => { size => 29.8 });
  Let!("\\upn", "\\textup");

  //======================================================================
  // Sec. 3. The Preamble
  // Included packages
  // amsmath, amsthm,
  // amsfonts (unless noamsfonts)

  RequirePackage!("amsmath");
  // RequirePackage!("amstex") if LookupValue('2.09_COMPATIBILITY');
  RequirePackage!("amsthm");
  RequirePackage!("amsfonts");
  RequirePackage!("makeidx");

  // Useful packages:
  // amssymb,
  // amsmidx for multiple-indexes,
  // graphicx,
  // longtable,
  // upref makes references upcase?, upright?
  // xypic,

  //======================================================================
  // Sec. 4. Top Matter
  // FrontMatter:
  DefMacro!("\\shorttitle{}", "\\@add@frontmatter{ltx:toctitle}{#1}");
  DefMacro!("\\shortauthor{}", "");   // Not useful?
  DefMacro!("\\authors{}", "");
  DefMacro!("\\shortauthors{}", "");
  DefMacro!("\\addresses{}", "");
  DefMacro!("\\publname{}", "");

  DefMacro!("\\title[]{}",
    "\\if.#1.\\else\\def\\shorttitle{#1}\\@add@frontmatter{ltx:toctitle}{#1}\\fi\\@add@frontmatter{ltx:title}{#2}");

  DefMacro!("\\lx@author@sep", ",\\ ");
  DefMacro!("\\lx@author@conj", "\\ and\\ ");   // \@@and

  DefMacro!("\\author[]{}",
    "\\if.#1.\\else\\def\\shortauthor{#1}\\fi\\def\\@author{#2}\\lx@author{#2}");

  DefMacro!("\\datename", None, "\\textit{Date}:");

  DefMacro!("\\contrib[]{}",
    "\\@add@frontmatter{ltx:creator}[role=contributor]{\\@personname{#2}}");

  DefMacro!("\\commby{}",
    "\\@add@frontmatter{ltx:creator}[role=communicator]{\\@personname{#1}}");

  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#2}}");

  DefConstructor!("\\@@@curraddr{}", "^ <ltx:contact role='current_address'>#1</ltx:contact>");
  DefMacro!("\\curraddr{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@curraddr{#1}}");

  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#2}}");

  DefConstructor!("\\@@@urladdr{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\urladdr{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@urladdr{#1}}");

  DefConstructor!("\\@@@dedicatory{}", "^ <ltx:contact role='dedicatory'>#1</ltx:contact>");
  DefMacro!("\\dedicatory{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@dedicatory{#1}}");

  // \date{}
  DefMacro!("\\dateposted{}", "\\@add@frontmatter{ltx:date}[role=posted]{#1}");

  // \thanks{} ( == ack, not latex's \thanks, not in author)
  // make a throwaway optional argument available for OmniBus use
  DefMacro!("\\thanks[]{}",
    "\\@add@frontmatter{ltx:acknowledgements}[name={\\@ifundefined{thanksname}{}{\\thanksname}}]{#2}");

  DefMacro!("\\translator[]{}",
    "\\@add@frontmatter{ltx:creator}[role=translator]{\\@personname{#2}}");

  DefMacro!("\\keywordsname", None, "Key words and phrases");
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:keywords}[name={\\keywordsname}]{#1}");

  // Non-standard but makes it easier to create bindings for variations on AMS classes;
  // just redefine this macro
  DefMacro!("\\@subjclassyear", None, "1991");

  DefMacro!("\\subjclassname", None,
    "\\textup{\\@subjclassyear} Mathematics Subject Classification");
  // Perl: DefMacro('\subjclass[Default:\@subjclassyear]{}', ...);
  // The Default: syntax provides \@subjclassyear as default for the optional arg.
  // Implement by splitting into two macros: one that handles the default.
  DefMacro!("\\subjclass[]{}", "\\lx@subjclass@{#1}{#2}");
  DefMacro!("\\lx@subjclass@{}{}", sub[args] {
    let year_str = args[0].to_string();
    let body_str = args[1].to_string();
    // If year is empty, use current \@subjclassyear; otherwise update it
    let effective_year = if year_str.trim().is_empty() {
      let expanded = gullet::do_expand(Tokens!(T_CS!("\\@subjclassyear")))?;
      expanded.to_string()
    } else {
      // Update \@subjclassyear
      def_macro(T_CS!("\\@subjclassyear"), None,
        Some(ExpansionBody::from(year_str.as_str())), None)?;
      year_str
    };
    let expansion = s!(
      "\\@add@frontmatter{{ltx:classification}}[scheme={{{} Mathematics Subject Classification}},name={{\\subjclassname}}]{{{}}}",
      effective_year, body_str);
    Ok(mouth::tokenize_internal(&expansion))
  });

  DefMacro!("\\copyrightinfo{}{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{\\copyright #1: #2}");

  DefMacro!("\\pagespan{}{}", ""); // ?
  DefMacro!("\\PII{}",
    "\\@add@frontmatter{ltx:classification}[scheme=PII]{#1}");
  DefMacro!("\\ISSN{}",
    "\\@add@frontmatter{ltx:classification}[scheme=ISSN]{#1}");

  DefMacro!("\\currentvolume", None, "");
  DefMacro!("\\currentissue", None, "");
  DefMacro!("\\currentmonth", None, "");
  DefMacro!("\\currentyear", None, "");
  DefMacro!("\\volinfo", None, "");
  DefMacro!("\\issueinfo{}{}{}{}",
    "\\def\\currentvolume{#1}\\def\\currentissue{#2}\\def\\currentmonth{#3}\\def\\currentyear{#4}\\def\\volinfo{Volume \\currentvolume, Number \\number0\\currentissue, \\currentmonth\\ \\currentyear}\\@add@frontmatter{ltx:note}[role=volume-info]{\\volinfo}");

  // abstract otherwise defined in LaTeX.pool
  DefMacro!("\\abstractname", None, "\\textsc{Abstract}");

  //======================================================================
  // Sec. 5. Document Body

  // Mostly normal LaTeX

  // For multiple indexes:
  // \usepackage{amsmidex}
  // \makeindex{name of index file}
  // \makeindex{name of index file}
  //
  // \index{name of index}{index term}   ...
  // \Printindex{name of index}{title of index} ...

  DefMacro!("\\format@title@abstract{}", "#1. ");
  DefMacro!("\\format@title@section{}", "\\lx@tag[][.\\space]{\\thesection}#1");
  DefMacro!("\\format@title@subsection{}", "\\lx@tag[][.\\space]{\\thesubsection}#1");
  DefMacro!("\\format@title@subsubsection{}", "\\lx@tag[][.\\space]{\\thesubsubsection}#1");

  DefMacro!("\\format@title@description{}", "\\lx@tag[][:\\space]{#1}");
  DefMacro!("\\descriptionlabel{}", "\\normalfont\\bfseries #1:\\space");

  //======================================================================
  // Sec 6. Floating objects: Figures and tables
  // Normal LaTeX

  // For compatibility
  // Note: 2.09_COMPATIBILITY support skipped (rarely used)

  DefMacro!("\\format@title@figure{}", "\\lx@tag[][. ]{\\lx@fnum@@{figure}}#1");
  DefMacro!("\\format@title@table{}", "\\lx@tag[][. ]{\\lx@fnum@@{table}}#1");

  // Excersise environments ??:
  // xca "must be defined with \theoremstyle{definition} and \newtheorem ???
  // xcb only for monographs, at end of chapter

  //======================================================================
  // Sec 7. Bibliographic References
  // \bibliographicstyle{}  amsplain or amsalpha
  // \bibliography{bibfile}
  // Normal LaTeX

  DefMacro!("\\bysame", " by same author");
  DefMacro!("\\bibsetup", None, "");

  //======================================================================
  // Sec 8 Monograph Formatting:

  // TOC's should be built by latexml... ?
  DefMacro!("\\tocpart{}{}{}", "");
  DefMacro!("\\tocchapter{}{}{}", "");
  DefMacro!("\\tocsection{}{}{}", "");
  DefMacro!("\\tocsubsection{}{}{}", "");
  DefMacro!("\\tocsubsubsection{}{}{}", "");
  DefMacro!("\\tocparagraph{}{}{}", "");
  DefMacro!("\\tocsubparagraph{}{}{}", "");
  DefMacro!("\\tocappendix{}{}{}", "");
  DefMacro!("\\contentsnamefont", None, "\\scshape");

  DefMacro!("\\labelenumi", None, "(\\theenumi)");
  DefMacro!("\\labelenumii", None, "(\\theenumii)");
  DefMacro!("\\labelenumiii", None, "(\\theenumiii)");
  DefMacro!("\\labelenumiv", None, "(\\theenumiv)");

  DefRegister!("\\normaltopskip"    => Glue!("10pt"));
  DefRegister!("\\linespacing"      => Dimension::from_str("1pt")?);
  DefRegister!("\\normalparindent"  => Dimension::from_str("12pt")?);
  DefRegister!("\\abovecaptionskip" => Glue!("12pt"));
  DefRegister!("\\belowcaptionskip" => Glue!("12pt"));
  DefRegister!("\\captionindent"    => Glue!("3pc"));
  DefPrimitive!("\\nonbreakingspace", "\u{00A0}");
  DefMacro!("\\fullwidthdisplay", None, "");
  DefRegister!("\\listisep" => Glue::new(0));

  DefMacro!("\\calclayout", None, "");
  DefMacro!("\\indentlabel", None, "");

  //======================================================================
  DefMacro!("\\@True", None, "00");
  DefMacro!("\\@False", None, "01");

  // \newswitch, \setFalse, \setTrue — complex sub closures, stubbed as no-ops
  DefMacro!("\\newswitch[]{}", "");
  DefMacro!("\\setFalse{}", "");
  DefMacro!("\\setTrue{}", "");

  // funny control structures, using above switches
  // \except
  // \for
  // \forany

  DefMacro!("\\Mc", None, "Mc");

  // Generated comma and "and" separated lists...
  // \andify, \xandlist, \nxandlist

  //======================================================================

  DefMacro!("\\URLhref{}", "");
  // \URL — complex catcode manipulation, stubbed as simple macro
  DefMacro!("\\URL{}", "#1");

  DefMacro!("\\MR{}", "MR #1");
  DefMacro!("\\MRhref{}", "");
});
