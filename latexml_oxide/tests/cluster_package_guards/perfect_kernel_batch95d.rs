//! Red/green guards for batch 95d: the last four lost-words papers of the frontmatter scorer's 3,000-paper sample,
//! three K11 store mechanisms (latexml_engine/src/frontmatter_stores.rs, OXIDIZED_DESIGN_DIVERGENCES #939-#941):
//! a raw class's token register printed only by the page style its `\maketitle` selects (`toks_store`, with
//! fancyhdr_sty.rs's faithful `\fancypagestyle`; 2607.06973, 2608.26053, 2604.13010, 2401.14196), a raw package's
//! table stores rerouted as a raw class's are (`reroute_raw_package_stores`; 2307.16262), and a table setter that
//! prints a mark before its pure store (`table_setter_stores`; 1202.2889); and (95d-r3) how a date's printed label is
//! read (`labelled_api`, `line_label_before`: 2408.01858, 2607.07582, 2311.02753, 1211.4450). Each guard converts its
//! repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, with no
//! Error and no Warning, and asserts the frontmatter it makes whole.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    repro!("sectioning-frontmatter", $name)
  };
  ($dir:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $name,
      ".tex"
    ))
  };
}

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and no Warning and both give the
/// same XML, and returns it squashed.
#[track_caller]
fn clean_both_ways(tex: &str) -> String {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (log, streamed) = cli_conversion(tex, true);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(eager, streamed);
  squash(&eager)
}

/// Converts `tex` eager, checks it has no Error and no Warning, and returns it squashed.
#[track_caller]
fn clean(tex: &str) -> String {
  let (log, xml) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  squash(&xml)
}

/// The document from its `<title>` to its end: the frontmatter and the whole body.
fn front(xml: &str) -> &str {
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</document>")
    .map_or(xml.len(), |end| start + end);
  xml[start..end].trim_end()
}

/// googlecloud.cls's corresponding-author token register, printed only in the foot of the fancyhdr page style its
/// `\maketitle` selects, is the frontmatter's corresponding note, led by the label the class prints before it; the
/// same register is no note when the page style is not selected, or when the foot only tests it (nvidiatechreport.cls's
/// copy in 2507.09850 prints "* Equal contribution." instead). Witnesses 2607.06973, 2608.26053, 2604.13010.
#[test]
fn toks_store_printed_in_a_selected_page_style_is_a_corresponding_note() {
  let tex = repro!("toks_store_printed_in_a_fancy_page_style_is_a_note");
  assert_eq!(
    front(&clean_both_ways(tex)),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <note role="corresponding" xml:id="id1">Corresponding author(s): ann@x.org</note> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  let plain = r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  // (the page style no `\maketitle` selects prints nothing)
  let unselected = tex.replace(r"\par}\thispagestyle{firststyle}}", r"\par}}");
  assert_ne!(unselected, tex);
  assert_eq!(front(&clean(&unselected)), plain);
  // (a foot that only tests the register prints the class's own text, not the register)
  let tested = tex.replace(
    r"\else Corresponding author(s): \the\correspondingauthor\fi",
    r"\else Equal contribution.\fi",
  );
  assert_ne!(tested, tex);
  assert_eq!(front(&clean(&tested)), plain);
}

/// A raw package's pure date stores under elsarticle (medima.sty's "Article history") are the frontmatter's five dates,
/// in the roles fixed for them: received, final form `revised`, accepted, available online `published`, communicated —
/// the two of no default name both kept —, each named by the label the package's title page prints before it, in
/// medima's shape a head macro in the `\else` branch of the store's emptiness test, which leaves the title page with it;
/// a date whose label is not read has no name, not its role's default (review rev95d B2: "Revised " for "Received in
/// final form"). Under article, whose kernel `\maketitle` deposits the package's refused one, the same: each date printed
/// once, no lone label left in the body, the "Article history:" heading kept. Witnesses 2307.16262, 2104.10903.
#[test]
fn raw_package_date_stores_are_the_frontmatter_dates() {
  let tex = repro!("raw_package_history_dates_are_the_frontmatter_dates");
  let head =
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator>"#;
  let dates = r#"<date name="Received" role="received">1 May 2020</date> <date name="Received in final form" role="revised">9 May 2020</date> <date name="Accepted" role="accepted">2 June 2020</date> <date name="Available online" role="published">5 June 2020</date> <date name="Communicated by" role="communicated">B. Editor</date>"#;
  let text = r#"<para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(
    front(&clean_both_ways(tex)),
    format!("{head} {dates} {text}")
  );
  // (a date printed with no label: unnamed)
  let bare = tex.replace(r"\receivedhead~\@received", r"\@received");
  assert_ne!(bare, tex);
  assert_eq!(
    front(&clean(&bare)),
    format!(
      "{head} {} {text}",
      dates.replace(
        r#"<date name="Received" role="received">"#,
        r#"<date role="received">"#
      )
    )
  );
  let article = tex
    .replace(r"\documentclass{elsarticle}", r"\documentclass{article}")
    .replace(r"\elsauthors", r"\@author")
    .replace(r"\begin{frontmatter}", "")
    .replace(r"\end{frontmatter}", r"\maketitle");
  assert_ne!(article, tex);
  assert_eq!(
    front(&clean(&article)),
    format!(
      r#"{head} {dates} <para xml:id="p1"> <p xml:id="p1.1"><text font="italic" xml:id="p1.1.1">Article history</text>:<break/></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
    )
  );
}

