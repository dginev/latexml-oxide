//! dgruyter.sty (De Gruyter's journal style, shipped by the papers that use it; not in TeX Live) — a stand-in: the
//! paper's copy is read after it ("a stand-in's copy provides", OXIDIZED_DESIGN_DIVERGENCES #731), so the copy's
//! definitions of names the kernel's semantic layer or this binding owns hold only inside its own code — its copy of
//! array.sty's `\@array` (dgruyter.sty:2003-2029, calling array.sty's `\@arrayleft`), its `\let\@@array\@array` (:2031)
//! and `\@@array`, `\@tabular`, `\@classz`, `\endtabular` wrappers (:2032-2056) — and the document's alignments stay the
//! native ones, while the copy's extras stand as it makes them. Read raw with no binding (the ar5iv preload reads a
//! shipped style that has none), the copy's array code took over every `array`: 2510.17333's bordered block matrices,
//! 274 errors and two section labels lost; 2301.07931 101 errors and a Fatal. Guard
//! `perfect_kernel_batch80a::dgruyter_copy_keeps_the_native_array`.
//!
//! The copy's `\maketitle` is the kernel's too, so the frontmatter dgruyter stores for its title page (:2665-2760:
//! `\subtitle`, `\keywords`, `\classification`, the history dates, `\communicated`, `\dedication`, `\DOI`) is
//! declared here as the frontmatter's elements; the running heads and the journal's issue data are page furniture.
//! Guard `perfect_kernel_batch80a::dgruyter_frontmatter_is_kept`.
use latexml_package::prelude::*;

LoadDefinitions!({
  // dgruyter.sty:2392-2475 builds its author block on authblk (`\RequirePackage[noblocks]{authblk}`, :2394): under
  // `article`, `\author*[1]{Name}` is authblk's `\author[1]{Name}` (:2397) whose star (`\@ifstar`, :2472-2475) marks the
  // corresponding author — a `*` after the name, "*Corresponding author:" before the affiliation note (:2418-2421). The
  // copy's redefinitions are refused (the kernel's `\author` is locked), so authblk's binding read the star as the name:
  // a creator "*", "[1]Karoline Busse" a paragraph after the abstract (1910.07269). authblk is loaded here, before the
  // copy (whose own request then finds it loaded), and its `\author` takes the star: the corresponding-author contact
  // after the creator. Under another class the copy keeps the kernel's `\author` (:2476-2481). Guard
  // `perfect_kernel_batch86c::dgruyter_author_star_and_affiliation_labels`.
  RequirePackage!("authblk", options => vec![s!("noblocks")]);
  if has_meaning(&T_CS!("\\ver@article.cls")) {
    Let!("\\lx@dg@blk@author", "\\author");
    DefMacro!("\\author OptionalMatch:* []{}", sub[(star, label, name)] {
      let mut call = Invocation!(T_CS!("\\lx@dg@blk@author"), vec![label, Some(name)]).unlist();
      if star.is_some() {
        call.extend(
          TokenizeInternal!("\\lx@add@contact[role=corresponding]{\\@ifundefined{CorrAuth}{Corresponding author}{\\CorrAuth}}")
            .unlist(),
        );
      }
      Ok(Tokens::new(call))
    });
  }
  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords{#1}");
  // `\transabstract[<language>]{…}`, `\transkeywords[<language>]{…}` (:2733-2742): the abstract and keywords again, in
  // the second language
  DefMacro!("\\transabstract[]{}", "\\lx@add@abstract{#2}");
  DefMacro!("\\transkeywords[]{}", "\\lx@add@keywords{#2}");
  // `\classification[<scheme name>]{…}` (:2743-2745, "PACS" by default)
  DefMacro!(
    "\\classification[]{}",
    "\\lx@add@classification[scheme={#1}]{#2}"
  );
  // with the labels dgruyter prints them under (:2886-2958), in its language's names (:1771-1810), which its captions set
  DefMacro!(
    "\\received{}",
    "\\lx@add@date[role=received,name={\\@ifundefined{receivedname}{Received}{\\receivedname}~}]{#1}"
  );
  DefMacro!(
    "\\revised{}",
    "\\lx@add@date[role=revised,name={\\@ifundefined{revisedname}{revised}{\\revisedname}~}]{#1}"
  );
  DefMacro!(
    "\\accepted{}",
    "\\lx@add@date[role=accepted,name={\\@ifundefined{acceptedname}{accepted}{\\acceptedname}~}]{#1}"
  );
  DefMacro!(
    "\\communicated{}",
    "\\lx@add@pubnote[role=communicated,name={\\@ifundefined{communicatedname}{Communicated by}{\\communicatedname}:~}]{#1}"
  );
  DefMacro!("\\dedication{}", "\\lx@add@pubnote[role=dedication]{#1}");
  // dgruyter.sty:370 `\def\DOI#1{\gdef\@DOI{https://doi.org/#1}}`
  DefMacro!(
    "\\DOI{}",
    "\\@add@frontmatter{ltx:note}[role=doi,name={DOI:~}]{https://doi.org/#1}"
  );
  for proto in [
    "\\runningtitle{}",
    "\\runningauthor{}",
    "\\journalname{}",
    "\\journalyear{}",
    "\\journalvolume{}",
    "\\journalissue{}",
    "\\startpage{}",
    "\\articletype{}",
    "\\articlesubtype{}",
  ] {
    def_macro_noop(proto)?;
  }
});
