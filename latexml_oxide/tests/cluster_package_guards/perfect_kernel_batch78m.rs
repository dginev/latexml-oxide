//! Red/green guards for batch 78m (fidelity audit round 3, items FM1-FM4 and N9; r2 the review's fixes): a shipped
//! spie.cls that loads authblk reads authblk's authors; the horizontal space a name sets before its own mark, a math
//! group of spacing between two names, the tie after an entry's mark and `\enskip` in a name or after a note; jpconf's
//! and bmcart's title-page blocks (bmcart's article notes too) as frontmatter; pst-plot's own loads. Each guard converts
//! its repro under `tools/perfect_kernel/repros/` and asserts no Error or Warning and the authored content in whole
//! elements, against pdflatex (latex for PSTricks). Witnesses 2406.08304 (spie); 0905.3658, 1312.1414, 1602.04504,
//! 2606.01993, 2402.19080 (spacing); 0705.0202, 1011.0523 (jpconf); 1711.01177, 1507.06515, 2007.00437, 2112.10068
//! (bmcart); 0711.2399 (pst-plot).
use latexml::util::test::{
  assert_element, convert_files_with, convert_with, normalize_markup, xml_element,
};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Every `<creator>` of `xml`, in document order, is EXACTLY the one `expected` lists at its place (each whole, as
/// [`assert_element`] compares one), and there are no others.
#[track_caller]
fn assert_creators(xml: &str, expected: &[&str]) {
  let found: Vec<String> = xml
    .match_indices("<creator ")
    .filter_map(|(at, _)| xml_element(&xml[at..], "creator", &[]))
    .collect();
  let want: Vec<String> = expected.iter().map(|e| normalize_markup(e)).collect();
  assert_eq!(found, want, "{xml}");
}

/// SPIE's 2015 spie.cls v3.4 loads authblk (its :31); the binding loads it too, so `\author[\udem]{…}` is authblk's
/// marked author, not the kernel's one-argument `\author` that took "[" as the name (2406.08304: one creator "[", 30
/// empty affiliations, the names in the body). pdflatex: "Etienne Artigau^a and Francois Bouchy^b", then the two
/// marked affiliations.
#[test]
fn spie_copy_loading_authblk_reads_authblk_authors() {
  let (log, xml) = convert_files_with(
    repro!(
      "sectioning-frontmatter/spie_authblk_copy/spie_copy_loading_authblk_reads_authblk_authors.tex"
    ),
    &[(
      "spie.cls",
      repro!("sectioning-frontmatter/spie_authblk_copy/spie.cls"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<creator ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>Etienne Artigau</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Universite de Montreal</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\"", "role=\"author\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Francois Bouchy</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Observatoire de Geneve</contact></creator>",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">Body.</p></para>",
  );
}

/// `Oleg Lychkovskiy\thanks{…}\hspace*{2mm}$^{\rm a,b}$, Sergei Blinnikov\hspace*{2mm}$^{\rm a,c}$` (0905.3658): the
/// space before each mark is its name's, and `\rm` no part of the labels, so each creator has the institutes its marks
/// name. pdflatex: "Oleg Lychkovskiy* a,b, Sergei Blinnikov a,c" over the three marked institutes.
#[test]
fn author_space_before_own_mark_is_the_name() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/author_space_before_own_mark_is_the_name.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<creator ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>Oleg Lychkovskiy</personname><note class=\"ltx_note_frontmatter \
     ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">e-mail: o@x.ru</note><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> <text font=\"italic\" fontsize=\"90%\" xml:id=\"id2\">Institute A</text></contact><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> <text font=\"italic\" fontsize=\"90%\" xml:id=\"id3\">Institute \
     B</text></contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\"", "role=\"author\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Sergei Blinnikov</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> <text font=\"italic\" fontsize=\"90%\" xml:id=\"id4\">Institute \
     A</text></contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> <text font=\"italic\" \
     fontsize=\"90%\" xml:id=\"id5\">Institute C</text></contact></creator>",
  );
}

/// `Ann Able$^{1,2}$ $\qquad$ Bob Baker$^{1}$ \enskip Cat Cole$^{2}$` (1602.04504's `$\qquad$`, 1312.1414's
/// `\enskip`): three creators, each with the universities its marks name, where all three names were one creator.
/// pdflatex: "Ann Able^{1,2}   Bob Baker^1 Cat Cole^2", then "1 Univ A  2 Univ B".
#[test]
fn author_spacing_math_between_names_separates() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/author_spacing_math_between_names_separates.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Univ A</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ \
     B</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ A</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ B</contact></creator>",
  ]);
}

