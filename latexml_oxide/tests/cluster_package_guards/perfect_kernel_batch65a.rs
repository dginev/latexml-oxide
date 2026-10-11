//! Red/green guards for batch 65a, the regressions sweep 149 found against sweep 148: a class's own `\title` and
//! `\author` called after the document body sealed the frontmatter (recipebook/LongSample), a heading name's font
//! leaking out of its heading (the same manual's TOC title; `\refname`), a caption in a rotated `\parbox` nesting a
//! figure in an inline block (heria/heria-proposal), and an ASCE membership grade read as an author (ascelike/
//! ascexmpl). Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv with `files` beside it, which must give no error and no warning.
fn convert_clean_files(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = latexml::util::test::convert_files_with(tex, files, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

fn convert_clean(tex: &str) -> String { convert_clean_files(tex, &[]) }

/// The document validates against the LaTeXML schema. jing is required: a guard that skipped without it would pass
/// without checking (CI installs it).
fn assert_schema_valid(xml: &str) {
  let lines = latexml::util::test::rng_error_count(xml)
    .expect("jing must be on PATH for this schema guard (CI installs it)");
  assert_eq!(lines, 0, "jing:\n{xml}");
}

/// Every `<creator>` element of `xml`, whole, with the indentation between its tags removed.
fn creators(xml: &str) -> Vec<String> {
  xml
    .split("<creator ")
    .skip(1)
    .map(|rest| {
      let element = &rest[..rest.find("</creator>").expect("closed creator") + "</creator>".len()];
      let lines: Vec<&str> = element.lines().map(str::trim).collect();
      format!("<creator {}", lines.concat())
    })
    .collect()
}

const RECIPE_CLASS: &str =
  include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/recipemini.cls");

/// A class `\title` that typesets in place (RecipeBook.cls:284-286) and an `\author` that only sets the page footer
/// (:300-302), called once per recipe. The first title and author, before the body sealed the frontmatter, are the
/// document's (the lock keeps the kernel's commands). A later `\title` the seal turns away runs the class's own
/// definition where it stands — the recipe's title in bold — and a later `\author`, which the class sets nowhere in
/// the flow, is the author of the section it stands in (user ruling 2026-10-11), labelled with the class's default
/// `Author: `. The `\contentsname`'s unbraced font switch stays in the TOC title. Repro
/// sectioning-frontmatter/class_title_after_the_body_typesets_in_place.
#[test]
fn sealed_title_runs_the_class_title_in_place() {
  let xml = convert_clean_files(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title_after_the_body_typesets_in_place.tex"
    ),
    &[("recipemini.cls", RECIPE_CLASS)],
  );
  assert_eq!(xml.matches("<title>French dip</title>").count(), 1, "{xml}");
  assert_eq!(
    creators(&xml),
    vec![
      r#"<creator role="author"><personname>Brad</personname></creator>"#,
      r#"<creator before="Author: " role="author"><personname>Brad, dough recipe from Chef Rider</personname></creator>"#,
    ],
    "{xml}"
  );
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S2""#],
    r#"<section inlist="toc" xml:id="S2">
      <tags>
        <tag>2</tag>
        <tag role="refnum">2</tag>
        <tag role="typerefnum">§2</tag>
      </tags>
      <title><tag close=" ">2</tag>Pork</title>
      <creator before="Author: " role="author">
        <personname>Brad, dough recipe from Chef Rider</personname>
      </creator>
      <para xml:id="S2.p1">
        <p xml:id="S2.p1.1"><text font="bold" fontsize="144%" xml:id="S2.p1.1.1">Italian calzones</text></p>
      </para>
      <para xml:id="S2.p2">
        <p xml:id="S2.p2.1">Knead the dough.</p>
      </para>
    </section>"#,
  );
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S3""#],
    r#"<section inlist="toc" xml:id="S3">
      <tags>
        <tag>3</tag>
        <tag role="refnum">3</tag>
        <tag role="typerefnum">§3</tag>
      </tags>
      <title><tag close=" ">3</tag>Soup</title>
      <para xml:id="S3.p1">
        <p xml:id="S3.p1.1"><text font="bold" fontsize="144%" xml:id="S3.p1.1.1">Squash soup</text></p>
      </para>
      <para xml:id="S3.p2">
        <p xml:id="S3.p2.1">Simmer the squash.</p>
      </para>
    </section>"#,
  );
  assert_schema_valid(&xml);
}