/// A raw package's setter whose own store other code of the package prints — a macro the document calls
/// (`\printkeywords`), a `\printhistory` in its body — stays the package's: the text printed once, where it prints, no
/// frontmatter item (int124a's output); and one only uncalled code reads (rlj.sty:73's `\@keywords`, read only by the
/// `\makeCover` 2605.31044 comments out) is no item at all, as pdflatex prints it nowhere. Review rev95d B1, probes p5,
/// p7 (a raw class's, p9, stays RED: `raw_class_store_only_an_uncalled_macro_reads_is_no_item`).
#[test]
fn raw_package_store_other_code_prints_stays_where_it_prints() {
  let tex = repro!("raw_package_store_a_document_macro_prints_stays_in_the_body");
  let head = r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abs.</p> </abstract>"#;
  assert_eq!(
    front(&clean_both_ways(tex)),
    format!(
      r#"{head} <para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Keywords:</text> alpha, beta</p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
    )
  );
  // (the reader never called: no keywords anywhere)
  let uncalled = tex.replace("\\printkeywords\nText.", "Text.");
  assert_ne!(uncalled, tex);
  // (an uncalled macro reads the store: the third source cannot tell it is uncalled, so the words are told as ones
  // pdflatex may not print, OD #953 "108b")
  let (log, xml) = cli_conversion(&uncalled, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Warning:frontmatter:unclaimed_store 2 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: alpha beta"),
    "{log}"
  );
  assert_eq!(
    front(&squash(&xml)),
    format!(r#"{head} <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#)
  );
  assert_eq!(
    front(&clean_both_ways(repro!(
      "raw_package_dates_printed_in_the_body_stay_there"
    ))),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para> <para class="ltx_noindent" xml:id="p2"> <p xml:id="p2.1">Received 1 May 2020; accepted 2 June 2020.</p> </para>"#
  );
}

/// A fancyhdr page style a shipped package gives a binding's class (popets.sty's `standardpagestyle` under the acmart
/// binding, its code testing acmart.cls's `\if@ACM@printfolios` and `\if@ACM@timestamp`) is stored to be read, and
/// selecting it runs nothing (fancyhdr_sty.rs, as Perl's no-op `\fancypagestyle`): no Error pdflatex does not raise, the
/// document as int124a converts it; a style using `\fancycenter` (fancyhdr.sty:378) is selected clean too. Witnesses
/// 2408.16304, 2602.20222, 2606.05273 (9 Errors each when the style ran); review rev95d S3, probe p3.
#[test]
fn a_fancy_page_style_a_package_gives_a_binding_class_selects_nothing() {
  let xml = clean_both_ways(repro!(
    "package-bindings",
    "fancy_page_style_a_package_gives_a_binding_class_selects_nothing"
  ));
  let doc = front(&xml);
  // (the copyright year is acmart's `\the\year`, the conversion's)
  let year = doc
    .split_once(r#"<date name="©" role="copyright">"#)
    .map_or("", |(_, rest)| &rest[..rest.len().min(4)]);
  assert!(
    year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit()),
    "{doc}"
  );
  assert_eq!(
    doc,
    format!(
      r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <date name="©" role="copyright">{year} Copyright held by the owner/author(s). Publication rights licensed to ACM.</date> <pubnote role="license">Permission to make digital or hard copies of all or part of this work for personal or classroom use is granted without fee provided that copies are not made or distributed for profit or commercial advantage and that copies bear this notice and the full citation on the first page. Copyrights for components of this work owned by others than the author(s) must be honored. Abstracting with credit is permitted. To copy otherwise, or republish, to post on servers or to redistribute to lists, requires prior specific permission and/or a fee. Request permissions from permissions@acm.org.</pubnote> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
    )
  );
  let fancycenter = "\\documentclass{article}\n\\usepackage{fancyhdr}\n\\fancypagestyle{mystyle}{\\fancyhf{}\\fancycenter{a}{b}{c}}\n\\pagestyle{mystyle}\n\\begin{document}\nText.\n\\end{document}\n";
  let xml = clean(fancycenter);
  assert!(
    xml.ends_with(r#"<para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para> </document>"#),
    "{xml}"
  );
}

/// A raw class's title page prints its history dates in one line, a clause each (pazha2.cls:166-167's `Received
/// \@submitted; revised \@revised; accepted \@accepted`): each date is named by its own clause's label, as printed — a
/// label phrase ends at a `;` or a `,` —, in the line's order, and the separators leave the title page with the
/// clauses; a store right after a separator has no label (iaus.cls, 0707.3527). Witnesses 2408.01858, 2607.07582
/// (review rev95d-r2 B: unnamed, the defaults before).
#[test]
fn raw_class_dates_in_one_line_are_named_by_their_clauses() {
  let tex = repro!("raw_class_dates_in_one_line_are_named_by_their_clauses");
  let expected = r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <date name="Received" role="received">15 December 2022</date> <date name="revised" role="revised">18 April 2023</date> <date name="accepted" role="accepted">2 June 2023</date> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(front(&clean_both_ways(tex)), expected);
  // (the clauses joined by commas)
  let commas = tex.replace(
    "\\@received; revised\n  \\@revised; accepted",
    "\\@received, revised\n  \\@revised, accepted",
  );
  assert_ne!(commas, tex);
  assert_eq!(front(&clean(&commas)), expected);
  // (a store right after a separator has no label of its own: the separator ends the clause before it — iaus.cls's
  // `Proceedings IAU Symposium No. \@volume, \@pubyear`, 0707.3527 — so the accepted date is unnamed; the unnamed
  // date's place before the labelled ones and the ";" it leaves are SYNC "K11 store residuals after 95d" (j), (l))
  let bare = tex.replace(
    "\\@revised; accepted \\@accepted}",
    "\\@revised; \\@accepted}",
  );
  assert_ne!(bare, tex);
  let doc = front(&clean(&bare)).to_string();
  for date in [
    r#"<date role="accepted">2 June 2023</date>"#,
    r#"<date name="Received" role="received">15 December 2022</date>"#,
    r#"<date name="revised" role="revised">18 April 2023</date>"#,
  ] {
    assert_eq!(doc.matches(date).count(), 1, "{doc}");
  }
  assert_eq!(doc.matches("<date ").count(), 3, "{doc}");
}

/// A raw package's history box, printed by a title macro the refused `\maketitle` reaches through a measuring box and
/// an uncalled variant calls too — so it never grows into the title page —, names each date by the label printed before
/// it: a date's label is read in any code the title page reaches. Witness 2311.02753 (jasr.sty's `\elsarthistorybox` in
/// `\MaketitleBox`; review rev95d-r2 B: unnamed, the default before).
#[test]
fn raw_package_history_box_a_shared_title_macro_prints_is_named() {
  assert_eq!(
    front(&clean_both_ways(repro!(
      "raw_package_history_box_a_shared_title_macro_prints_is_named"
    ))),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <date name="Received" role="received">1 May 2020</date> <date name="Accepted" role="accepted">2 June 2020</date> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A date's label printed before the store's emptiness test, the store alone in its `\else` branch (`Received:
/// \ifx\@received\@empty--\else\@received\fi`), names the date, and the whole test leaves the title page with it: no
/// "Received: –" in the body (review rev95d-r2's sibling of 2311.02753).
#[test]
fn a_date_labelled_before_its_emptiness_test_is_named() {
  assert_eq!(
    front(&clean_both_ways(repro!(
      "raw_class_date_labelled_before_its_emptiness_test_is_named"
    ))),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <date name="Received:" role="received">1 May 2020</date> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// Where no label is read before a date — wiley2sp.cls:296-302's `\receivedname\ignorespaces\@received`, whose
/// `\def\receivedname{{\normalfont Received:}\ }` the line reader takes for a value —, the role's default name stands
/// only when its word is what the class prints right before the store: "Received " under "Received:" (1211.4450,
/// 1306.2559, as before 95d-r2) and under `\mbox{Received}\hspace{1.5mm}` (rose_v01.cls, 0907.2032), no name under
/// "Submitted:". (The printed label stays in the body: SYNC "K11 store
/// residuals after 95d" (l).)
#[test]
fn a_date_with_no_label_read_keeps_its_default_only_where_printed() {
  let tex = repro!("raw_class_date_labelled_before_its_emptiness_test_is_named")
    .replace(
      r"\def\received#1{\gdef\@received{#1}}",
      r"\def\received#1{\gdef\@received{#1}}\def\receivedname{{\normalfont Received:}\ }",
    )
    .replace(
      r"Received: \ifx\@received\@empty--\else\@received\fi",
      r"\receivedname\ignorespaces\@received",
    );
  let head =
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator>"#;
  let body = |label: &str| {
    format!(
      r#"<para xml:id="p1"> <p align="center" xml:id="p1.1"><text fontsize="80%" xml:id="p1.1.1">{label}</text></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
    )
  };
  assert_eq!(
    front(&clean_both_ways(&tex)),
    format!(
      r#"{head} <date name="Received " role="received">1 May 2020</date> {}"#,
      body("Received:")
    )
  );
  // (a layout command's braced argument between the word and the store prints nothing: rose_v01.cls:259's
  // `\text{Received}\hspace{1.5mm}\@received`, 0907.2032)
  let spaced = repro!("raw_class_date_labelled_before_its_emptiness_test_is_named").replace(
    r"Received: \ifx\@received\@empty--\else\@received\fi",
    r"\mbox{Received}\hspace{1.5mm}\@received",
  );
  let doc = front(&clean(&spaced)).to_string();
  assert_eq!(
    (
      doc
        .matches(r#"<date name="Received " role="received">1 May 2020</date>"#)
        .count(),
      doc.matches("<date ").count()
    ),
    (1, 1),
    "{doc}"
  );
  let other = tex.replace("Received:}", "Submitted:}");
  assert_ne!(other, tex);
  assert_eq!(
    front(&clean(&other)),
    format!(
      r#"{head} <date role="received">1 May 2020</date> {}"#,
      body("Submitted:")
    )
  );
}
