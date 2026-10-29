//! Red/green guards for batch 91a: a binding for just.cls, cpc-hepnp.cls (the Chinese Physics C template) under the
//! name the PhiPsi15 proceedings ship it as (OXIDIZED_DESIGN_DIVERGENCES #912). The class loads raw as written; the
//! binding routes its `\email` (just.cls:100-105, an "n) E-mail:" note in `\@thanks`, which the kernel's locked
//! `\maketitle` never set) to `\lx@add@email`, and its `\address` (:173-181, printed where it is called, after
//! `\maketitle`, below the whole name block) to `\lx@add@affiliation@marked[annotate=all]`, the marked list iopart's
//! `\address` reads. Registered as `just` only: cpc-hepnp routing waits for 91c (its `,n)`/`;n)` e-mail marks and
//! unmarked multi-line lists gave about 90 papers affiliations on wrong names), so a `\documentclass{cpc-hepnp}` paper
//! converts as before (the no-change row). Witnesses 1603.03230 (55 names, none with an affiliation before),
//! 1511.03783, 1511.03934, 1511.08514 (just.cls); 1101.0224, 0904.1329 (cpc-hepnp.cls, one unmarked address under
//! several names). Each guard converts its repro under `tools/perfect_kernel/repros/class-bindings/` (a stand-in class
//! cut to those definitions, written by `{filecontents*}`) and asserts no Error or Warning and every creator whole. One
//! guard per path of the `\address` setter: a marked list after `\maketitle`, built eager and `--streaming` (the late
//! frontmatter path); an unmarked address under one name; an unmarked address under several names, every name's,
//! eager and `--streaming` (the late `annotate=all` path). And the class's other e-mail setter, `\emails` (:107-109,
//! the footnote set where it is called), after `\maketitle`: no census paper calls it, so its guard pins the mapping
//! the 2009-13 copies, which lack it, skip. The `;n)` e-mail mark (`$^{1,2;1)}$`) is the RED catalog repro
//! `just_semicolon_email_marks.tex`, deferred (OD #912).
use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// Checks a conversion's log has no Error and no Warning, and returns the creators of its XML.
#[track_caller]
fn clean_creators(log: &str, xml: &str) -> Vec<Vec<String>> {
  assert_eq!((error_count(log), warning_count(log)), (0, 0), "{log}");
  creators(xml)
}

#[track_caller]
fn owned(expected: &[&[&str]]) -> Vec<Vec<String>> {
  expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect()
}

/// 1603.03230's shape: names with `$^{n}$` marks, `\quad` separators and a `\\` row, the first with its `\email`, then
/// `\maketitle`, then the marked `\address` list. Each name holds the lines its marks request and Ann Able her
/// e-mail, labelled as the class prints it; no copy of the list is left in the body. The same in process, from the
/// binary, and from the binary `--streaming`, where the list arrives after the frontmatter was placed.
#[test]
fn just_marked_address_after_maketitle_affiliates_every_name() {
  let tex = repro!("just_marked_address_after_maketitle.tex");
  let want = owned(&[
    &[
      "Ann Able",
      "email=ann@x.org",
      "affiliation=Univ A, Town, Country",
      "affiliation=Univ B, City, Country",
    ],
    &["Bob Baker", "affiliation=Univ A, Town, Country"],
    &["Cy Cole", "affiliation=Univ B, City, Country"],
    &[
      "Di Dunn",
      "affiliation=Univ A, Town, Country",
      "affiliation=Lab C, Place, Country",
    ],
  ]);
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!(clean_creators(&log, &xml), want, "{xml}");
  // (the class's label, `E-mail:~`, its tie a no-break space)
  assert!(
    xml.contains("<contact name=\"E-mail:\u{a0}\" role=\"email\">ann@x.org</contact>"),
    "{xml}"
  );
  assert_eq!(xml.matches("Univ B, City, Country").count(), 2, "{xml}");
  assert!(!xml.contains("ltx_titlepage_continued"), "{xml}");
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!(clean_creators(&log, &eager), want, "{eager}");
  let (log, streamed) = cli_conversion(tex, true);
  assert_eq!(clean_creators(&log, &streamed), want, "{streamed}");
  assert_eq!(eager, streamed);
}