/// A class `\title` whose optional argument defaults to a length register (RecipeBook.cls:284
/// `O{\recipetitlefontsize}`) still only typesets: a length used as a value is no store, so a late title is set in
/// place.
#[test]
fn class_title_with_a_length_default_typesets_in_place() {
  let xml = convert_clean_files(
    r"\documentclass{sizeclass}
\begin{document}
\section{Beef}
\title{French dip}
Slice the roast.
\section{Pork}
\title{Italian calzones}
Knead the dough.
\end{document}",
    &[(
      "sizeclass.cls",
      r"\ProvidesClass{sizeclass}
\LoadClass{article}
\newlength\recipetitlefontsize \setlength\recipetitlefontsize{22pt}
\RenewDocumentCommand{\title}{O{\recipetitlefontsize} m}{\begingroup\fontsize{#1}{#1}\selectfont\bfseries #2\endgroup}",
    )],
  );
  assert!(
    xml.contains(r#"<text font="bold" fontsize="220%" xml:id="S2.p1.1.1">Italian calzones</text>"#),
    "{xml}"
  );
}

/// A late `\author` whose class's own body hands it straight to page furniture — a mark (`\markboth{#1}{#1}`,
/// `\markright{#1}`, `\markboth{#1 et al.}{}`), a fancyhdr head or foot (`\fancyfoot[L]{#1}`, `\lfoot{#1}`,
/// `\fancyhead[RO]{#1}`), or a definition of the page style's foot (`\def\@oddfoot{#1}`, `\global\def\@oddfoot{#1}`) —
/// is the author of the section it stands in (user ruling 2026-10-11, OXIDIZED_DESIGN #527).
#[test]
fn author_handed_to_page_furniture_is_the_units_author() {
  for class in [
    r"\renewcommand\author[1]{\markboth{#1}{#1}}",
    r"\renewcommand\author[1]{\markright{#1}}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\renewcommand\author[1]{\fancyfoot[L]{#1}}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\renewcommand\author[1]{\lfoot{#1}}",
    r"\renewcommand\author[1]{\def\@oddfoot{#1}}",
    r"\renewcommand\author[1]{\global\def\@oddfoot{#1}}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\renewcommand\author[1]{\fancyhead[RO]{#1}}",
    // the author beside other text in the furniture's argument still counts
    r"\renewcommand\author[1]{\markboth{#1 et al.}{}}",
    // xparse: the specification numbers the author (`{o m}` → `#2`; an all-`m` command is optimized to its code macro)
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\RenewDocumentCommand\author{o m}{\lfoot{#2}}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\RenewDocumentCommand\author{m}{\lfoot{#1}}",
  ] {
    let cls = format!("\\ProvidesClass{{footer}}\n\\LoadClass{{article}}\n{class}\n");
    let xml = convert_clean_files(
      r"\documentclass{footer}
\title{Main Title}
\author{Main Author}
\begin{document}
\section{First}
First text.
\author{Late Person}
Later text.
\end{document}",
      &[("footer.cls", &cls)],
    );
    assert_eq!(
      creators(&xml),
      vec![
        r#"<creator role="author"><personname>Main Author</personname></creator>"#,
        r#"<creator role="author"><personname>Late Person</personname></creator>"#,
      ],
      "{class}\n{xml}"
    );
  }
}

/// The unit's author goes among its front matter, after its title, even when the section's text came first; an
/// explicit optional argument is its label.
#[test]
fn author_after_section_text_joins_the_units_front_matter() {
  let xml = convert_clean_files(
    r"\documentclass{recipemini}
\begin{document}
\section{Beef}
\author{Brad}
\title{French dip}
Slice the roast.
\section{Pork}
Knead the dough.
\author[By: ]{Chef Rider}
\end{document}",
    &[("recipemini.cls", RECIPE_CLASS)],
  );
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S2""#],
    r#"<section inlist="toc" xml:id="S2">
      <tags>
        <tag>2</tag>
        <tag role="refnum">2</tag>
        <tag role="typerefnum">§2</tag>
      </tags>
      <title><tag close=" ">2</tag>Pork</title>
      <creator before="By: " role="author">
        <personname>Chef Rider</personname>
      </creator>
      <para xml:id="S2.p1">
        <p xml:id="S2.p1.1">Knead the dough.</p>
      </para>
    </section>"#,
  );
  assert_schema_valid(&xml);
}

/// A class command that reaches the kernel's through a helper (`\let\origauthor\author`, as beamerbasetitle.sty:69-70
/// and authoraftertitle.sty:11-23 save it, called from the class's own `\@author@line`) reaches it once while the
/// class's own call runs: no loop (`Fatal:Timeout:PushbackLimit` before the re-entry guard). Reaching the kernel's
/// means the class keeps the value, so the sealed author is dropped, not made the section's author.
#[test]
fn class_command_calling_the_kernels_does_not_loop() {
  let xml = convert_clean_files(
    r"\documentclass{wrapauthor}
\begin{document}
\author{Ann Able}
\section{Intro}
Text.
\author{Bob Baker}
\section{More}
More text.
\end{document}",
    &[(
      "wrapauthor.cls",
      r"\ProvidesClass{wrapauthor}
\LoadClass{article}
\let\origauthor\author
\def\@author@line#1{\origauthor{#1}}
\renewcommand{\author}[1]{\@author@line{#1}}",
    )],
  );
  // the kernel's `\author`, reached through the helper, keeps it — a store the seal drops, no unit author
  assert_eq!(
    creators(&xml),
    vec![r#"<creator role="author"><personname>Ann Able</personname></creator>"#],
    "{xml}"
  );
}

/// A class `\author` that fills a token register in any form (`\authtoks{#1}`, `\authtoks\expandafter{…}`,
/// `\expandafter\authtoks\expandafter{…}`, `\authtoks\@xp{…}`, `\authtoks\othertoks`, `\global\authtoks{…}`: tex.web
/// §1226 reads the right-hand side with `get_x_token`) or a box (`\sbox\authbox{#1}`, `\setbox\authbox\vbox{\unvbox
/// \authbox…}`, which appends) keeps its value: after the body it is a supplement's, dropped as the base drops it, not
/// the author of the section it stands in (pdflatex prints nothing there). One that only reads a
/// register or prints the box it fills typesets the late author in place.
#[test]
fn author_kept_in_a_register_or_a_box_is_a_store() {
  for class in [
    r"\newtoks\authtoks \renewcommand\author[1]{\authtoks{#1}}",
    r"\newsavebox\authbox \renewcommand\author[1]{\sbox\authbox{#1}}",
    r"\newtoks\authtoks \renewcommand\author[1]{\authtoks\expandafter{\the\authtoks #1}}",
    r"\newtoks\authtoks \renewcommand\author[1]{\expandafter\authtoks\expandafter{\the\authtoks\and #1}}",
    r"\newtoks\authtoks \let\@xp\expandafter \renewcommand\author[1]{\authtoks\@xp{\the\authtoks #1}}",
    r"\newtoks\authtoks \newtoks\othertoks \renewcommand\author[1]{\othertoks{#1}\authtoks\othertoks}",
    r"\newsavebox\authbox \renewcommand\author[1]{\setbox\authbox\vbox{\unvbox\authbox\hbox{#1}}}",
    r"\newtoks\authtoks \renewcommand\author[1]{\global\authtoks{#1}}",
    // a late author the class only logs, or stores through its own helpers: nothing shows, nothing reaches page
    // furniture (pdflatex prints nothing), so no unit author either
    r"\renewcommand\author[1]{\ClassInfo{keeper}{late author ignored}}",
    r"\renewcommand\author[1]{\typeout{#1}}",
    r"\newcommand\setauthor[1]{\gdef\@author{#1}}\RenewDocumentCommand\author{m}{\setauthor{#1}}",
    r"\newcommand\@@setauthor[1]{\gdef\@author{#1}}\newcommand\@setauthor[1]{\@@setauthor{#1}}\renewcommand\author[1]{\@setauthor{#1}}",
    r"\ExplSyntaxOn\tl_new:N\g_my_author_tl\cs_new:Npn\my_store_author:n#1{\tl_gset:Nn\g_my_author_tl{#1}}\RenewDocumentCommand\author{m}{\my_store_author:n{#1}}\ExplSyntaxOff",
    // page furniture that gets something other than the author: a helper's own parameter filled with other text, or
    // only the optional label (the author, `#2`, discarded)
    r"\newcommand\@runhead[1]{\markboth{#1}{#1}}\renewcommand\author[1]{\@runhead{Draft}}",
    r"\newcommand\@runfoot[1]{\def\@oddfoot{#1}}\renewcommand\author[1]{\@runfoot{\thepage}}",
    r"\renewcommand\author[2][]{\markboth{#1}{}}",
    // readers and dispatchers that never hand the author on (`\@runhead{Draft}\@gobble`, `\@runhead{\@title}…`,
    // only the starred or the `[` branch handing it to `\markboth`)
    r"\newcommand\@runhead[1]{\markboth{#1}{}}\def\author{\@runhead{Draft}\@gobble}",
    r"\newcommand\@runhead[1]{\markboth{#1}{}}\def\author{\@runhead{\@title}\@authorx}\def\@authorx#1{}",
    r"\newcommand\@runhead[1]{\markboth{#1}{}}\def\author{\@runhead\@title\@gobble}",
    r"\def\author{\@ifstar\@sauthor\@author@}\def\@sauthor#1{\markboth{#1}{}}\def\@author@#1{}",
    r"\def\author{\@ifnextchar[\@authoropt\@authornoopt}\newcommand\@authoropt[2][]{\markboth{#2}{}}\def\@authornoopt#1{}",
    // an xparse optional that is not the author in the foot (`{m O{Anon}}`, `{m o}`), and furniture the body does not
    // run itself: inside a conditional (a `\newif` flag, `\iffalse`) or another macro's argument (`\@gobble{…}`)
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\RenewDocumentCommand\author{m O{Anon}}{\lfoot{#2}}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\RenewDocumentCommand\author{m o}{\lfoot{\IfValueT{#2}{#2}}}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\newif\if@authfoot \renewcommand\author[1]{\if@authfoot\lfoot{#1}\fi}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\renewcommand\author[1]{\@gobble{\lfoot{#1}}}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\renewcommand\author[1]{\iffalse\lfoot{#1}\fi}",
    // a conditional opened through a macro (its `\fi` closes nothing known), and a group that discards the local foot
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\def\ifauthfoot{\iffalse}\renewcommand\author[1]{\ifauthfoot\lfoot{#1}\fi}",
    r"\RequirePackage{fancyhdr}\pagestyle{fancy}\renewcommand\author[1]{\begingroup\lfoot{#1}\endgroup}",
    // fail-safe residuals (OXIDIZED_DESIGN #527): furniture reached only through a helper or a dispatcher is not the
    // class's own body handing the author on, so the late author is dropped although the head shows it
    r"\ExplSyntaxOn\cs_new:Npn\my_mark:n#1{\markboth{#1}{}}\RenewDocumentCommand\author{m}{\my_mark:n{#1}}\ExplSyntaxOff",
    r"\def\author{\@ifnextchar[\@authoropt\@authornoopt}\def\@authornoopt#1{\markboth{#1}{}}\def\@authoropt[#1]#2{\markboth{#2}{}}",
    r"\newcommand\@runhead[1]{\markboth{#1}{}}\renewcommand\author[2][]{\@runhead{#2}}",
  ] {
    let cls = format!("\\ProvidesClass{{keeper}}\n\\LoadClass{{article}}\n{class}\n");
    let xml = convert_clean_files(
      r"\documentclass{keeper}
\title{Main Title}
\author{Main Author}
\begin{document}
\section{First}
First text.
\author{Late Person}
Later text.
\end{document}",
      &[("keeper.cls", &cls)],
    );
    assert_eq!(
      creators(&xml),
      vec![r#"<creator role="author"><personname>Main Author</personname></creator>"#],
      "{class}\n{xml}"
    );
  }
  // …but a class `\author` that only reads a register (`\the\authprefix`) or fills a box it prints (`\sbox` then
  // `\usebox`) typesets the author where it stands, as pdflatex does
  for class in [
    r"\newtoks\authprefix \authprefix={Recipe by } \renewcommand\author[1]{\par\noindent\textit{\the\authprefix#1}\par}",
    r"\renewcommand\author[1]{\sbox\@tempboxa{\itshape #1}\par\noindent\usebox\@tempboxa\par}",
  ] {
    let cls = format!("\\ProvidesClass{{printer}}\n\\LoadClass{{article}}\n{class}\n");
    let xml = convert_clean_files(
      r"\documentclass{printer}
\title{Main Title}
\begin{document}
\author{Main Author}
\section{First}
First text.
\author{Late Person}
Later text.
\end{document}",
      &[("printer.cls", &cls)],
    );
    let printed = if class.contains("authprefix") {
      "Recipe by Late Person"
    } else {
      "Late Person"
    };
    assert_element(
      &xml,
      "para",
      &[r#"xml:id="S1.p2""#],
      &format!(
        r#"<para class="ltx_noindent" xml:id="S1.p2">
          <p xml:id="S1.p2.1"><text font="italic" xml:id="S1.p2.1.1">{printed}</text></p>
        </para>"#
      ),
    );
  }
}

/// The kernel's `\title` without a class definition of its own keeps a title the body sealed off out of the
/// frontmatter (user ruling 2026-10-09: the paper's frontmatter is kept, a supplement's is not; cmpj/template,
/// schooldocs-examples): the second `\maketitle` sets its title in the text, where the first section's paragraph
/// stands.
#[test]
fn sealed_title_without_a_class_title_is_dropped() {
  let xml = convert_clean(
    r"\documentclass{article}
\title{The Paper}
\author{Ann Able}
\begin{document}
\maketitle
\section{Intro}
Text.
\title{The Supplement}
\maketitle
\section{More}
More text.
\end{document}",
  );
  assert_eq!(xml.matches("<title>The Paper</title>").count(), 1, "{xml}");
  assert_eq!(xml.matches("The Supplement").count(), 1, "{xml}");
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r#"<section inlist="toc" xml:id="S1">
      <tags>
        <tag>1</tag>
        <tag role="refnum">1</tag>
        <tag role="typerefnum">§1</tag>
      </tags>
      <title><tag close=" ">1</tag>Intro</title>
      <para xml:id="S1.p1">
        <p xml:id="S1.p1.1">Text.
The Supplement</p>
      </para>
    </section>"#,
  );
}

/// A class `\title` that keeps what it is given is the class's frontmatter setter, whose call after the body is a
/// supplement's, dropped, however it stores: `\gdef` (then prints), `\protected@xdef`, an xparse command's code,
/// imsart.sty:1958-1968's `\xdef` inside a group, or by calling a saved kernel `\title` (beamerbasetitle.sty:69-70
/// shape; 2301.10468, 2401.11672).
#[test]
fn storing_class_title_after_the_body_is_dropped() {
  let doc = r"\documentclass{setterclass}
\begin{document}
\title{The Paper}
\section{Intro}
Text.
\title{The Supplement}
\section{More}
More text.
\end{document}";
  for class in [
    r"\renewcommand{\title}[1]{\gdef\@title{#1}\par\noindent\textbf{#1}\par}",
    r"\renewcommand{\title}[1]{\protected@xdef\@title{#1}\par\noindent\textbf{#1}\par}",
    r"\RenewDocumentCommand{\title}{m}{\gdef\@title{#1}\par\noindent\textbf{#1}\par}",
    r"\def\title#1{\vglue 1pt\bgroup\xdef\@argi{#1}\xdef\@title{#1}\egroup\par\noindent\textbf{#1}\par}",
    r"\let\origtitle\title \renewcommand{\title}[1]{\origtitle{#1}\par\noindent\textbf{#1}\par}",
  ] {
    let cls = format!("\\ProvidesClass{{setterclass}}\n\\LoadClass{{article}}\n{class}\n");
    let xml = convert_clean_files(doc, &[("setterclass.cls", &cls)]);
    assert_eq!(
      xml.matches("<title>The Paper</title>").count(),
      1,
      "{class}\n{xml}"
    );
    assert!(!xml.contains("The Supplement"), "{class}\n{xml}");
  }
}

/// A heading name is digested in a group of its own: a `\contentsname` that switches the font unbraced
/// (RecipeBook.cls:207-209) sets the TOC title in that font and nothing after it, one that says `\centering`
/// (tidyres-doc.tex:9) centres the title, and a `\refname` with a font switch sets the bibliography's title and not
/// its entries (Perl `DigestIf`, latex_constructs.pool.ltxml:3885, leaks it).
#[test]
fn contents_name_font_stays_in_the_toc_title() {
  let xml = convert_clean(
    r"\documentclass{article}
\renewcommand{\contentsname}{\bfseries\Large Recipes}
\begin{document}
\tableofcontents
\section{Beef}
Slice the roast.
\end{document}",
  );
  assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r#"<section inlist="toc" xml:id="S1">
      <tags>
        <tag>1</tag>
        <tag role="refnum">1</tag>
        <tag role="typerefnum">§1</tag>
      </tags>
      <title><tag close=" ">1</tag>Beef</title>
      <para xml:id="S1.p1">
        <p xml:id="S1.p1.1">Slice the roast.</p>
      </para>
    </section>"#,
  );
  assert!(
    xml.contains(r#"<title font="bold" fontsize="144%">Recipes</title>"#),
    "{xml}"
  );
  let xml = convert_clean(
    r"\documentclass{article}
\renewcommand{\contentsname}{\centering Table of Contents}
\begin{document}
\tableofcontents
\section{Beef}
Slice the roast.
\end{document}",
  );
  assert!(
    xml.contains(r#"<title class="ltx_align_center">Table of Contents</title>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="S1.p1.1">Slice the roast.</p>"#),
    "{xml}"
  );
  let xml = convert_clean(
    r"\documentclass{article}
\renewcommand{\refname}{\bfseries\Large References}
\begin{document}
\section{Intro}
Text \cite{a}.
\begin{thebibliography}{9}
\bibitem{a} A. Author, Some title.
\end{thebibliography}
\end{document}",
  );
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib">
      <title font="bold" fontsize="144%">References</title>
      <biblist>
        <bibitem key="a" xml:id="bib.bib1">
          <tags>
            <tag>[1]</tag>
            <tag role="refnum">1</tag>
          </tags>
          <bibblock> A. Author, Some title.
</bibblock>
        </bibitem>
      </biblist>
    </bibliography>"#,
  );
}

/// A `\caption` closing a `\parbox` in a `\rotatebox` in a figure (heria.cls:737-745 `SidewaysFigure`): the rotated
/// inline block holds no figure, so the box is a block for its leading text and its caption goes to the figure it is
/// in, after it, which keeps the number and the label; the document is schema-valid. Repro
/// captions-floats/caption_in_a_rotated_parbox_stays_with_its_figure.
#[test]
fn caption_in_a_rotated_parbox_stays_with_its_figure() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption_in_a_rotated_parbox_stays_with_its_figure.tex"
  ));
  assert_eq!(xml.matches("<figure").count(), 1, "{xml}");
  assert_element(
    &xml,
    "figure",
    &[r#"placement="p""#],
    r#"<figure inlist="lof" labels="LABEL:fig:impact" placement="p" xml:id="S0.F1">
    <tags>
      <tag>Figure 1</tag>
      <tag role="refnum">1</tag>
      <tag role="typerefnum">Figure 1</tag>
    </tags>
    <inline-block angle="90" depth="0.0pt" height="550.0pt" innerdepth="272.5pt" innerheight="277.5pt" innerwidth="550.0pt" width="550.0pt" xtranslate="0.0pt" ytranslate="0.0pt" xml:id="S0.F1.1">
      <p class="ltx_parbox" vattach="middle" width="550.0pt" xml:id="S0.F1.1.1">The impact tableau.</p>
    </inline-block>
    <toccaption><tag close=" ">1</tag>Impact summary tableau</toccaption>
    <caption><tag close=": ">Figure 1</tag>Impact summary tableau</caption>
  </figure>"#,
  );
  assert_schema_valid(&xml);
}

/// A box in a transform that OPENS with its caption keeps all its content inside the transform, as Perl chooses its
/// container (TeX_Box.pool.ltxml:507-509: the first candidate holding all of it): a rotated minipage in a table and a
/// `\resizebox`d one. Hoisting the caption would read it after the table it heads and leave an empty, sized box.
#[test]
fn caption_first_box_in_a_transform_keeps_its_content() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{graphicx}
\begin{document}
Table~\ref{t1}.
\begin{table}
\centering
\rotatebox{90}{\begin{minipage}{8cm}\caption{Results}\label{t1}
\begin{tabular}{cc}a & b\\ c & d\end{tabular}\end{minipage}}
\end{table}
\begin{table}
\resizebox{\linewidth}{!}{\begin{minipage}{12cm}\caption{Scaled}\label{t2}
\begin{tabular}{cc}e & f\end{tabular}\end{minipage}}
\end{table}
\end{document}",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="tab1""#],
    r#"<table xml:id="tab1">
    <inline-block align="center" angle="90" depth="0.0pt" height="227.6pt" innerdepth="25.5pt" innerheight="30.5pt" innerwidth="227.6pt" width="55.9pt" xtranslate="-85.8pt" ytranslate="-85.8pt" xml:id="tab1.1">
      <figure class="ltx_minipage" inlist="lot" labels="LABEL:t1" vattach="middle" width="227.6pt" xml:id="S0.T1">
        <tags>
          <tag>Table 1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">Table 1</tag>
        </tags>
        <toccaption><tag close=" ">1</tag>Results</toccaption>
        <caption><tag close=": ">Table 1</tag>Results</caption>
        <tabular vattach="middle" xml:id="S0.T1.1">
          <tbody>
            <tr xml:id="S0.T1.1.1">
              <td align="center" xml:id="S0.T1.1.1.1">a</td>
              <td align="center" xml:id="S0.T1.1.1.2">b</td>
            </tr>
            <tr xml:id="S0.T1.1.2">
              <td align="center" xml:id="S0.T1.1.2.1">c</td>
              <td align="center" xml:id="S0.T1.1.2.2">d</td>
            </tr>
          </tbody>
        </tabular>
      </figure>
    </inline-block>
  </table>"#,
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="tab2""#],
    r#"<table xml:id="tab2">
    <inline-block depth="20.9pt" height="26.0pt" width="345.0pt" xscale="1.01044696817242" xtranslate="1.8pt" yscale="1.01044696817242" ytranslate="-0.2pt" xml:id="tab2.1">
      <figure class="ltx_minipage" inlist="lot" labels="LABEL:t2" vattach="middle" width="341.4pt" xml:id="S0.T2">
        <tags>
          <tag>Table 2</tag>
          <tag role="refnum">2</tag>
          <tag role="typerefnum">Table 2</tag>
        </tags>
        <toccaption><tag close=" ">2</tag>Scaled</toccaption>
        <caption><tag close=": ">Table 2</tag>Scaled</caption>
        <tabular vattach="middle" xml:id="S0.T2.1">
          <tbody>
            <tr xml:id="S0.T2.1.1">
              <td align="center" xml:id="S0.T2.1.1.1">e</td>
              <td align="center" xml:id="S0.T2.1.1.2">f</td>
            </tr>
          </tbody>
        </tabular>
      </figure>
    </inline-block>
  </table>"#,
  );
}

/// A captioned box beside other content in a transform (`\scalebox{…}{minipage \hfill minipage}`, 2605.06364) keeps
/// Perl's container: its caption is not hoisted, since the hoist moves the insertion point and the next panel would
/// leave the transform; both panels stay inside, in order.
#[test]
fn captioned_box_beside_other_content_in_a_transform_stays_inside() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{graphicx}
\usepackage{caption}
\begin{document}
\begin{figure}[h]
\scalebox{0.98}{\begin{minipage}{0.48\columnwidth}
A drawing.
\caption{Aux path}\label{fig:a}
\end{minipage}\hfill
\begin{minipage}{0.48\columnwidth}
\captionof{table}{Metrics}\label{tab:m}
\begin{tabular}{lc}x & 1\end{tabular}
\end{minipage}}
\end{figure}
\end{document}",
  );
  assert_element(
    &xml,
    "figure",
    &[r#"placement="h""#],
    r#"<figure placement="h" xml:id="fig1">
    <inline-block depth="8.5pt" height="13.4pt" width="327.8pt" xscale="1.0" xtranslate="-3.3pt" yscale="1.0" ytranslate="0.2pt" xml:id="fig1.1">
      <figure class="ltx_minipage" inlist="lof" labels="LABEL:fig:a" vattach="middle" width="165.6pt" xml:id="S0.F1">
        <tags>
          <tag>Figure 1</tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">Figure 1</tag>
        </tags>
        <p xml:id="S0.F1.1">A drawing.</p>
        <toccaption><tag close=" ">1</tag>Aux path</toccaption>
        <caption><tag close=": ">Figure 1</tag>Aux path</caption>
      </figure>
      <inline-logical-block class="ltx_minipage" vattach="middle" width="165.6pt" xml:id="fig1.1.1">
        <table inlist="lot" labels="LABEL:tab:m" xml:id="S0.T1">
          <tags>
            <tag>Table 1</tag>
            <tag role="refnum">1</tag>
            <tag role="typerefnum">Table 1</tag>
          </tags>
          <toccaption><tag close=" ">1</tag>Metrics</toccaption>
          <caption><tag close=": ">Table 1</tag>Metrics</caption>
          <tabular vattach="middle" xml:id="S0.T1.1">
            <tbody>
              <tr xml:id="S0.T1.1.1">
                <td align="left" xml:id="S0.T1.1.1.1">x</td>
                <td align="center" xml:id="S0.T1.1.1.2">1</td>
              </tr>
            </tbody>
          </tabular>
        </table>
      </inline-logical-block>
    </inline-block>
  </figure>"#,
  );
}

/// The element whose start tag begins at `start` in `xml`, to its matching end tag (same-name nesting counted).
fn element_at(xml: &str, start: usize) -> &str {
  let name_end = xml[start + 1..].find([' ', '>', '/']).expect("a tag name") + start + 1;
  let name = &xml[start + 1..name_end];
  let (open, close) = (format!("<{name}"), format!("</{name}>"));
  // a start tag of this name, not of a longer one (`<inline-block` is not `<inline-blocks`)
  let open_at = |from: usize| {
    let mut from = from;
    while let Some(o) = xml[from..].find(&open).map(|o| o + from) {
      if xml[o + open.len()..].starts_with([' ', '>', '/']) {
        return Some(o);
      }
      from = o + open.len();
    }
    None
  };
  let mut depth = 0usize;
  let mut at = start;
  loop {
    let next_open = open_at(at);
    let next_close = xml[at..]
      .find(&close)
      .map(|c| c + at)
      .expect("a closing tag");
    match next_open {
      Some(o) if o < next_close => {
        depth += 1;
        at = o + open.len();
      },
      _ => {
        depth -= 1;
        at = next_close + close.len();
        if depth == 0 {
          return &xml[start..at];
        }
      },
    }
  }
}

/// The `labels` start tag and the number of the float element holding exactly `label`.
fn float_of(xml: &str, label: &str) -> (String, String) {
  let attr = format!("labels=\"LABEL:{label}\"");
  let at = xml
    .find(&attr)
    .unwrap_or_else(|| panic!("no element labelled exactly {label}:\n{xml}"));
  let start = xml[..at].rfind('<').expect("start tag");
  let tag_open = xml[at..].find("<tag>").expect("a tag") + at + "<tag>".len();
  let tag_close = xml[tag_open..].find("</tag>").expect("closed tag") + tag_open;
  (
    xml[start..at + attr.len()].to_string(),
    xml[tag_open..tag_close].to_string(),
  )
}

/// A closing caption is hoisted out of a transform only into a float that holds no other caption: two `\resizebox`ed
/// captioned panels side by side, or a rotated captioned `\parbox` beside the float's own `\caption`, stay separate
/// floats with their own labels and numbers (one hoist made "See Figures 1 and 1"), and `\subcaption` panels in
/// `\scalebox`es keep their panels. Perl's container stands for all of them.
#[test]
fn caption_hoist_never_merges_two_captions() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{graphicx}
\begin{document}
See Figures~\ref{fe} and~\ref{fz}.
\begin{figure}
\resizebox{0.45\linewidth}{!}{\begin{minipage}{6cm}Epsilon text.\caption{Cap E}\label{fe}\end{minipage}}\hfill
\resizebox{0.45\linewidth}{!}{\begin{minipage}{6cm}Zeta text.\caption{Cap Z}\label{fz}\end{minipage}}
\end{figure}
\end{document}",
  );
  assert_eq!(float_of(&xml, "fe").1, "Figure 1", "{xml}");
  assert_eq!(float_of(&xml, "fz").1, "Figure 2", "{xml}");
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{graphicx}
\usepackage{subcaption}
\begin{document}
See \ref{inner}, \ref{outer}.
\begin{figure}
\centering
\rotatebox{90}{\parbox{3cm}{Inner text.\caption{Inner cap}\label{inner}}}
\caption{Outer cap}\label{outer}
\end{figure}
\begin{figure}
\centering
\scalebox{0.9}{\begin{minipage}{.4\linewidth}Panel A.\subcaption{Sub A}\end{minipage}}
\scalebox{0.9}{\begin{minipage}{.4\linewidth}Panel B.\subcaption{Sub B}\end{minipage}}
\caption{Whole}
\end{figure}
\end{document}",
  );
  let (_, inner_number) = float_of(&xml, "inner");
  let (_, outer_number) = float_of(&xml, "outer");
  assert!(
    inner_number.ends_with("Figure 1</text>") && outer_number.ends_with("Figure 2</text>"),
    "{xml}"
  );
  assert!(
    xml.contains(r#"xml:id="S0.F3.fig1""#) && xml.contains(r#"xml:id="S0.F3.fig2""#),
    "{xml}"
  );
}

/// The hoist moves the insertion point, so it is refused when anything beside the box would follow it out: a second
/// panel behind an extra brace group in the transform (`\scalebox{0.8}{{minipage \hfill minipage}}`), or text after a
/// rotated box inside a frame (`\fbox{\rotatebox{90}{\parbox{…\caption…}} inside fbox after}`). Both keep Perl's
/// container and every word in its box.
#[test]
fn caption_hoist_never_moves_content_beside_the_box() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{graphicx}
\begin{document}
\begin{figure}
\scalebox{0.8}{{\begin{minipage}{.4\linewidth}Alpha panel.\caption{Cap A}\end{minipage}\hfill\begin{minipage}{.4\linewidth}Beta panel.\caption{Cap B}\end{minipage}}}
\end{figure}
\begin{figure}
\fbox{\rotatebox{90}{\parbox{3cm}{Gamma text.\caption{Cap G}}} inside fbox after}
\end{figure}
\end{document}",
  );
  // the scaled box (its element, to its own closing tag) holds both panels
  let scaled_start = xml[..xml.find("xscale=").expect("the scaled box")]
    .rfind("<inline-block")
    .expect("its tag");
  let scaled = element_at(&xml, scaled_start);
  assert!(
    scaled.contains("Alpha panel.") && scaled.contains("Beta panel."),
    "{xml}"
  );
  // the frame keeps its text after the rotated box, and the rotated box its caption
  assert!(
    xml.contains("</inline-block> inside fbox after</text>"),
    "{xml}"
  );
  let rotated_start = xml[..xml.find(r#"angle="90""#).expect("the rotated box")]
    .rfind("<inline-block")
    .expect("its tag");
  let rotated = element_at(&xml, rotated_start);
  assert!(
    rotated.contains("<caption") && rotated.contains("Cap G"),
    "{xml}"
  );
}

/// A transform alone in an alignment environment (`{center}`) in a figure: its closing caption goes to the figure
/// through the environment's capture block, after the rotated box, schema-valid. An environment that holds more than
/// the transform (a tabular after it), or one that is not an alignment wrapper (`{minipage}`: its panel would lose its
/// width), keeps Perl's container: the caption stays in its panel inside the transform, the tabular after it.
#[test]
fn caption_hoist_through_an_environment_holding_only_the_transform() {
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{graphicx}
\begin{document}
See \ref{a}.
\begin{figure}
\begin{center}\rotatebox{90}{\parbox{3cm}{Alpha text.\caption{Cap A}\label{a}}}\end{center}
\end{figure}
After text.
\end{document}",
  );
  assert_element(
    &xml,
    "figure",
    &[r#"labels="LABEL:a""#],
    r#"<figure inlist="lof" labels="LABEL:a" xml:id="S0.F1">
    <tags>
      <tag>Figure 1</tag>
      <tag role="refnum">1</tag>
      <tag role="typerefnum">Figure 1</tag>
    </tags>
    <inline-block align="center" angle="90" depth="0.0pt" height="85.4pt" innerdepth="7.9pt" innerheight="12.9pt" innerwidth="85.4pt" width="20.9pt" xtranslate="-32.2pt" ytranslate="-32.2pt" xml:id="S0.F1.1">
      <p class="ltx_parbox" vattach="middle" width="85.4pt" xml:id="S0.F1.1.1">Alpha text.</p>
    </inline-block>
    <toccaption class="ltx_centering"><tag close=" ">1</tag>Cap A</toccaption>
    <caption class="ltx_centering"><tag close=": ">Figure 1</tag>Cap A</caption>
  </figure>"#,
  );
  assert_schema_valid(&xml);
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{graphicx}
\begin{document}
\begin{figure}
\begin{center}
\rotatebox{90}{\parbox{3cm}{Alpha text.\caption{Cap A}\label{a}}}

\begin{tabular}{c}Theta cell\end{tabular}
\end{center}
\end{figure}
\end{document}",
  );
  let outer = element_at(
    &xml,
    xml
      .find(r#"<figure xml:id="fig1">"#)
      .expect("the outer figure"),
  );
  let rotated = element_at(outer, outer.find("<inline-block").expect("the rotated box"));
  assert!(
    rotated.contains(r#"labels="LABEL:a""#) && rotated.contains("Cap A</caption>"),
    "{xml}"
  );
  assert!(
    outer.find("</inline-block>") < outer.find("Theta cell"),
    "{xml}"
  );
  let xml = convert_clean(
    r"\documentclass{article}
\usepackage{graphicx}
\begin{document}
\begin{figure}
\begin{minipage}{.45\linewidth}\rotatebox{90}{\parbox{3cm}{Alpha text.\caption{Cap A}\label{a}}}\end{minipage}
\end{figure}
\end{document}",
  );
  let rotated = element_at(
    &xml,
    xml
      .find(r#"<inline-block angle="90""#)
      .expect("the rotated box"),
  );
  assert!(
    rotated.contains(r#"labels="LABEL:a""#) && rotated.contains("Cap A</caption>"),
    "{xml}"
  );
}

/// An ASCE membership grade before the society after the name, glued or spaced (ascexmpl.tex:48 `Matthew R. Kuhn,
/// M.ASCE`), qualifies the name, as `Member, IEEE` and `PhD` do: no creator of its own, and no merged-author error.
/// Repro sectioning-frontmatter/asce_membership_grade_qualifies_the_name.
#[test]
fn asce_membership_grade_qualifies_the_name() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/asce_membership_grade_qualifies_the_name.tex"
  ));
  let names: Vec<String> = creators(&xml)
    .iter()
    .map(|c| {
      let start = c.find("<personname>").expect("a personname") + "<personname>".len();
      c[start..start + c[start..].find("</personname>").expect("closed")].to_string()
    })
    .collect();
  assert_eq!(
    names,
    vec!["Matthew R. Kuhn, M.ASCE", "Second Author"],
    "{xml}"
  );
  let xml = convert_clean(
    r"\documentclass{article}
\author{Jane Roe, F.ASCE \and John Doe, A.M.ASCE \and Ann Able, Ph.D., P.E. \and Ann Ross, M. ASCE}
\begin{document}
\maketitle
Text.
\end{document}",
  );
  assert_eq!(
    creators(&xml),
    vec![
      r#"<creator role="author"><personname>Jane Roe, F.ASCE</personname></creator>"#,
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>John Doe, A.M.ASCE</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ann Able, Ph.D., P.E.</personname></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ann Ross, M. ASCE</personname></creator>",
    ],
    "{xml}"
  );
}
