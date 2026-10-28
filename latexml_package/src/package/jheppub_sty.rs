use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: jheppub.sty.ltxml — 112 lines
  RequirePackage!("hyperref");
  RequirePackage!("color");
  // jheppub.sty:37 `\RequirePackage[numbers,sort&compress]{natbib}` (`no-natbib-sort`: `[numbers,compress]`, the same
  // citations here): numbered citations as 2112.14648's and 2607.20603's PDFs.
  RequirePackage!("natbib", options => vec![s!("numbers"), s!("sort&compress")]);
  // (the package's own load, so a document's repeat with other options is LaTeX's "Option clash" — binding/content.rs
  // `apply_new_options_on_reload`)
  AssignValue!("natbib.sty:requested_by_binding" => false, Some(Scope::Global));
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("epsfig");
  RequirePackage!("graphicx");

  // \author[]{} (Perl PR #2767)
  // One \author per author followed by \affiliation
  // OR both are supplied an optional label by which the affiliation is attached to author
  // optional arg is a label identifying which affiliation belongs
  // (a name list in one `\author` is several people, read as the per-author classes read theirs; Perl's
  // jheppub.sty.ltxml:35 makes one creator. 2411.15300)
  DefMacro!("\\author[]{}", "\\lx@add@authors@append[annotations={#1}]{#2}");
  jhep_contact_markup()?;

  // Keywords
  DefMacro!("\\keywordname", "\\textbf{Keywords}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords[name={\\keywordname}]{#1}");

  // Frontmatter metadata
  DefMacro!("\\arxivnumber{}", "\\lx@add@pubnote[role=arxiv]{#1}");
  DefMacro!("\\preprint{}", "\\lx@add@pubnote[role=preprint]{#1}");
  DefMacro!("\\proceeding{}", "\\lx@add@pubnote[role=conference]{#1}");
  DefMacro!("\\collaboration{}", "\\lx@add@pubnote[role=collaboration]{#1}");
  def_macro_noop("\\collaborationImg[]{}")?;

  // Acknowledgements — Perl L56-60 emits `name='#name'` on
  // <ltx:acknowledgements> with the name digested from
  // \acknowledgmentsname. Rust was omitting the attribute, so
  // downstream templates that need the title (e.g. HTML post-proc)
  // had no name to pick up.
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => sub[_args] {
      Ok(stored_map!("name" =>
        digest_in_group(T_CS!("\\acknowledgmentsname"))?))
    });
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");
  // ltx:acknowledgements Tag (autoClose + inlist=toc) is global — set in
  // latex_constructs.rs (arXiv-fork 23771504 removed the binding-local copies).

  // Conditionals — Perl L62-65
  DefConditional!("\\ifaffil");
  DefConditional!("\\ifnotoc");
  DefConditional!("\\ifemailadd");
  DefConditional!("\\iftoccontinuous");

  // Empty defaults — Perl L68-77
  DefMacro!("\\@subheader", "\\@empty");
  DefMacro!("\\@keywords", "\\@empty");
  DefMacro!("\\@abstract", "\\@empty");
  // jheppub.sty:66 `\renewcommand{\abstract}[1]{\gdef\@abstract{#1}}`: the abstract is ONE
  // argument, read pre-tokenized and stored for `\maketitle`. The kernel's locked `\abstract`
  // reads it so when the setter it refused took an argument (`\abstract:redefined@nargs`,
  // sect05.rs); this binding is that setter's record. Read as a group instead, a sectioning
  // command inside it (`\paragraph{}`) ended the abstract inside its own braces (an
  // `unexpected:}` Error: 1903.02433).
  AssignValue!("\\abstract:redefined@nargs" => Stored::Number(Number::new(1)), Some(Scope::Global));
  DefMacro!("\\@xtum", "\\@empty");
  DefMacro!("\\@dedicated", "\\@empty");
  DefMacro!("\\@arxivnumber", "\\@empty");
  DefMacro!("\\@collaboration", "\\@empty");
  DefMacro!("\\@collaborationImg", "\\@empty");
  DefMacro!("\\@proceeding", "\\@empty");
  DefMacro!("\\@preprint", "\\@empty");

  // Spacing macros — Perl L80-96
  DefMacro!("\\afterLogoSpace", "\\smallskip");
  DefMacro!("\\afterSubheaderSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterProceedingsSpace", "\\vskip21pt plus0.4fil minus15pt");
  DefMacro!("\\afterTitleSpace", "\\vskip23pt plus0.06fil minus13pt");
  DefMacro!("\\afterRuleSpace", "\\vskip23pt plus0.06fil minus13pt");
  DefMacro!("\\afterCollaborationSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterCollaborationImgSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterAuthorSpace", "\\vskip5pt plus4pt minus4pt");
  DefMacro!("\\afterAffiliationSpace", "\\vskip3pt plus3pt");
  DefMacro!("\\afterEmailSpace", "\\vskip16pt plus9pt minus10pt\\filbreak");
  DefMacro!("\\afterXtumSpace", "\\par\\bigskip");
  DefMacro!("\\afterAbstractSpace", "\\vskip16pt plus9pt minus13pt");
  DefMacro!("\\afterKeywordsSpace", "\\vskip16pt plus9pt minus13pt");
  DefMacro!("\\afterArxivSpace", "\\vskip3pt plus0.01fil minus10pt");
  DefMacro!("\\afterDedicatedSpace", "\\vskip0pt plus0.01fil");
  DefMacro!("\\afterTocSpace", "\\bigskip\\medskip");
  DefMacro!("\\afterTocRuleSpace", "\\bigskip\\bigskip");

  // Misc — Perl L99-109
  def_macro_noop("\\beforetochook")?;
  def_macro_noop("\\notoc")?;
  def_macro_noop("\\compress")?;
  // \correctionref{label}{url}{text} — link to a corrigendum.
  // Perl gobbles (raw \gdef binding hard to translate); we surpass
  // by emitting the text as a link via hyperref's \href.
  DefMacro!("\\correctionref{}{}{}", "\\href{#2}{#3}");
  DefMacro!("\\jname", "JHEP");
  // \subheader{text} — author-typed subheader prose; preserve as
  // ltx:note. Perl L? gobbles.
  DefMacro!("\\subheader{}",
    "\\lx@add@frontmatter{ltx:note}[role=subheader]{#1}");
  DefMacro!("\\xtumfont{}", "\\textsc{#1}");
  Let!("\\oldthebibliography", "\\thebibliography");
  Let!("\\endoldthebibliography", "\\endthebibliography");

  // jheppub.sty v.1.1227 (2112.14648's copy):313-314, :290-291 in older copies, "number equations after the sections":
  // `\renewcommand{\theequation}{\thesection.\arabic{equation}}` and `\numberwithin{equation}{section}`, "(2.1)" and
  // "(A.1)", as a paper's shipped copy numbers them (2112.14648, 2607.20603). Perl's jheppub.sty.ltxml keeps the
  // article's "(1)" (KNOWN_PERL_ERRORS #1097). Guard `perfect_kernel_batch78q::jhep_family_equations_within_sections`.
  RawTeX!(r"\renewcommand{\theequation}{\thesection.\arabic{equation}}\numberwithin{equation}{section}");
});

