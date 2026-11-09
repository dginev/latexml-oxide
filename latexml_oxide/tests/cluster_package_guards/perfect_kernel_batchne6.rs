//! Guards for NE6 (`agent_reports/2026-11-09_brief_NE6.md`): the last `=2`-only Errors of the flip A/B. A mark a macro
//! prints is written on its name as the call with its arguments, and a run a call in which reads past it is no run
//! (aiaa-tc's `\thanksibid{2}`: 1505.07844, 1411.3642, 1602.00849; apa6 with a preamble's `\footrecall{uva}`: 2503.21719,
//! 2407.13730, 2606.19583; `\author\href{url}{name}`: 2603.11001); an IJCAI sectioned block's address section, from
//! `\emails` on, is the class's own split's (2501.02356; NE8).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// Convert `tex`: the log and the XML.
fn run(tex: &str) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(None);
    },
    |_| (),
  );
  (log, xml)
}

/// aiaa-tc's `Carl Cole\thanksibid{2}` under calc: the footnote mark's span is the call alone, so `=2` wrote the name
/// `Carl Cole\thanksibid`, whose `\setcounter{…}{#1}` read the author wrapper's `\let` (calc: `\let` invalid, an Error).
/// The mark is the call with its argument now (`content::arguments_end`), the creators off's.
#[test]
fn a_mark_macro_is_written_with_its_arguments() {
  let (log, xml) = run(repro!("ne6_mark_macro_arguments"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=Princeton University"]),
    row(&["Bob Baker", "affiliation=Princeton University"]),
    row(&[
      "Carl Cole † † footnote †",
      "affiliation=Princeton University"
    ]),
  ]);
}

/// `\author[2]{\author\href{url}{Bob~Baker}}` under authblk: `=2` cut the URL contact out of its `\href` as
/// `\author\href{url}`, whose `\href` read the author wrapper's `\let` as its text, `\lx@saved@inst` undefined (an Error).
/// A run a call in which reads past it (`\href`'s verbatim URL and `\author`'s option judged) is no run: the URL is its
/// text, counted. Off has no Error (its third creator two empty links).
#[test]
fn a_url_cut_from_its_href_is_written_as_its_text() {
  let (log, xml) = run(repro!("ne6_href_url_cut_from_its_call"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("writes \"https://orcid.org/0009-0002-2849-5423\" as its plain text"),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=Univ A"]),
    row(&["Bob Baker", "affiliation=Univ B"]),
    row(&["url=https://orcid.org/0009-0002-2849-5423"]),
  ]);
}

/// An IJCAI block (a class defining `\affiliations`/`\emails` only in its title code): the reading continued `Univ B`
/// onto the e-mail line past `\emails`, whose rehearsal held an Error there — `=2` wrote "Univ B" twice (the entry by its
/// text, and the class split's `$^2$Univ B`) and the addresses as an affiliation (2501.02356, review revX4 1). The
/// address section is the class's own split's now (`Resolver::block`'s `markers`): the creators off's, the e-mail call
/// written as queued, counted.
#[test]
fn an_ijcai_emails_section_is_the_class_split() {
  let expected = [
    row(&["Ann Able", "affiliation=Univ A", "email=ann, bob @x.edu"]),
    row(&["Bob Baker", "affiliation=Univ B"]),
  ];
  let (log, xml) = run(repro!("ne6_ijcai_entry_past_emails"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "The frontmatter emitter's reading holds no place for words its author block or call prints (x edu); the call \
       is written as queued, as the switch off writes it"
    ),
    "{log}"
  );
  assert_eq!(creators(&xml), expected);
}
