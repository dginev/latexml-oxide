//! IEEEtran.cls — IEEE Transactions document class
//! Perl: IEEEtran.cls.ltxml — 458 lines
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Option conditionals — use DefConditional! instead of TeX! \newif (avoids compile-time OOM)
  DefConditional!("\\ifCLASSOPTIONcompsoc", { false });
  DefConditional!("\\ifCLASSOPTIONjournal", { true });
  DefConditional!("\\ifCLASSOPTIONconference", { false });
  DefConditional!("\\ifCLASSOPTIONtechnote", { false });
  DefConditional!("\\ifCLASSOPTIONromanappendices", { false });
  DefConditional!("\\ifCLASSINFOpdf", { true });

  // Front matter macros (Perl L134-165)
  DefMacro!("\\IEEEtitleabstractindextext{}", "#1");
  DefMacro!("\\IEEEdisplaynontitleabstractindextext", "");
  DefMacro!("\\IEEEdisplaynotcompsoctitleabstractindextext", "");
  DefMacro!("\\IEEEcompsoctitleabstractindextext", "");
  Let!("\\IEEEpeerreviewmaketitle", "\\maketitle");
  DefMacro!("\\IEEEoverridecommandlockouts", "");
  DefMacro!("\\overrideIEEEmargins", "");
  DefMacro!("\\IEEEaftertitletext{}", "");
  DefMacro!("\\IEEEspecialpapernotice{}", "");
  DefMacro!("\\IEEEmembership{}", "");
  DefMacro!("\\IEEEauthorblockN{}", "#1");
  // DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\IEEEauthorblockA{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");

  // IEEEkeywords environment (Perl L152-155)
  Let!("\\@endIEEEkeywords", "\\relax");
  DefMacro!("\\@IEEEkeywords XUntil:\\@endIEEEkeywords",
    "\\@add@frontmatter{ltx:keywords}[name={\\IEEEkeywordsname}]{#1}");
  DefMacro!("\\IEEEkeywords", "\\@IEEEkeywords");
  DefMacro!("\\endIEEEkeywords", "\\@endIEEEkeywords");

  DefMacro!("\\IEEEraisesectionheading{}", "#1");
  DefMacro!("\\IEEEPARstart{}{}", "#1#2");
  DefMacro!("\\IEEEcompsocitemizethanks{}", "\\thanks{#1}");
  DefMacro!("\\IEEEcompsocthanksitem[]", "");
  DefMacro!("\\IEEEauthorrefmark", "");
  DefMacro!("\\IEEEtriggeratref{}", "");
  DefMacro!("\\IEEEpubid{}", "\\@add@frontmatter{ltx:note}[role=publicationid]{pubid: #1}");
  DefMacro!("\\IEEEpubidadjcol", "");

  // Section numbering — default journal mode uses Roman numerals
  DefMacro!("\\thesection", "\\Roman{section}");
  DefMacro!("\\thesubsection", "\\mbox{\\thesection-\\Alph{subsection}}");
  DefMacro!("\\thesubsubsection", "\\thesubsection\\arabic{subsubsection}");
  DefMacro!("\\theparagraph", "\\thesubsubsection\\alph{paragraph}");

  // Font primitives (Perl L183-186)
  DefPrimitive!("\\ltx@ieeetran@it", None, font => { shape => "italic", family => "serif", series => "medium" }, locked => true);
  DefPrimitive!("\\ltx@ieeetran@sc", None, font => { shape => "smallcaps", family => "serif", series => "medium" }, locked => true);
  DefMacro!("\\format@title@font@section", "\\ltx@ieeetran@sc");
  DefMacro!("\\format@title@font@subsection", "\\ltx@ieeetran@it");
  DefMacro!("\\figurename", "Fig.");
  DefMacro!("\\tablename", "TABLE");
  DefMacro!("\\thetable", "\\Roman{table}");

  // QED symbols (Perl L194-198)
  // DefConstructor!("\\IEEEQEDclosed",
  //   "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
  //   enter_horizontal => true);
  Let!("\\IEEEQEDopen", "\\IEEEQEDclosed");
  Let!("\\IEEEQED", "\\IEEEQEDclosed");

  // IEEEproof environment (Perl L206-229)

  // ERROR! INFINITE LOOP IN rustc COMPILATION !!!
  //
  // DefEnvironment!("{IEEEproof}[]",
  //   "<ltx:proof class='ltx_runin'><ltx:title font='italic' _force_font='true' class='ltx_runin'>\\textbf{\\textit{Proof:}}</ltx:title>#body</ltx:proof>");
  DefEnvironment!("{IEEEproof}[]",
    "<ltx:proof class='ltx_runin'><ltx:title font='italic' class='ltx_runin'>Proof:</ltx:title>#body</ltx:proof>");

  // // IEEEeqnarray (Perl L299-332) — map to eqnarray
  // DefMacro!("\\IEEEeqnarray{}", "\\eqnarray");
  // DefMacro!("\\endIEEEeqnarray", "\\endeqnarray");
  // // Starred variants
  // DefMacro!("\\IEEEeqnarray*{}", "\\csname eqnarray*\\endcsname");
  // Let!("\\endIEEEeqnarray*", "\\csname endeqnarray*\\endcsname");
  // DefMacro!("\\IEEEeqnarraynumspace", "");

  // // IEEEnonumber/yesnumber stubs
  // DefMacro!("\\IEEEnonumber OptionalMatch:*", "\\nonumber");
  // DefMacro!("\\IEEEyesnumber OptionalMatch:*", "");
  // DefMacro!("\\IEEEyessubnumber OptionalMatch:*", "");
  // DefMacro!("\\IEEEnosubnumber OptionalMatch:*", "");

  // // Column types (Perl L308-314)
  // DefColumnType!('L', "\\hfil", "");
  // DefColumnType!('C', "\\hfil", "\\hfil");
  // DefColumnType!('R', "", "\\hfil");

  // Let!("\\appendices", "\\appendix");

  // // Bibliography style
  // AssignMapping!("BIBSTYLES_IEEEtran", "citestyle" => "numbers", "sort" => "true");

  // // IED list stubs (Perl L340-347)
  // DefMacro!("\\IEEEsetlabelwidth{}", "\\settowidth{\\labelwidth}{#1}");
  // DefMacro!("\\IEEEusemathlabelsep", "");
  // DefMacro!("\\IEEEtriggercmd{}", "");
  // DefMacro!("\\IEEElabelindent", "");
  // DefMacro!("\\IEEEcalcleftmargin{}", "");
  // DefMacro!("\\IEEEiedlabeljustifyc", "");
  // DefMacro!("\\IEEEiedlabeljustifyl", "");
  // DefMacro!("\\IEEEiedlabeljustifyr", "");

  // // IEEEitemize/enumerate/description (Perl L351-366)
  // DefEnvironment!("{IEEEitemize}[]",
  //   "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
  //   mode => "internal_vertical");
  // DefEnvironment!("{IEEEenumerate}[]",
  //   "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
  //   mode => "internal_vertical");
  // DefEnvironment!("{IEEEdescription}[]",
  //   "<ltx:description xml:id='#id'>#body</ltx:description>",
  //   mode => "internal_vertical");

  // // String macros (Perl L383-395)
  // DefMacro!("\\contentsname", "Contents");
  // DefMacro!("\\listfigurename", "List of Figures");
  // DefMacro!("\\listtablename", "List of Tables");
  // DefMacro!("\\refname", "References");
  // DefMacro!("\\indexname", "Index");
  // DefMacro!("\\partname", "Part");
  // DefMacro!("\\appendixname", "Appendix");
  // DefMacro!("\\abstractname", "Abstract");
  // DefMacro!("\\IEEEkeywordsname", "Index Terms");
  // DefMacro!("\\IEEEproofname", "Proof");

  // // Legacy aliases (Perl L398-439)
  // Let!("\\authorblockA", "\\IEEEauthorblockA");
  // Let!("\\authorblockN", "\\IEEEauthorblockN");
  // Let!("\\authorrefmark", "\\IEEEauthorrefmark");
  // Let!("\\PARstart", "\\IEEEPARstart");
  // Let!("\\pubid", "\\IEEEpubid");
  // Let!("\\pubidadjcol", "\\IEEEpubidadjcol");
  // Let!("\\specialpapernotice", "\\IEEEspecialpapernotice");

  // // Keywords environment aliases
  // DefMacro!("\\keywords", "\\@IEEEkeywords");
  // DefMacro!("\\endkeywords", "\\@endIEEEkeywords");

  // // QED/proof aliases
  // Let!("\\QED", "\\IEEEQED");
  // Let!("\\QEDclosed", "\\IEEEQEDclosed");
  // Let!("\\QEDopen", "\\IEEEQEDopen");
  // DefMacro!("\\qed", "\\ltx@qed");
  // // DefConstructor!("\\ltx@qed",
  // //   "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
  //   // enter_horizontal => true, reversion => "\\qed");
  // Let!("\\proof", "\\IEEEproof");
  // Let!("\\endproof", "\\endIEEEproof");

  // // bstctlcite stub (Perl L445)
  // DefMacro!("\\bstctlcite[]{}", "");

  // // Disable internal alignment env (Perl L453-454)
  // DefMacro!("\\@IEEEauthorhalign", "\\relax");
  // DefMacro!("\\end@IEEEauthorhalign", "\\relax");
});
