//! acmart.cls — ACM article class
//! Perl: acmart.cls.ltxml (259 lines)
use crate::engine::latex_constructs::{after_float, before_float};
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: LoadClass('amsart', withoptions => 1)
  load_class_with_options("amsart", Tokens!())?;

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
  DefMacro!("\\copyrightyear{}", "\\@add@frontmatter{ltx:date}[role=copyright]{#1}");
  DefMacro!("\\setcopyright{}", "\\@add@frontmatter{ltx:note}[role=copyright]{#1}");
  DefMacro!("\\received[]{}", "\\@add@frontmatter{ltx:date}[role=received]{#2}");
  DefMacro!("\\acmJournal{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\acmSubmissionID{}", "\\@add@frontmatter{ltx:note}[role=submissionid]{#1}");
  DefMacro!("\\acmConference[]{}{}{}", "\\@add@frontmatter{ltx:note}[role=conference]{#2; #3; #4}");
  DefMacro!("\\acmBooktitle{}", "\\@add@frontmatter{ltx:note}[role=booktitle]{#1}");
  DefMacro!("\\acmArticle{}", "\\@add@frontmatter{ltx:note}[role=article]{#1}");
  DefMacro!("\\acmArticleSeq{}", "\\@add@frontmatter{ltx:note}[role=articleseq]{#1}");
  DefMacro!("\\acmDOI{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\acmISBN{}", "\\@add@frontmatter{ltx:note}[role=isbn]{#1}");
  DefMacro!("\\acmMonth{}", "\\@add@frontmatter{ltx:note}[role=publicationmonth]{#1}");
  DefMacro!("\\acmNumber{}", "\\@add@frontmatter{ltx:note}[role=journalnumber]{#1}");
  DefMacro!("\\acmPrice{}", "\\@add@frontmatter{ltx:note}[role=price]{#1}");
  DefMacro!("\\acmVolume{}", "\\@add@frontmatter{ltx:note}[role=journalvolume]{#1}");
  DefMacro!("\\acmYear{}", "\\@add@frontmatter{ltx:note}[role=journalyear]{#1}");
  DefMacro!("\\editor{}", "\\@add@frontmatter{ltx:creator}[role=editor]{\\@personname{#1}}");
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  DefMacro!("\\terms{}", "\\@add@frontmatter{ltx:keywords}{#1}");

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
  // Affiliation contact constructors
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");

  DefConstructor!("\\@@@addaffiliation{}", "^ <ltx:contact role='additional_affiliation'>#1</ltx:contact>");
  DefMacro!("\\additionalaffiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@addaffiliation{#1}}");

  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email [] Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#2}}");

  DefMacro!("\\orcid Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@orcid{\\@@orcid{#1}}}");
  // Perl acmart.cls.ltxml has `mode=>'restricted_horizontal',
  // enterHorizontal=>1` on \@@orcid and all eight affiliation
  // constructors. enter_horizontal triggers an implicit horizontal-mode
  // entry when invoked between paragraphs (vertical mode), so these
  // text-shaped <ltx:text> wrappers don't get emitted as block-level
  // children of the section root with no enclosing <ltx:p>. Same
  // class as cancel_sty cycle 86 / hyperref cycle 87.
  DefConstructor!("\\@@orcid{}",
    "<ltx:ref title='ORCID identifier' href='https://orcid.org/#1'>#1</ltx:ref>",
    mode => "restricted_horizontal", enter_horizontal => true
  );
  DefConstructor!("\\@@@orcid{}", "^ <ltx:contact role='orcid'>#1</ltx:contact>");

  //======================================================================
  // Internal structure to affiliation — Perl uses enterHorizontal=>1
  // on each so each `\institution{...}` etc. between paragraphs of an
  // \affiliation block opens a paragraph instead of a stray block.
  DefConstructor!("\\position{}", "<ltx:text class='ltx_affiliation_position' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefConstructor!("\\institution{}", "<ltx:text class='ltx_affiliation_institution' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefConstructor!("\\department{}", "<ltx:text class='ltx_affiliation_department' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefConstructor!("\\streetaddress{}", "<ltx:text class='ltx_affiliation_streetaddress' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefConstructor!("\\city{}", "<ltx:text class='ltx_affiliation_city' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefConstructor!("\\state{}", "<ltx:text class='ltx_affiliation_state' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefConstructor!("\\postcode{}", "<ltx:text class='ltx_affiliation_postcode' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefConstructor!("\\country{}", "<ltx:text class='ltx_affiliation_country' _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);

  //======================================================================
  // Ignorable stuff
  DefMacro!("\\shortauthors{}", None);
  DefMacro!("\\titlenote{}", None);
  DefMacro!("\\subtitlenote{}", None);
  DefMacro!("\\authornote{}", None);
  DefMacro!("\\authornotemark[]", None);
  DefMacro!("\\authorsaddresses{}", None);
  DefMacro!("\\startPage", None);
  DefMacro!("\\settopmatter{}", None);
  DefMacro!("\\copyrightpermissionfootnoterule", None);
  DefMacro!("\\acmBadgeL", None);

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
  DefMacro!("\\ccsdesc[]{}", "\\@add@frontmatter{ltx:note}[role=ccs]{#2}");

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
    "\\@add@frontmatter{ltx:note}[role=cc-license]{#2}");

  // acmart conditional toggles — declare as conditionals so user
  // paper's \\@printpermissiontrue / \\@printccstrue / \\@printcopyrighttrue
  // etc. don't error.
  DefConditional!("\\if@printpermission");
  DefConditional!("\\if@printccs");
  DefConditional!("\\if@printcopyright");
  DefConditional!("\\if@printcopyrightbox");
  DefConditional!("\\if@printfolios");
  DefConditional!("\\if@acmReview");

  // acmart.cls L578: \def\@makefntext{\noindent\@makefnmark}.
  // Footnote helper used by acmart at L587/L600 in some path our
  // stub doesn't replicate; some templates probe it before our
  // explicit definition. Stub as a no-op so footnote processing
  // continues. Witness 2408.09084, 2408.03532 (sigconf papers).
  DefMacro!("\\@makefntext", "");
});
