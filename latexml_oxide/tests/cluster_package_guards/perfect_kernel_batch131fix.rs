//! Red/green guards for int131fix: the three real-paper regressions of the Bundle 1 A/B (int131a vs int130a).
//!
//! - R1, 2410.07647 (2105.02599): a recovered Fatal left the Stomach's stack of tokens being invoked full, so the
//!   build's next digestion was a second Fatal (`stomach::clear_token_stack`, called by `digest_internal`'s recovery
//!   and the streaming loop's resource stop, as Perl's `hardYankProcessing` empties it); and the Fatal itself:
//!   `define_optional_internal` (sect08.rs) named the null control sequence's optional-argument internal `\\`.
//! - R2, 2110.07418: the comma between two names printed inside an ORCID icon's link is the name list's
//!   (`hoist_wrapped_separators`), and a separator the link prints after its icon stays (`\lx@author@href`).
//! - R3, astro-ph0612088: an unspaced list of local parts glued to a word that is no local part is no list
//!   (`write_out_unspaced_lists`), and one whose every part spells an author is written out (`spelled_unspaced_list`,
//!   the Link step `insert::link_entry_addresses`).
//!
//! One guard per code path. The R1 guards convert from the binary, eager and `--streaming` (the two recovery arms); the
//! others through `perfect_kernel_batch99b::converted` (both modes, clean).
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::text,
  perfect_kernel_batch89a::cli_conversion,
  perfect_kernel_batch99b::converted,
  perfect_kernel_batcht1::red_at_two,
};

macro_rules! repro {
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

/// `xml`'s content from its `<title>` (else its start) to its end, each whitespace run one space.
fn body(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</document>")
    .map_or(xml.len(), |end| start + end);
  xml[start..end].trim_end().to_string()
}

/// A Fatal recovered in the document — the Stomach's recursion guard, deep in `\date{\deep}` — leaves no token on the
/// Stomach's stack: the build digests again after it (a frontmatter name read at insert time, 103c's
/// `kind_legend_contacts`) and the build no longer stops early with a second, ghost Fatal; the text before the Fatal and
/// the creator with its affiliation are kept. Exactly one Fatal, eager and streaming (2410.07647, 2105.02599). The
/// input is invalid on purpose (pdflatex: "TeX capacity exceeded"): a robustness guard.
#[test]
fn a_recovered_fatal_leaves_no_token_stack_behind() {
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(
      repro!("macro-state", "recovered_fatal_leaves_no_token_stack"),
      streaming,
    );
    assert_eq!(
      log.matches("Fatal:").count(),
      1,
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      log.matches("Fatal:Stomach:Recursion").count(),
      1,
      "streaming={streaming}: {log}"
    );
    assert!(
      !log.contains("build stopped early"),
      "streaming={streaming}: {log}"
    );
    // (the lax `error_count` reads the recovery Warning's `Error {…}` debug text: count the lines)
    let opening = |prefix: &str| log.lines().filter(|l| l.starts_with(prefix)).count();
    assert_eq!(
      (opening("Error:"), opening("Warning:"), opening("Fatal:")),
      (0, 1, 1),
      "streaming={streaming}: {log}"
    );
    assert_eq!(
      opening("Warning:recovery:digest_internal"),
      1,
      "streaming={streaming}: {log}"
    );
    let xml = body(&xml);
    assert!(
      xml.starts_with(
        r#"<title>T</title> <creator role="author"> <personname>Ann Author</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> "#
      ),
      "streaming={streaming}: {xml}"
    );
    assert!(
      xml.ends_with(r#"<para xml:id="p1"> <p xml:id="p1.1">Before text.</p> </para>"#),
      "streaming={streaming}: {xml}"
    );
  }
}

/// `\expandafter\newcommand\expandafter{\csname\endcsname}[1][x]{…}` defines its optional-argument internal as
/// `\csname\string#1\endcsname` names it, `\\csname\endcsname` (latex.ltx:1252/1255, tex.web §263), and the line break
/// `\\` stays the kernel's: in the text, and in a `\date` the frontmatter digests, where the overwritten `\\` recursed
/// to a Fatal (2410.07647, changes' `\csname\Changes@commandname\endcsname`). The null CS itself still reads its
/// optional argument.
#[test]
fn null_cs_optional_internal_keeps_the_line_break() {
  let (log, xml) = cli_conversion(
    repro!(
      "macro-state",
      "null_cs_optional_internal_keeps_the_line_break"
    ),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    body(&xml)
      .ends_with(r#"<para xml:id="p1"> <p xml:id="p1.1">A<break/>B, and y here.</p> </para>"#),
    "{xml}"
  );
  assert_eq!(
    converted(
      repro!(
        "macro-state",
        "null_cs_optional_internal_line_break_in_the_date"
      ),
      0
    ),
    r#"<title>T</title> <creator role="author"> <personname>Ann Author</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <date role="creation">A<break/>B</date> <para xml:id="p1"> <p xml:id="p1.1">Before text.</p> </para> <para xml:id="p2"> <p xml:id="p2.1">Body text.</p> </para>"#
  );
}

/// The comma between two names printed inside the first's ORCID icon link (`\href{…}{\includegraphics{…}, }%`) parts
/// the names, so each has its own iD, its personname the name and its mark: 103b r2 had made both links the ORCID
/// contacts of one creator "Ann AbleBob Baker", the ", " lost; int130a one creator "Ann Able, Bob Baker" holding both
/// links (2110.07418); a `;` the split leaves inside the link stays where it prints (`\lx@author@href`'s fail-safe).
#[test]
fn orcid_icon_link_closing_with_the_comma_parts_the_names() {
  // (per creator: its personname's text, each contact's role and link)
  let creators = |xml: &str| -> Vec<Vec<String>> {
    xml
      .match_indices("<creator ")
      .map(|(start, _)| {
        let end = start + xml[start..].find("</creator>").expect("closed creator");
        let creator = &xml[start..end];
        let mut out = Vec::new();
        if let Some(at) = creator.find("<personname>") {
          let close = at
            + creator[at..]
              .find("</personname>")
              .expect("closed personname");
          out.push(text(&creator[at + "<personname>".len()..close]));
        }
        for (at, _) in creator.match_indices("<contact ") {
          let tag = &creator[at..at + creator[at..].find('>').expect("contact tag")];
          let close = at + creator[at..].find("</contact>").expect("closed contact");
          let role = tag
            .split("role=\"")
            .nth(1)
            .and_then(|r| r.split('"').next());
          let href = creator[at..close]
            .split("href=\"")
            .nth(1)
            .and_then(|h| h.split('"').next());
          out.push(format!("{}={}", role.unwrap_or(""), href.unwrap_or("")));
        }
        out
      })
      .collect()
  };
  let xml = converted(
    repro!(
      "sectioning-frontmatter",
      "author_orcid_icon_link_closing_with_the_comma"
    ),
    0,
  );
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "orcid=https://orcid.org/0000-0002-1825-0097"],
    vec!["Bob Baker 2", "orcid=https://orcid.org/0000-0001-5109-3700"],
    vec!["Carl Cole 1"],
  ]);
  // (the fail-safe: a `;` the split leaves in the link stays where it prints, as `\lx@author@href` makes the contact —
  // int131a: one creator "Ann AbleBob Baker")
  let xml = converted(
    repro!(
      "sectioning-frontmatter",
      "author_orcid_icon_link_closing_with_a_semicolon"
    ),
    0,
  );
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "orcid=https://orcid.org/0000-0002-1825-0097"],
    vec!["Bob Baker"],
  ]);
}