/// jheppub.sty's markup of an author's contacts, which the PoS template's pos.sty v2 shares (the same commands to the
/// same effect, pos.sty:359-430: `pos_sty`): the labelled affiliations, the footnote text without its mark given
/// inside an `\author`, the e-mail list printed under the affiliations, and the dedication.
#[rustfmt::skip]
pub fn jhep_contact_markup() -> Result<()> {
  DefMacro!("\\affiliation OptionalSemiverbatim {}",
    "\\lx@add@contact[role=affiliation,label={#1}]{#2}");
  // jheppub.sty:84-89 `\newcommand\note[2][]`: a footnote's text without its mark, for use inside `\author`; the
  // optional argument is the mark it prints the text under (the footnote counter then not stepped), else the next
  // footnote number. Perl's jheppub.sty.ltxml:39 `\note{}` takes one argument, so `\note[*]{Corresponding authors.}`
  // read `[` as the note and the rest into the name ("Jia-Jun Wu*]Corresponding authors" and a note "[", 2606.26596;
  // 1510.03434, 2404.00867; KNOWN_PERL_ERRORS #1033). The note is placed as an unmarked one is (inside an `\author`
  // that author's, given apart from the names the author's before it: 2404.00867's `\note[*]{Corresponding author:
  // steinh@princeton.edu}` after `\author[*\,b,c]{Paul J. Steinhardt,}`); its mark (`_mark`) is the legend the
  // corresponding-author step of an address list reads (`marked_corresponding_author`, OXIDIZED_DESIGN_DIVERGENCES
  // #809).
  DefMacro!("\\note[]{}", "\\lx@add@contact[role=note,_mark={#1}]{#2}");
  // jheppub.sty:127-131 `\emailAdd` lists the addresses, each `\href{mailto:#1}{\tt #1}` (2301.03278), which
  // `\maketitle` prints as one "E-mail:" line under the affiliations, attributing none. Perl's jheppub.sty.ltxml
  // gives the n-th address to the n-th author (`labelseq=author`), which named the wrong author where the list is not
  // in author order (2011.05971, 2212.09776, 2407.18115; jcappub 1806.05015, 2310.18996, 2407.07862; jinstpub
  // 2311.16063's bross@fnal.gov on Andrea Bersani, 2307.12991, 1910.06438, 2311.00870, 2605.30709). Rust-only
  // (OXIDIZED_DESIGN_DIVERGENCES #805, #801, #809): an address goes to the author it spells, else to the n-th author
  // while the list is one address per author in their order (no address spells another author than the one at its
  // place), else to the group's only author, else — the one address left on the author block — to the ONE author
  // marked as the corresponding author, by a note naming them so or a mark whose legend says so (1901.08773's and
  // 2510.11030's `\note{Corresponding author.}`; 1802.03677's note, whose number 1 two authors carry, leaves its
  // address to the block; a `*` with no legend counts only where the address itself says "(Corresponding author)",
  // 2308.06532),
  // else to the author block (`\lx@add@contact@placed[owner=sequence]`, `sequenced_contact_labels`). An
  // `\emailAdd{a@x, b@y}` list is an address each (2406.03543, 2505.23122; jinstpub 2211.05026, 2409.03065,
  // 2112.00370). The family's rule: jcappub and jinstpub take it with this binding.
  DefMacro!("\\emailAdd Semiverbatim", "\\lx@add@contact@list{\\lx@jhep@email}{#1}");
  DefMacro!("\\lx@jhep@email{}", "\\lx@add@contact@placed[role=email,owner=sequence]{}{\\lx@address@urlfont{#1}}");
  DefMacro!("\\dedicated{}", "\\lx@add@pubnote[role=dedication]{#1}");
  Ok(())
}
