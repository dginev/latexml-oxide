//! acmart.cls — ACM article class
//! Perl: acmart.cls.ltxml (259 lines)
use crate::{
  engine::latex_constructs::{after_float, before_float},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: LoadClass('amsart', withoptions => 1)
  load_class_with_options("amsart", Tokens!())?;

  // Beyond-Perl fidelity (OXIDIZED_DESIGN "acmart establishes T1 font
  // encoding"): real acmart.cls loads libertine + `\RequirePackage[T1]{fontenc}`
  // (acmart.cls L867-881), so `<`/`>`/`|`/`\`/`{`/`}`/`_`/`"` are LITERAL in the
  // PDF. Neither LaTeXML binding modeled this, so both defaulted to OT1 where
  // `<`->¡, `>`->¿ (witness arXiv:2405.17739 `num < 0 && num > 0`). Perl leaves
  // it at OT1; we honor acmart's real T1 to match the PDF. Divergence from Perl.
  RequirePackage!("fontenc", options => vec!["T1".to_string()]);

  RequirePackage!("fancyhdr");
  RequirePackage!("geometry");
  RequirePackage!("comment");
  RequirePackage!("natbib");
  RequirePackage!("textcomp");
  RequirePackage!("graphicx");
  // Real acmart.cls passes [prologue,table]{xcolor} but doesn't pass
  // dvipsnames; many user papers nevertheless use Cerulean / ForestGreen
  // etc. without an explicit \\usepackage[dvipsnames]{xcolor}. Pre-load
  // the extended palette eagerly so the named colors resolve. Witness
  // 2 acmart papers/100k cluster with `Error:unexpected:ForestGreen`.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string()]);
  // RequirePackage('totpages');
  RequirePackage!("microtype");
  RequirePackage!("hyperref");
  RequirePackage!("caption");
  RequirePackage!("float");
  // RequirePackage('environ');
  // RequirePackage('zi4');
  RequirePackage!("setspace");
  RequirePackage!("newtxmath");
  // RequirePackage('manyfoot');
  // RequirePackage('libertine');

  // Added based on acmart.cls in texlive 2020
  RequirePackage!("xkeyval");
  // RequirePackage('xstring');
  RequirePackage!("iftex");
  RequirePackage!("etoolbox");
  RequirePackage!("booktabs");
  RequirePackage!("refcount");
  RequirePackage!("textcase");
  RequirePackage!("hyperxmp");
  // RequirePackage('draftwatermark');
  // RequirePackage('cmap');
  // RequirePackage('pbalance');
  RequirePackage!("balance");

  //======================================================================
  // Various bits of frontmatter
  DefMacro!("\\copyrightyear{}", "\\lx@add@copyrightyear{#1}");
  // This should be keyvals!
  DefMacro!("\\setcopyright{}", "\\lx@add@copyright{#1}");
  DefMacro!("\\received[]{}", "\\lx@add@date[role=received]{#2}");
  DefMacro!("\\acmJournal{}", "\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\acmSubmissionID{}", "\\lx@add@pubnote[role=submissionid]{#1}");
  DefMacro!("\\acmConference[]{}{}{}", "\\lx@add@pubnote[role=conference]{#2; #3; #4}");
  DefMacro!("\\acmBooktitle{}", "\\lx@add@pubnote[role=booktitle]{#1}");
  DefMacro!("\\acmArticle{}", "\\lx@add@pubnote[role=article]{#1}");
  DefMacro!("\\acmArticleSeq{}", "\\lx@add@pubnote[role=articleseq]{#1}");
  DefMacro!("\\acmDOI{}", "\\lx@add@pubnote[role=doi]{#1}");
  DefMacro!("\\acmISBN{}", "\\lx@add@pubnote[role=isbn]{#1}");
  DefMacro!("\\acmMonth{}", "\\lx@add@pubnote[role=publicationmonth]{#1}");
  DefMacro!("\\acmNumber{}", "\\lx@add@pubnote[role=number]{#1}");
  DefMacro!("\\acmPrice{}", "\\lx@add@pubnote[role=price,name={Price:~}]{#1}");
  DefMacro!("\\acmVolume{}", "\\lx@add@pubnote[role=volume]{#1}");
  DefMacro!("\\acmYear{}", "\\lx@add@date[role=published]{#1}");
  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords{#1}");
  DefMacro!("\\terms{}", "\\lx@add@keywords{#1}");

  //======================================================================
  // Accessible figure descriptions
  // Register WAI-ARIA namespace for accessible descriptions
  RegisterDocumentNamespace!("aria", "http://www.w3.org/ns/wai-aria");

  NewCounter!("acmlabel", "");
  // Perl: \Description[short]{long} — displays #1 (short desc) in the note
  // properties: RefStepCounter('acmlabel') for xml:id
  // beforeConstruct: sets aria:labelledby on parent figure element
  DefConstructor!("\\Description[]{}", "^^<ltx:note xml:id='#id' class='ltx_nodisplay'>#1</ltx:note>",
    properties => { RefStepCounter!("acmlabel") },
    before_construct => sub[document, whatsit] {
      if let Some(id) = whatsit.get_property("id") {
        let id_str = id.to_string();
        if let Some(mut figure) = document.get_element() {
          document.set_attribute(&mut figure, "aria:labelledby", &id_str)?;
        }
      }
    }
  );

  //======================================================================
  // Use \author for EACH author, follow with \orcid, \affiliation, \email as needed.
  // Note that \affiliation can apply to all preceding authors without one
  // (Perl PR #2767)
  DefMacro!("\\author{}",                "\\lx@add@creator[role=author]{#1}");
  DefMacro!("\\editor{}",                "\\lx@add@creator[role=editor]{#1}");
  DefMacro!("\\affiliation{}",           "\\lx@add@contact[role=affiliation,annotate=new]{#1}");
  DefMacro!("\\additionalaffiliation{}", "\\lx@add@contact[role=altaffiliation]{#1}");
  DefMacro!("\\email [] Semiverbatim",   "\\lx@add@contact[role=email,name={email: }]{#2}");
  DefMacro!("\\orcid{}",                 "\\lx@add@contact[role=orcid, name={OrcID: }]{#1}");

  //======================================================================
  // Internal structure to affiliation (Perl PR #2767: comma-joined parts;
  // empty parts skipped)
  DefMacro!("\\lx@acm@addresspartsep", "");
  DefMacro!("\\lx@acm@addresspart{}{}",
    "\\ifx.#2.\\else\\lx@acm@addresspartsep\\def\\lx@acm@addresspartsep{,~}\\lx@acm@addresspart@{#1}{#2}\\fi");
  DefConstructor!("\\lx@acm@addresspart@{}{}",
    "<ltx:text class='ltx_affiliation_#1' _noautoclose='1'>#2</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefMacro!("\\position{}",      "\\lx@acm@addresspart{position}{#1}");
  DefMacro!("\\institution{}",   "\\lx@acm@addresspart{institution}{#1}");
  DefMacro!("\\department{}",    "\\lx@acm@addresspart{department}{#1}");
  DefMacro!("\\streetaddress{}", "\\lx@acm@addresspart{streetaddress}{#1}");
  DefMacro!("\\city{}",          "\\lx@acm@addresspart{city}{#1}");
  DefMacro!("\\state{}",         "\\lx@acm@addresspart{state}{#1}");
  DefMacro!("\\postcode{}",      "\\lx@acm@addresspart{postcode}{#1}");
  DefMacro!("\\country{}",       "\\lx@acm@addresspart{country}{#1}");

  DefMacro!("\\titlenote{}",    "\\lx@add@pubnote[role=note]{#1}");
  DefMacro!("\\subtitlenote{}", "\\lx@add@pubnote[role=note]{#1}");
  DefMacro!("\\authornote{}",   "\\lx@add@contact[role=note]{#1}");

  DefMacro!("\\abstract",    "\\lx@begin@abstract");
  DefMacro!("\\endabstract", "\\lx@end@abstract");

  // Rust-only content preserves (Perl gobbles these)
  DefMacro!("\\shortauthors{}", "\\lx@add@frontmatter{ltx:note}[role=shortauthors]{#1}");
  def_macro_noop("\\authornotemark[]")?;
  DefMacro!("\\authorsaddresses{}",
    "\\lx@add@frontmatter{ltx:note}[role=authorsaddresses]{#1}");
  def_macro_noop("\\startPage")?;
  def_macro_noop("\\settopmatter{}")?;
  def_macro_noop("\\copyrightpermissionfootnoterule")?;
  def_macro_noop("\\acmBadgeL")?;

  //======================================================================
  // Natbib cite aliases
  Let!("\\citeN", "\\cite");
  Let!("\\cite", "\\citep");
  Let!("\\citeANP", "\\citeauthor");
  Let!("\\citeNN", "\\citeyearpar");
  Let!("\\citeyearNP", "\\citeyear");
  Let!("\\citeyear", "\\citeyearpar");
  Let!("\\citeNP", "\\citealt");
  DefMacro!("\\shortcite{}", "\\citeyear{#1}");
  Let!("\\citeA", "\\citeauthor");

  DefRegister!("\\fulltextwidth" => Dimension::from_str("0pt")?);

  //======================================================================
  // Environments
  DefEnvironment!("{printonly}", "");
  DefEnvironment!("{screenonly}", "#body");
  DefEnvironment!("{anonsuppress}", "");

  //======================================================================
  // CCS descriptions
  DefMacro!("\\ccsdesc[]{}", "\\lx@add@pubnote[role=ccs,name={CCS:~}]{#2}");

  // Exclude CCSXML environment (Perl: defineExcluded(undef, 'CCSXML'))
  RawTeX!(r"\excludecomment{CCSXML}");

  //======================================================================
  // Acknowledgements
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  // Perl L167-168 ships properties => sub { (name => Digest(T_CS('\acknowledgmentsname'))) }
  // so a user `\renewcommand{\acknowledgmentsname}{Danksagung}` localizes the
  // attribute. Rust previously hard-coded "Acknowledgements", ignoring any
  // override. Use DigestIf! pattern (same as listings_sty:2060) to resolve
  // dynamically. Inline `<ltx:acknowledgements name='#name'>` template
  // matches the Perl form.
  DefConstructor!("\\acks", "<ltx:acknowledgements name='#name'>",
    properties => {
      let name_toks = DigestIf!(T_CS!("\\acknowledgmentsname"))?;
      stored_map!("name" => name_toks)
    });
  DefConstructor!("\\endacks", "</ltx:acknowledgements>");
  DefMacro!("\\grantsponsor Semiverbatim {} Semiverbatim", "Sponsor #2 \\url{#3}");
  DefMacro!("\\grantnum OptionalSemiverbatim Semiverbatim {}", "Grant \\##3");

  //======================================================================
  // Float environments
  DefEnvironment!("{teaserfigure}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' class='ltx_teaserfigure' ?#1(placement='#1')>#tags#body</ltx:figure>",
    before_digest => { before_float("figure", None); },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );

  DefEnvironment!("{marginfigure}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' class='ltx_marginfigure' ?#1(placement='#1')>#tags#body</ltx:figure>",
    before_digest => { before_float("figure", None); },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );

  DefEnvironment!("{margintable}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' class='ltx_margintable' ?#1(placement='#1')>#tags#body</ltx:table>",
    before_digest => { before_float("table", None); },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );

  //======================================================================
  // Sidebar — Perl L200-210
  DefMacro!("\\sidebarname", "Sidebar");
  DefMacro!("\\fnum@sidebar", "\\sidebarname\\nobreakspace\\thesidebar");
  DefMacro!("\\format@title@sidebar{}", "\\lx@tag{\\fnum@sidebar: }#1");

  // Perl L204-210: the {sidebar} env wraps body in <ltx:sidebar>. The
  // Perl signature is `{}{} Undigested [] {}` — title, bio, id, mark.
  // Rust previously had no sidebar env defined so ACM papers using
  // `\begin{sidebar}{title}...\end{sidebar}` hit undefined-env.
  // Simplified template (Perl has the title / creator fields
  // commented out too, so #body is the practical payload). The
  // optional labels/id attributes resolve via LaTeXML's normal
  // xml:id/labels machinery on DefEnvironment bodies.
  DefEnvironment!("{sidebar}{} Undigested [] {}",
    "<ltx:sidebar xml:id='#id'>#body</ltx:sidebar>");

  //======================================================================
  // Theorem styles via RawTeX
  RawTeX!(r"\def\@acmplainbodyfont{\itshape}");
  RawTeX!(r"\def\@acmplainindent{\parindent}");
  RawTeX!(r"\def\@acmplainheadfont{\scshape}");
  RawTeX!(r"\def\@acmplainnotefont{\@empty}");

  RawTeX!(r"\newtheoremstyle{acmplain}%
  {.5\baselineskip\@plus.2\baselineskip\@minus.2\baselineskip}%
  {.5\baselineskip\@plus.2\baselineskip\@minus.2\baselineskip}%
  {\@acmplainbodyfont}{\@acmplainindent}{\@acmplainheadfont}{.}{.5em}%
  {\thmname{#1}\thmnumber{ #2}\thmnote{ {\@acmplainnotefont(#3)}}}");

  RawTeX!(r"\def\@acmdefinitionbodyfont{\normalfont}");
  RawTeX!(r"\def\@acmdefinitionindent{\parindent}");
  RawTeX!(r"\def\@acmdefinitionheadfont{\itshape}");
  RawTeX!(r"\def\@acmdefinitionnotefont{\@empty}");

  RawTeX!(r"\newtheoremstyle{acmdefinition}%
  {.5\baselineskip\@plus.2\baselineskip\@minus.2\baselineskip}%
  {.5\baselineskip\@plus.2\baselineskip\@minus.2\baselineskip}%
  {\@acmdefinitionbodyfont}{\@acmdefinitionindent}{\@acmdefinitionheadfont}{.}{.5em}%
  {\thmname{#1}\thmnumber{ #2}\thmnote{ {\@acmdefinitionnotefont(#3)}}}");

  RawTeX!(r"\theoremstyle{acmplain}");
  RawTeX!(r"\newtheorem{theorem}{Theorem}[section]");
  RawTeX!(r"\newtheorem{conjecture}[theorem]{Conjecture}");
  RawTeX!(r"\newtheorem{proposition}[theorem]{Proposition}");
  RawTeX!(r"\newtheorem{lemma}[theorem]{Lemma}");
  RawTeX!(r"\newtheorem{corollary}[theorem]{Corollary}");
  RawTeX!(r"\theoremstyle{acmdefinition}");
  RawTeX!(r"\newtheorem{example}[theorem]{Example}");
  RawTeX!(r"\newtheorem{definition}[theorem]{Definition}");
  RawTeX!(r"\theoremstyle{acmplain}");

  Let!("\\proof", "\\@proof");
  Let!("\\endproof", "\\end@proof");

  // acmart.cls L1902: \setcctype[version]{by-spec} sets the Creative
  // Commons license. Preserve the license spec as ltx:note.
  // Witnesses 2406.04861, 2406.09266.
  DefMacro!("\\setcctype[]{}",
    "\\lx@add@frontmatter{ltx:note}[role=cc-license]{#2}");

  // acmart conditional toggles — declare as conditionals so user
  // paper's \@printpermissiontrue / \@printccstrue / \@printcopyrighttrue
  // etc. don't error. The list mirrors `\newif` declarations in
  // acmart.cls (TL2025 L181-L200); paper-local extension styles such
  // as `popets.sty` (acmart-derived) flip these without re-declaring,
  // so we must predeclare all of them. Driver: arXiv-2503.08256v1
  // (popets/acmart) where `\@acmownedfalse`, `\@acmownedtrue`, and
  // `\@ACM@journal@bibstripfalse` came up undefined.
  DefConditional!("\\if@printpermission");
  DefConditional!("\\if@printccs");
  DefConditional!("\\if@printcopyright");
  DefConditional!("\\if@printcopyrightbox");
  DefConditional!("\\if@printfolios");
  DefConditional!("\\if@acmReview");
  DefConditional!("\\if@ACM@manuscript");
  // \if@ACM@nonacm is NOT a newif in current acmart.cls, but some
  // papers (or older acmart versions) call `\@ACM@nonacmtrue` in the
  // preamble. Declare to avoid undefined errors. Witness 2211.10881.
  DefConditional!("\\if@ACM@nonacm");
  DefConditional!("\\if@ACM@journal");
  DefConditional!("\\if@ACM@journal@bibstrip");
  DefConditional!("\\if@ACM@journal@bibstrip@or@tog");
  DefConditional!("\\if@ACM@sigchiamode");
  DefConditional!("\\if@ACM@engage");
  DefConditional!("\\if@ACM@acmcp");
  DefConditional!("\\if@ACM@newfonts");
  DefConditional!("\\if@Description@present");
  DefConditional!("\\if@undescribed@images");
  DefConditional!("\\if@ACM@maketitle@typeset");
  DefConditional!("\\if@insideauthorgroup");
  DefConditional!("\\if@acmowned");
  DefConditional!("\\if@ACM@instpresent");
  DefConditional!("\\if@ACM@citypresent");
  DefConditional!("\\if@ACM@countrypresent");

  // acmart.cls L578: \def\@makefntext{\noindent\@makefnmark}.
  // Footnote helper used by acmart at L587/L600 in some path our
  // stub doesn't replicate; some templates probe it before our
  // explicit definition. Stub as a no-op so footnote processing
  // continues. Witness 2408.09084, 2408.03532 (sigconf papers).
  def_macro_noop("\\@makefntext")?;
});