/// `$^1$~Univ A\quad$^2$~Univ B\quad\textsuperscript{3}\ Univ C`: the tie or control space right after an entry's mark
/// is that entry's, so the `\quad` before the mark parts two entries, not the space a name sets before its own mark
/// (78m r1: Ann with "Univ A² Univ B³ Univ C", Bob and Cat with none; the corpus shape 2606.01993, 2402.19080, kept).
/// pdflatex: "Ann Able^1, Bob Baker^2, Cat Cole^3", then "1 Univ A  2 Univ B  3 Univ C".
#[test]
fn author_tie_after_entry_mark_is_the_entry() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/author_tie_after_entry_mark_is_the_entry.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">\u{a0}Univ A</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">\u{a0}Univ B</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cat Cole</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ C</contact></creator>",
  ]);
}

/// `J.\enskip Smith\hspace*{1mm}$^{1}$\thanks{…} \enskip K.\enskip Jones$^{2}$`: an `\enskip` parts two names only right
/// after a mark or note (1312.1414's `Berry\thanks{…} \enskip \normalsize Andrew M.\ Childs`), the `\hspace*` before the
/// marks then the name's own; one inside a name stays there (78m r1: "J.", "Smith K.", "Jones"). pdflatex: "J.
/// Smith^{1*} K. Jones^2", then "1 Univ A", "2 Univ B".
#[test]
fn author_enskip_in_name_and_after_note() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/author_enskip_in_name_and_after_note.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>J.\u{2002}Smith</personname><note class=\"ltx_note_frontmatter \
     ltx_thanks_correspondence\" role=\"thanks\" xml:id=\"id1\">Corresponding author.</note><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>K.\u{2002}Jones</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
  ]);
}

/// IOP's jpconf.cls prints `\address` and `\ead` as `{indented}` list items where they stand (:104-110); read raw, they
/// were bullet lists after the abstract (0705.0202, 1011.0523). The binding makes them the author's affiliation and
/// e-mail, as iopart_support does. pdflatex: "Andreas Haungs", the institute, "E-mail: haungs@ik.fzk.de".
#[test]
fn jpconf_address_and_email_are_frontmatter() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/jpconf_copy/jpconf_address_and_email_are_frontmatter.tex"),
    &[(
      "jpconf.cls",
      repro!("sectioning-frontmatter/jpconf_copy/jpconf.cls"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>Andreas Haungs</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Institut für Kernphysik, Karlsruhe, Germany</contact><contact name=\"Email:\u{a0}\" \
     role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">haungs@ik.fzk.de</text></contact></creator>",
  );
  assert!(
    !xml.contains("<itemize"),
    "no address list in the body: {xml}"
  );
}

/// bmcart.cls's `{frontmatter}` lets `\title`, `\author`, `\address` to its printers (:966-984); read raw, the
/// document had no title, no creator and no address (1711.01177). The binding makes them the frontmatter, each author
/// requesting the addresses its `addressref` names and the article notes its `noteref` names (`{artnotes}`'s
/// `\note[id=n1]{Corresponding author}`, 1507.06515, 2007.00437, 2112.10068, dropped at 78m r1). pdflatex: "The Title",
/// "Jeremie Kim, Ann Able", "n1 Corresponding author", "1 Carnegie Mellon University, Pittsburgh, USA".
#[test]
fn bmcart_frontmatter_is_frontmatter() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/bmcart_copy/bmcart_frontmatter_is_frontmatter.tex"),
    &[(
      "bmcart.cls",
      repro!("sectioning-frontmatter/bmcart_copy/bmcart.cls"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<title>The Title</title>"), "{xml}");
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Jeremie Kim</personname><contact name=\"Email:\u{a0}\" \
     role=\"email\">j@x.org</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Carnegie Mellon \
     University, Pittsburgh, USA</contact><contact name=\"Note:\u{a0}\" role=\"note\">Corresponding \
     author</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Ann Able</personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Carnegie Mellon University, Pittsburgh, USA</contact><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">ETH Zurich, Zurich, Switzerland</contact></creator>",
  ]);
  assert_eq!(
    xml.matches("Corresponding author").count(),
    1,
    "the note once, on its author: {xml}"
  );
  assert!(!xml.contains("<titlepage"), "no typeset title page: {xml}");
}

/// pst-plot.sty:29-31 loads pstricks, pst-xkey and multido, so a paper's `\multido` over its plot points needs no
/// `\usepackage{multido}` (0711.2399's `Error:undefined:\multido`). latex: three dots.
#[test]
fn pst_plot_loads_multido() {
  let (log, xml) = convert_with(
    repro!("stub-dependencies/pst_plot_loads_multido.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<dots ").count(), 3, "{xml}");
}
