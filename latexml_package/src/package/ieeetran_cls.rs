//! IEEEtran.cls — IEEE Transactions document class
//! Perl: IEEEtran.cls.ltxml — 458 lines
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // DeclareOption stubs — Perl L18-108
  DeclareOption!("9pt", {});
  DeclareOption!("10pt", {});
  DeclareOption!("11pt", {});
  DeclareOption!("12pt", {});
  DeclareOption!("letterpaper", {});
  DeclareOption!("a4paper", {});
  DeclareOption!("cspaper", {});
  DeclareOption!("draft", {});
  DeclareOption!("final", {});
  DeclareOption!("journal", { Let!("\\ifCLASSOPTIONjournal", "\\iftrue"); Let!("\\ifCLASSOPTIONconference", "\\iffalse"); });
  DeclareOption!("conference", { Let!("\\ifCLASSOPTIONjournal", "\\iffalse"); Let!("\\ifCLASSOPTIONconference", "\\iftrue"); });
  DeclareOption!("technote", { Let!("\\ifCLASSOPTIONtechnote", "\\iftrue"); });
  DeclareOption!("nofonttune", {});
  DeclareOption!("captionsoff", {});
  DeclareOption!("compsoc", { Let!("\\ifCLASSOPTIONcompsoc", "\\iftrue"); });
  DeclareOption!("comsoc", { Let!("\\ifCLASSOPTIONcompsoc", "\\iftrue"); });
  DeclareOption!("transmag", {});
  DeclareOption!("romanappendices", { Let!("\\ifCLASSOPTIONromanappendices", "\\iftrue"); });
  DeclareOption!("onecolumn", {});
  DeclareOption!("twocolumn", {});
  DeclareOption!("peerreview", {});
  DeclareOption!("peerreviewca", {});
  ProcessOptions!();

  // Load article as base
  load_class("article", Vec::new(), Tokens!())?;

  // Option conditionals — Perl L18-108
  Let!("\\ifCLASSOPTIONcompsoc", "\\iffalse");
  Let!("\\ifCLASSOPTIONjournal", "\\iftrue");
  Let!("\\ifCLASSOPTIONconference", "\\iffalse");
  Let!("\\ifCLASSOPTIONtechnote", "\\iffalse");
  Let!("\\ifCLASSOPTIONromanappendices", "\\iffalse");
  Let!("\\ifCLASSINFOpdf", "\\iftrue");
  Let!("\\ifCLASSOPTIONonecolumn", "\\iffalse");
  Let!("\\ifCLASSOPTIONtwocolumn", "\\iftrue");
  Let!("\\ifCLASSOPTIONdraftcls", "\\iffalse");
  Let!("\\ifCLASSOPTIONpeerreview", "\\iffalse");
  Let!("\\ifCLASSOPTIONcaptionsoff", "\\iffalse");

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
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\IEEEauthorblockA{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");

  // IEEEkeywords environment (Perl L152-155)
  Let!("\\@endIEEEkeywords", "\\relax");
  DefMacro!("\\@IEEEkeywords XUntil:\\@endIEEEkeywords",
    "\\@add@frontmatter{ltx:keywords}[name={Index Terms}]{#1}");
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
  DefConstructor!("\\IEEEQEDclosed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true);
  Let!("\\IEEEQEDopen", "\\IEEEQEDclosed");
  Let!("\\IEEEQED", "\\IEEEQEDclosed");

  // IEEEproof environment (Perl L206-229)
  // Perl digests \\textbf{\\textit{Proof:}} producing font="bold italic".
  // Our codegen treats \\word as literal text, so use explicit attributes instead.
  DefEnvironment!("{IEEEproof}[]",
    "<ltx:proof><ltx:title font='bold italic' _force_font='true' class='ltx_runin'>Proof:</ltx:title>#body</ltx:proof>");

  // IEEEbiography (Perl L238-247)
  DefEnvironment!("{IEEEbiography}[]{}",
    "<ltx:section class='ltx_biography'><ltx:title>#2</ltx:title>#body</ltx:section>");
  DefEnvironment!("{IEEEbiographynophoto}{}",
    "<ltx:section class='ltx_biography'><ltx:title>#1</ltx:title>#body</ltx:section>");

  // IEEEeqnarray (Perl L299-332) — map to eqnarray
  DefMacro!("\\IEEEeqnarray{}", "\\eqnarray");
  DefMacro!("\\endIEEEeqnarray", "\\endeqnarray");
  DefMacro!("\\IEEEeqnarraynumspace", "");
  DefMacro!("\\IEEEeqnarraybox{}", "\\begin{array}{#1}");
  DefMacro!("\\endIEEEeqnarraybox", "\\end{array}");
  DefMacro!("\\IEEEeqnarraymulticol{}{}{}", "\\multicolumn{#1}{#2}{#3}");
  DefMacro!("\\IEEEeqnarraydefcol{}{}{}", "");
  DefMacro!("\\IEEEeqnarraydefcolsep{}{}", "");

  // IEEEnonumber/yesnumber stubs
  DefMacro!("\\IEEEnonumber OptionalMatch:*", "\\nonumber");
  DefMacro!("\\IEEEyesnumber OptionalMatch:*", "");
  DefMacro!("\\IEEEyessubnumber OptionalMatch:*", "");
  DefMacro!("\\IEEEnosubnumber OptionalMatch:*", "");

  // Column types (Perl L308-314) — DefColumnType not yet ported, skip

  Let!("\\appendices", "\\appendix");

  // Bibliography style — AssignMapping not yet ported, skip

  // IED list stubs (Perl L340-347)
  DefMacro!("\\IEEEsetlabelwidth{}", "\\settowidth{\\labelwidth}{#1}");
  DefMacro!("\\IEEEusemathlabelsep", "");
  DefMacro!("\\IEEEtriggercmd{}", "");
  DefMacro!("\\IEEElabelindent", "");
  DefMacro!("\\IEEEcalcleftmargin{}", "");
  DefMacro!("\\IEEEiedlabeljustifyc", "");
  DefMacro!("\\IEEEiedlabeljustifyl", "");
  DefMacro!("\\IEEEiedlabeljustifyr", "");

  // IEEEitemize/enumerate/description (Perl L351-366)
  DefEnvironment!("{IEEEitemize}[]",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    mode => "internal_vertical");
  DefEnvironment!("{IEEEenumerate}[]",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    mode => "internal_vertical");
  DefEnvironment!("{IEEEdescription}[]",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    mode => "internal_vertical");

  // String macros (Perl L383-395)
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\indexname", "Index");
  DefMacro!("\\partname", "Part");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\abstractname", "Abstract");
  DefMacro!("\\IEEEkeywordsname", "Index Terms");
  DefMacro!("\\IEEEproofname", "Proof");

  // Legacy aliases (Perl L398-439)
  Let!("\\authorblockA", "\\IEEEauthorblockA");
  Let!("\\authorblockN", "\\IEEEauthorblockN");
  Let!("\\authorrefmark", "\\IEEEauthorrefmark");
  Let!("\\PARstart", "\\IEEEPARstart");
  Let!("\\pubid", "\\IEEEpubid");
  Let!("\\pubidadjcol", "\\IEEEpubidadjcol");
  Let!("\\specialpapernotice", "\\IEEEspecialpapernotice");

  // Keywords environment aliases — Perl L406-414
  // Perl dispatches on whether the next token is a brace:
  //   \keywords{foo}  → \keywords@onearg{foo}
  //   \keywords … \endkeywords (env form) → \@IEEEkeywords
  // Rust was hardcoding the env-start path, so braced `\keywords{foo}`
  // never reached the one-arg expansion.
  DefMacro!("\\keywords", sub[_args] {
    let next = gullet::read_token()?;
    if let Some(t) = next {
      gullet::unread(Tokens!(t.clone()));
      if t.get_catcode() == Catcode::BEGIN {
        return Ok(Tokens!(T_CS!("\\keywords@onearg")));
      }
    }
    Ok(Tokens!(T_CS!("\\@IEEEkeywords")))
  }, locked => true);
  DefMacro!("\\keywords@onearg{}",
    "\\@IEEEkeywords #1 \\@endIEEEkeywords");
  DefMacro!("\\endkeywords", "\\@endIEEEkeywords");

  // Legacy IED list aliases
  Let!("\\labelindent", "\\IEEElabelindent");
  Let!("\\calcleftmargin", "\\IEEEcalcleftmargin");
  Let!("\\setlabelwidth", "\\IEEEsetlabelwidth");
  Let!("\\usemathlabelsep", "\\IEEEusemathlabelsep");

  // QED/proof aliases
  Let!("\\QED", "\\IEEEQED");
  Let!("\\QEDclosed", "\\IEEEQEDclosed");
  Let!("\\QEDopen", "\\IEEEQEDopen");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true, reversion => "\\qed");
  Let!("\\proof", "\\IEEEproof");
  Let!("\\endproof", "\\endIEEEproof");

  // Biography aliases
  Let!("\\biography", "\\IEEEbiography");
  Let!("\\biographynophoto", "\\IEEEbiographynophoto");
  Let!("\\endbiography", "\\endIEEEbiography");
  Let!("\\endbiographynophoto", "\\endIEEEbiographynophoto");

  // bstctlcite stub (Perl L445)
  DefMacro!("\\bstctlcite[]{}", "");

  // Disable internal alignment env (Perl L453-454)
  DefMacro!("\\@IEEEauthorhalign", "\\relax");
  DefMacro!("\\end@IEEEauthorhalign", "\\relax");
});