/// An aa `\institute` entry's `\email{M.R.Pestalozzi,a.chrysostomou,j.l.collett@herts.ac.uk}`, each part spelling one of
/// the entry's three authors, gives each its own address, the capitals as printed; the lowercase list of the next
/// entry stays written out (103a). int131a copied the whole first list onto all three; int130a put both lists whole
/// on a nameless creator (astro-ph0612088).
#[test]
fn unspaced_list_whose_every_part_spells_an_author_is_written_out() {
  if red_at_two(
    "a false markup-lost Warning: \"jconway@oso.chalmers.se\" written as plain text (one address of a list no ink prints alone)",
  ) {
    return;
  }
  assert_eq!(
    converted(
      repro!(
        "sectioning-frontmatter",
        "address_family_unspaced_list_capitalised_spelled"
      ),
      0
    ),
    r#"<title>T</title> <creator role="author"> <personname>M. R. Pestalozzi</personname> <contact name="Affiliation: " role="affiliation">Univ of Hertfordshire, Hatfield, UK,</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">M.R.Pestalozzi@herts.ac.uk</text></contact> </creator> <creator before=" " role="author"> <personname>A. Chrysostomou</personname> <contact name="Affiliation: " role="affiliation">Univ of Hertfordshire, Hatfield, UK,</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id2">a.chrysostomou@herts.ac.uk</text></contact> </creator> <creator before=" " role="author"> <personname>J. L. Collett</personname> <contact name="Affiliation: " role="affiliation">Univ of Hertfordshire, Hatfield, UK,</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id3">j.l.collett@herts.ac.uk</text></contact> </creator> <creator before=" " role="author"> <personname>J. Conway</personname> <contact name="Affiliation: " role="affiliation">Onsala Space Observatory, Sweden, </contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id4">jconway@oso.chalmers.se</text></contact> </creator> <creator before=" " role="author"> <personname>R. S. Booth</personname> <contact name="Affiliation: " role="affiliation">Onsala Space Observatory, Sweden, </contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id5">roy@oso.chalmers.se</text></contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1"><text font="italic" xml:id="abstract1.1.1">Context. </text>A.</p> <p xml:id="abstract1.2"><text font="italic" xml:id="abstract1.2.1">Aims. </text>B.</p> <p xml:id="abstract1.3"><text font="italic" xml:id="abstract1.3.1">Methods. </text>C.</p> <p xml:id="abstract1.4"><text font="italic" xml:id="abstract1.4.1">Results. </text>D.</p> <p xml:id="abstract1.5"><text font="italic" xml:id="abstract1.5.1">Conclusions. </text>E.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}
