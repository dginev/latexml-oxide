//! Guards for CRASH397B: five run-397 crash and Fatal roots, each a class fixed in one place. A cycle of counter reset
//! lists resets each counter once (`reset_counter_in_cascade`); the bibliography mouth reads `@` as data
//! (`BIB_DATA_LITERALS`); an alpha label slices the year by characters (`cleaners::substr`); a defining command's
//! `{\cs}` argument names its first non-space token, the rest its parameter text (`defined_cs`). One guard per path;
//! each converts its repro and asserts the Error/Warning counts and the element the crash lost.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch78i::html_of, perfect_kernel_batch84b::text, perfect_kernel_batchz4::run,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The `<tag …>…</tag>` elements of `xml`, whole.
fn elements<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
  let (open, close) = (format!("<{tag}"), format!("</{tag}>"));
  let mut found = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find(&open) {
    let after = &rest[at..];
    let next = after.as_bytes().get(open.len()).copied();
    if !matches!(next, Some(b' ' | b'>')) {
      rest = &after[open.len()..];
      continue;
    }
    let end = after.find(&close).map_or(after.len(), |e| e + close.len());
    found.push(&after[..end]);
    rest = &after[end..];
  }
  found
}

/// The titles of `xml`'s sections of kind `tag`, in order, as text.
fn titles(xml: &str, tag: &str) -> Vec<String> {
  elements(xml, tag)
    .into_iter()
    .map(|s| {
      let title = elements(s, "title");
      text(title.first().copied().unwrap_or(""))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect()
}

/// `\newcounter{subsubsection}[subsubsection]` (AEA.cls:797, witness 2308.05200): LaTeXML's subsection list resets the
/// counter, which resets itself once, with one Warning naming it (the other is TeX's own "already defined"); its id is
/// not built within itself. Red: a silent stack overflow (`reset_counter` into itself), then a PushbackLimit Fatal (the
/// id).
#[test]
fn counter_in_its_own_reset_list_is_reset_once() {
  let (log, xml) = run(repro!("macro-state/counter_in_its_own_reset_list.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert!(
    log.contains("Counter 'subsubsection' is in a cycle of reset lists"),
    "{log}"
  );
  let subsubsections = elements(&xml, "subsubsection");
  assert_eq!(subsubsections.len(), 1, "{xml}");
  assert_eq!(titles(&xml, "subsubsection"), ["C"], "{xml}");
  assert!(text(subsubsections[0]).contains("Text."), "{xml}");
}

/// A two-counter cycle through `\numberwithin` (subsubsection in foo's list, foo in subsubsection's), reached from
/// subsection's step through LaTeXML's own list (TeX's `\cl@subsection` is empty, as AEA.cls leaves it): the cascade
/// stops at the counter already on its path, one Warning. Distinct from the self-reset: the cycle runs through another
/// counter's list. Red: a silent stack overflow.
#[test]
fn numberwithin_reset_cycle_terminates() {
  let (log, xml) = run(repro!("macro-state/numberwithin_reset_cycle.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Counter 'subsubsection' is in a cycle of reset lists"),
    "{log}"
  );
  assert_eq!(titles(&xml, "section"), ["1 A"], "{xml}");
  assert_eq!(titles(&xml, "subsection"), ["1.1 B"], "{xml}");
  assert_eq!(titles(&xml, "subsubsection"), ["C"], "{xml}");
}

/// The document's own `\stepcounter` of a counter in its own reset list (`\lx@stepcounter@within`), which TeX ends in
/// "TeX capacity exceeded [grouping levels=255]": one Error, and the counter is reset once, as the sectioning step's
/// cascade does, instead of re-entering `\@stpelt` without end: `\arabic` prints 1, the step kept. Red: a silent stack
/// overflow.
#[test]
fn stepcounter_of_a_counter_in_its_own_reset_list_terminates() {
  let tex = repro!("macro-state/counter_in_its_own_reset_list.tex").replace(
    "Text.",
    r"Text \stepcounter{subsubsection}\arabic{subsubsection}.",
  );
  let (log, xml) = run(&tex);
  assert_eq!((error_count(&log), warning_count(&log)), (1, 2), "{log}");
  assert!(
    text(elements(&xml, "subsubsection")[0]).contains("Text 1."),
    "{xml}"
  );
}

/// An amsrefs list under an active `@` (1810.11524's `@book{…}` records): the entry scaffold's handler names read
/// their `@` as data, so the record is one bibentry, its title kept. Red: Fatal:Timeout:Recursion (the active `@` re-entered `\bib`).
#[test]
fn amsrefs_records_under_an_active_at() {
  let (log, xml) = run(repro!("index-bib/amsrefs_records_under_an_active_at.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let entries = elements(&xml, "bibentry");
  assert_eq!(entries.len(), 1, "{xml}");
  let title = elements(entries[0], "bib-title");
  assert_eq!(title, ["<bib-title>Maple 2017.0</bib-title>"], "{xml}");
  assert!(
    entries[0].contains(r#"<bibentry key="maple" type="book""#),
    "{xml}"
  );
}

/// An alpha label of a year whose third byte is inside a character (`1--250, 2023`, 2308.14725, 2607.14650): Perl's
/// `substr($yrtext, 2, 2)` in characters. Red: Fatal:panic:caught (`year[2..4]`).
#[test]
fn alpha_label_of_a_non_ascii_year() {
  let html = html_of(repro!("index-bib/alpha_label_of_a_non_ascii_year.tex"), &[]);
  assert!(
    html.contains(
      r#"<span class="ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem">[DOE25]</span>"#
    ),
    "{html}"
  );
}

/// `\DeclareMathAlphabet{ \mathcal}` (1810.09714, 1906.05222, 2011.04603, 2206.00709, 2209.01842): the command is the
/// first non-space token; the stock `\mathcal` is kept, as Perl keeps it. Red: Fatal:Parameter:Unknown "mathcal".
#[test]
fn declare_math_alphabet_command_after_a_space() {
  let (log, xml) = run(repro!(
    "fonts-nfss/declare_math_alphabet_command_after_a_space.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let tokens = elements(&xml, "XMTok");
  assert_eq!(tokens.len(), 1, "{xml}");
  assert!(
    tokens[0].contains(r#"font="caligraphic""#) && tokens[0].ends_with(">L</XMTok>"),
    "{xml}"
  );
}

/// `\urldef{\survey2011}\url{…}` (1312.0976): url.sty:185's `\edef#1` defines `\survey` delimited by `2011`, which the
/// text's `\survey2011` calls. Red: Fatal:Parameter:Unknown "2011".
#[test]
fn urldef_command_with_parameter_text() {
  let (log, xml) = run(repro!(
    "package-bindings/urldef_command_with_parameter_text.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let paras = elements(&xml, "para");
  assert_eq!(paras.len(), 1, "{xml}");
  let words = text(paras[0])
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
  assert_eq!(words, "See https://example.org/a%26b now.", "{xml}");
}

/// The same helper at a sibling definer: xcolor's `\extractcolorspec{red}{ \spec}` (xcolor.sty:1032 `\aftergroupdef#2`)
/// defines `\spec`. Red: Error:undefined `\spec` (the command ` \spec` was defined).
#[test]
fn extractcolorspec_command_after_a_space() {
  let (log, xml) = run(repro!(
    "package-bindings/extractcolorspec_command_after_a_space.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let paras = elements(&xml, "para");
  assert_eq!(paras.len(), 1, "{xml}");
  let words = text(paras[0])
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
  assert_eq!(words, "Spec rgb1,0,0 end.", "{xml}");
}