/// 1511.03783's shape: one name with its `\email`, and after `\maketitle` an `\address` with no mark: one
/// affiliation, the name's.
#[test]
fn just_unmarked_address_is_its_author_s_affiliation() {
  let (log, xml) = convert_with(repro!("just_unmarked_address.tex"), Some(PRELOAD));
  assert_eq!(
    clean_creators(&log, &xml),
    owned(&[&[
      "Ann Able",
      "email=ann@x.org",
      "affiliation=Center for Theory, Univ A, Town, Country",
    ]]),
    "{xml}"
  );
  assert_eq!(xml.matches("Center for Theory").count(), 1, "{xml}");
}

/// 1101.0224's and 0904.1329's shape (cpc-hepnp.cls papers) under `\documentclass{just}`: several names on one line,
/// the first with its `\email`, and after `\maketitle` one `\address` with no mark, which the class prints centred
/// below all of them: every name's affiliation, not the last name's alone (review rev91a B1). The same from the
/// binary, eager and `--streaming`, where the address arrives after the frontmatter was placed.
#[test]
fn just_unmarked_address_under_several_names_is_every_name_s() {
  let tex = repro!("just_unmarked_address_under_several_names.tex");
  let address = "affiliation=Center for Theory, Univ A, Town, Country";
  let want = owned(&[
    &["Ann Able", "email=ann@x.org", address],
    &["Bob Baker", address],
    &["Cy Cole", address],
  ]);
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!(clean_creators(&log, &xml), want, "{xml}");
  assert!(!xml.contains("<p>Center for Theory"), "{xml}");
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!(clean_creators(&log, &eager), want, "{eager}");
  let (log, streamed) = cli_conversion(tex, true);
  assert_eq!(clean_creators(&log, &streamed), want, "{streamed}");
  assert_eq!(eager, streamed);
}

/// The same input under `\documentclass{cpc-hepnp}`: a NO-CHANGE row. cpc-hepnp routing waits for 91c (OD #912), so
/// the class's copy is read raw as before the binding (int119a's output): three creators with no affiliation, the
/// address the class's centred body paragraph, and the e-mail in the `\@thanks` the kernel's `\maketitle` never sets.
/// Distinct from the guard above: it pins that `cpc-hepnp` is not routed through the `just` binding.
#[test]
fn cpc_hepnp_is_not_routed_until_91c() {
  let tex = repro!("cpc_hepnp_unmarked_address_under_several_names.tex");
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!(
    clean_creators(&log, &xml),
    owned(&[&["Ann Able"], &["Bob Baker"], &["Cy Cole"]]),
    "{xml}"
  );
  assert!(
    xml.contains(
      "<p align=\"center\" xml:id=\"p1.1\"><text fontsize=\"80%\" xml:id=\"p1.1.1\">Center for Theory, Univ A, Town, \
       Country</text></p>"
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("Center for Theory").count(), 1, "{xml}");
  assert!(!xml.contains("ann@x.org"), "{xml}");
}

/// `\emails` after `\maketitle` (just.cls:107-109; no census paper calls it): the footnote it sets where it is called,
/// "1) E-mail: x", is the e-mail of the name above it, labelled as the class prints it, not a body footnote. Inside
/// `\author` pdflatex loses it (a `\footnotetext` in the title tabular), so this is the shape that prints.
#[test]
fn just_emails_after_maketitle_is_the_author_s_e_mail() {
  let (log, xml) = convert_with(repro!("just_emails_after_maketitle.tex"), Some(PRELOAD));
  assert_eq!(
    clean_creators(&log, &xml),
    owned(&[&[
      "Ann Able",
      "affiliation=Center for Theory, Univ A, Town, Country",
      "email=ann@x.org",
    ]]),
    "{xml}"
  );
  assert!(
    xml.contains("<contact name=\"E-mail:\u{a0}\" role=\"email\">ann@x.org</contact>"),
    "{xml}"
  );
  assert_eq!(xml.matches("ann@x.org").count(), 1, "{xml}");
}
