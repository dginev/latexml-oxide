//! IEEEtran.cls — IEEE Transactions document class
//! Perl: IEEEtran.cls.ltxml — 458 lines
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Option conditionals (Perl L18-108)
  TeX!(r"
\newif\ifCLASSOPTIONonecolumn       \CLASSOPTIONonecolumnfalse
\newif\ifCLASSOPTIONtwocolumn       \CLASSOPTIONtwocolumntrue
\newif\ifCLASSOPTIONoneside         \CLASSOPTIONonesidetrue
\newif\ifCLASSOPTIONtwoside         \CLASSOPTIONtwosidefalse
\newif\ifCLASSOPTIONfinal           \CLASSOPTIONfinaltrue
\newif\ifCLASSOPTIONdraft           \CLASSOPTIONdraftfalse
\newif\ifCLASSOPTIONdraftcls        \CLASSOPTIONdraftclsfalse
\newif\ifCLASSOPTIONdraftclsnofoot  \CLASSOPTIONdraftclsnofootfalse
\newif\ifCLASSOPTIONpeerreview      \CLASSOPTIONpeerreviewfalse
\newif\ifCLASSOPTIONpeerreviewca    \CLASSOPTIONpeerreviewcafalse
\newif\ifCLASSOPTIONjournal         \CLASSOPTIONjournaltrue
\newif\ifCLASSOPTIONconference      \CLASSOPTIONconferencefalse
\newif\ifCLASSOPTIONtechnote        \CLASSOPTIONtechnotefalse
\newif\ifCLASSOPTIONnofonttune      \CLASSOPTIONnofonttunefalse
\newif\ifCLASSOPTIONcaptionsoff     \CLASSOPTIONcaptionsofffalse
\newif\ifCLASSOPTIONcomsoc          \CLASSOPTIONcomsocfalse
\newif\ifCLASSOPTIONcompsoc         \CLASSOPTIONcompsocfalse
\newif\ifCLASSOPTIONtransmag        \CLASSOPTIONtransmagfalse
\newif\ifCLASSOPTIONromanappendices \CLASSOPTIONromanappendicesfalse
\newif\ifCLASSINFOpdf               \CLASSINFOpdftrue
\DeclareOption{9pt}{\def\CLASSOPTIONpt{9}\def\@ptsize{0}}
\DeclareOption{10pt}{\def\CLASSOPTIONpt{10}\def\@ptsize{0}}
\DeclareOption{11pt}{\def\CLASSOPTIONpt{11}\def\@ptsize{1}}
\DeclareOption{12pt}{\def\CLASSOPTIONpt{12}\def\@ptsize{2}}
\DeclareOption{letterpaper}{\def\CLASSOPTIONpaper{letter}}
\DeclareOption{a4paper}{\def\CLASSOPTIONpaper{a4}}
\DeclareOption{oneside}{\CLASSOPTIONonesidetrue\CLASSOPTIONtwosidefalse}
\DeclareOption{twoside}{\CLASSOPTIONtwosidetrue\CLASSOPTIONonesidefalse}
\DeclareOption{onecolumn}{\CLASSOPTIONonecolumntrue\CLASSOPTIONtwocolumnfalse}
\DeclareOption{twocolumn}{\CLASSOPTIONtwocolumntrue\CLASSOPTIONonecolumnfalse}
\DeclareOption{draft}{\CLASSOPTIONdrafttrue\CLASSOPTIONdraftclstrue}
\DeclareOption{draftcls}{\CLASSOPTIONdraftfalse\CLASSOPTIONdraftclstrue}
\DeclareOption{final}{\CLASSOPTIONdraftfalse\CLASSOPTIONdraftclsfalse}
\DeclareOption{journal}{\CLASSOPTIONjournaltrue\CLASSOPTIONconferencefalse\CLASSOPTIONtechnotefalse}
\DeclareOption{conference}{\CLASSOPTIONjournalfalse\CLASSOPTIONconferencetrue\CLASSOPTIONtechnotefalse}
\DeclareOption{technote}{\CLASSOPTIONjournalfalse\CLASSOPTIONconferencefalse\CLASSOPTIONtechnotetrue}
\DeclareOption{peerreview}{\CLASSOPTIONpeerreviewtrue\CLASSOPTIONjournalfalse}
\DeclareOption{peerreviewca}{\CLASSOPTIONpeerreviewtrue\CLASSOPTIONpeerreviewcatrue}
\DeclareOption{nofonttune}{\CLASSOPTIONnofonttunetrue}
\DeclareOption{captionsoff}{\CLASSOPTIONcaptionsofftrue}
\DeclareOption{comsoc}{\CLASSOPTIONcomsoctrue\CLASSOPTIONcompsocfalse\CLASSOPTIONtransmagfalse}
\DeclareOption{compsoc}{\CLASSOPTIONcomsocfalse\CLASSOPTIONcompsoctrue\CLASSOPTIONtransmagfalse}
\DeclareOption{transmag}{\CLASSOPTIONtransmagtrue\CLASSOPTIONcomsocfalse\CLASSOPTIONcompsocfalse}
\DeclareOption{romanappendices}{\CLASSOPTIONromanappendicestrue}
  ");

  // Pass unknown options to article
  DeclareOption!(None, sub {
    let opt = Expand!(T_CS!("\\CurrentOption")).to_string();
    pass_options("article", "cls", &[&opt])?;
  });
  ProcessOptions!();
  LoadClass!("article");

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
  DefMacro!("\\csname begin{IEEEkeywords}\\endcsname", "\\@IEEEkeywords");
  DefMacro!("\\csname end{IEEEkeywords}\\endcsname", "\\@endIEEEkeywords");
  Let!("\\@endIEEEkeywords", "\\relax");
  DefMacro!("\\@IEEEkeywords XUntil:\\@endIEEEkeywords",
    "\\@add@frontmatter{ltx:keywords}[name={\\IEEEkeywordsname}]{#1}");

  DefMacro!("\\IEEEraisesectionheading{}", "#1");
  DefMacro!("\\IEEEPARstart{}{}", "#1#2");
  DefMacro!("\\IEEEcompsocitemizethanks{}", "\\thanks{#1}");
  DefMacro!("\\IEEEcompsocthanksitem[]", "");
  DefMacro!("\\IEEEauthorrefmark", "");
  DefMacro!("\\IEEEtriggeratref{}", "");
  DefMacro!("\\IEEEpubid{}", "\\@add@frontmatter{ltx:note}[role=publicationid]{pubid: #1}");
  DefMacro!("\\IEEEpubidadjcol", "");

  // Section numbering (Perl L167-182)
  TeX!(r"
\ifCLASSOPTIONcompsoc
\def\thesection{\arabic{section}}
\def\thesubsection{\thesection.\arabic{subsection}}
\def\thesubsubsection{\thesubsection.\arabic{subsubsection}}
\else
\def\thesection{\Roman{section}}
\def\thesubsection{\mbox{\thesection-\Alph{subsection}}}
\def\thesubsubsection{\thesubsection\arabic{subsubsection}}
\def\theparagraph{\thesubsubsection\alph{paragraph}}
\fi
  ");

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
  DefEnvironment!("{IEEEproof}[]",
    "<ltx:proof class='ltx_runin'>\
     <ltx:title font='italic' _force_font='true' class='ltx_runin'>\\textbf{\\textit{Proof:}}</ltx:title>\
     #body\
     </ltx:proof>");

  // Lengths (Perl L231-236)
  TeX!(r"
\newlength\abovecaptionskip
\newlength\belowcaptionskip
\setlength\abovecaptionskip{0.5\baselineskip}
\setlength\belowcaptionskip{0pt}
  ");

  // IEEEbiography environments (Perl L238-247)
  DefEnvironment!("{IEEEbiography}[]{}",
    "<ltx:float class='biography'><ltx:tabular>\
     <ltx:tr><ltx:td>#1</ltx:td><ltx:td><ltx:inline-block><ltx:text class='ltx_font_bold'>#2</ltx:text> \
     #body</ltx:inline-block></ltx:td></ltx:tr>\
     </ltx:tabular></ltx:float>");
  DefEnvironment!("{IEEEbiographynophoto}[]{}",
    "<ltx:float class='biography'><ltx:tabular>\
     <ltx:tr><ltx:td><ltx:inline-block><ltx:text class='ltx_font_bold'>#2</ltx:text> \
     #body</ltx:inline-block></ltx:td></ltx:tr>\
     </ltx:tabular></ltx:float>");

  // IEEEeqnarray (Perl L299-332) — map to eqnarray
  DefMacro!("\\csname IEEEeqnarray\\endcsname{}", "\\eqnarray");
  Let!("\\csname endIEEEeqnarray\\endcsname", "\\endeqnarray");
  DefMacro!("\\csname IEEEeqnarray*\\endcsname{}", "\\csname eqnarray*\\endcsname");
  Let!("\\csname endIEEEeqnarray*\\endcsname", "\\csname endeqnarray*\\endcsname");
  DefMacro!("\\IEEEeqnarraynumspace", "");

  // IEEEnonumber/yesnumber stubs (Perl L252-294) — simplified
  DefMacro!("\\IEEEnonumber OptionalMatch:*", "\\nonumber");
  DefMacro!("\\IEEEyesnumber OptionalMatch:*", "");
  DefMacro!("\\IEEEyessubnumber OptionalMatch:*", "");
  DefMacro!("\\IEEEnosubnumber OptionalMatch:*", "");

  // Column types (Perl L308-314)
  DefColumnType!('L', "\\hfil", "");
  DefColumnType!('C', "\\hfil", "\\hfil");
  DefColumnType!('R', "", "\\hfil");

  Let!("\\appendices", "\\appendix");

  // Bibliography style
  AssignMapping!("BIBSTYLES_IEEEtran", "citestyle" => "numbers", "sort" => "true");

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

  // Override standard lists (Perl L369-380)
  Let!("\\itemize", "\\IEEEitemize");
  Let!("\\enditemize", "\\endIEEEitemize");
  Let!("\\enumerate", "\\IEEEenumerate");
  Let!("\\endenumerate", "\\endIEEEenumerate");
  Let!("\\description", "\\IEEEdescription");
  Let!("\\enddescription", "\\endIEEEdescription");

  // String macros (Perl L383-395)
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\indexname", "Index");
  DefMacro!("\\figurename", "Figure");
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

  // Keywords environment aliases
  DefMacro!("\\csname begin{keywords}\\endcsname", "\\@IEEEkeywords");
  DefMacro!("\\csname end{keywords}\\endcsname", "\\@endIEEEkeywords");

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
  DefMacro!("\\csname begin{@IEEEauthorhalign}\\endcsname", "\\relax");
  DefMacro!("\\csname end{@IEEEauthorhalign}\\endcsname", "\\relax");
});
