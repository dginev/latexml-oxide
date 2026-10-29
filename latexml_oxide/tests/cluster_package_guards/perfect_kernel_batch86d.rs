//! Red/green guards for batch 86d, two causes of lost title-page words. (1) The `\maketitle` replay gate
//! (sect05.rs `body_vocabulary_is_defined`) took a token TeX compares or shows but never runs for code that runs:
//! `\ifx`'s two comparands (tex.web §507), e-TeX's `\ifdefined`'s one, `\string`'s and `\meaning`'s (§470-471) —
//! zaa.cls:166 `\ifx\@dedication\undefined` refused the class's title page, and the addresses
//! only it prints were lost (1902.07328; `unrun_operands`, OXIDIZED_DESIGN_DIVERGENCES #906). (2) A full stop closing
//! an IEEE author block (`…,\\ and Farshid Alambeigi$^{1}$ \IEEEmembership{Member, IEEE}\thanks{…}.}`, 2405.17600)
//! made its last `\\`-line read as no names, so the binding dropped the grade; IEEEtran's binding now takes out a stop
//! closing the block right after a grade (ieeetran_cls.rs `without_stop_after_grade`; Perl drops every grade,
//! KNOWN_PERL_ERRORS #534), only where no note changes hands by it (unmarked: the notes the block's, 2112.14209,
//! 2409.11755, 2510.15773; marked names and notes: read by their marks; any other block as the base reads it). One guard
//! per path: `\ifx`'s two comparands, the `\expandafter\ifx\csname…\endcsname` first comparand, the one-token operands,
//! what the gate still checks (`\noexpand`'s operand, an `\expandafter\ifx` first comparand not made by `\csname`), the
//! closing stop of a plain `\author` (`name_memberships`' line test) and of an `\IEEEauthorblockN` (its cautious split,
//! 2405.10825).
//! Each repro's pdflatex reference text is in its header; every guard asserts no Error/Warning and the whole output.
use latexml::util::test::convert_files_with;

use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch84b::{creators, text},
};

const PRELOAD: &str = "ar5iv.sty";

const IFX_TEX: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title_page/maketitle_replay_gate_ifx_comparands_are_not_run.tex"
);
const IFX_CLS: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title_page/tpifx.cls"
);
/// The stand-in class's title-page condition, which the variants replace.
const IFX_TEST: &str = r"\ifx\@dedication\undefined\else{\itshape\@dedication\par}\fi";

/// Converts the repro beside the stand-in class with its title-page condition replaced by `test`, and checks that the
/// title page was replayed: no Error or Warning, no refusal, and the address it prints one paragraph between the
/// frontmatter and the body.
#[track_caller]
fn assert_address_replayed(test: &str) {
  assert!(
    IFX_CLS.contains(IFX_TEST),
    "the stand-in holds the condition"
  );
  let cls = IFX_CLS.replace(IFX_TEST, test);
  let (log, xml) = convert_files_with(IFX_TEX, &[("tpifx.cls", &cls)], Some(PRELOAD));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, 0),
    "{test}\n{log}"
  );
  assert!(!log.contains("not replayed"), "{test}\n{log}");
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert!(
    xml.contains(
      r#"<title>On Stability</title> <creator role="author"> <personname>Elena Braverman</personname> </creator> <para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1">University of Calgary, Canada</p> </para> <para xml:id="p2"> <p xml:id="p2.1">Body.</p> </para> </document>"#
    ),
    "{test}\n{xml}"
  );
  assert_eq!(xml.matches("Calgary").count(), 1, "{test}\n{xml}");
}

/// zaa.cls:166 `\ifx\@dedication\undefined` (1902.07328): `\ifx`'s comparands are compared, never run — LaTeX's
/// `\@undefined` and a macro's parameter (`\ifx#1\undefined`, the call's argument) alike.
#[test]
fn maketitle_replay_gate_does_not_run_ifx_comparands() {
  assert_address_replayed(IFX_TEST);
  assert_address_replayed(r"\ifx\@dedication\@undefined\else{\itshape\@dedication\par}\fi");
  assert_address_replayed(
    r"\def\tpifx@test##1{\ifx##1\undefined\else{\itshape##1\par}\fi}\tpifx@test\@dedication",
  );
}

/// `\expandafter\ifx\csname…\endcsname\undefined`: the name's tokens run as the condition makes its first comparand,
/// and the token after `\endcsname` is the second, compared only.
#[test]
fn maketitle_replay_gate_reads_a_csname_comparand() {
  assert_address_replayed(
    r"\expandafter\ifx\csname tpifx@journal\endcsname\undefined\else{\itshape\@dedication\par}\fi",
  );
}

/// The one-token operands: e-TeX's `\ifdefined` tests, `\string` and `\meaning` show — an undefined control sequence
/// there is none TeX runs.
#[test]
fn maketitle_replay_gate_does_not_run_shown_tokens() {
  for test in [
    r"\ifdefined\tpifx@journal\else{\itshape\@dedication\par}\fi",
    r"\typeout{(\string\tpifx@journal)}",
    r"\edef\tpifx@m{\meaning\tpifx@journal}",
  ] {
    assert_address_replayed(test);
  }
}

/// What the gate still checks, as code that runs (OXIDIZED_DESIGN_DIVERGENCES #906): `\noexpand`'s operand, which an
/// `\edef` stores and its call runs (§367), and an `\expandafter\ifx` first comparand not made by `\csname`, which is
/// expanded (§367). Each is defined under pdflatex (hyperref's, 0 errors, the address printed) and not here: the body
/// is refused by the gate, naming it, not admitted and then dropped by the replay's error net (1 error each in 86d's
/// first round).
#[test]
fn maketitle_replay_gate_runs_what_noexpand_stores_and_expandafter_expands() {
  let hyperref = IFX_CLS.replace(
    r"\LoadClass{article}",
    "\\LoadClass{article}\n\\RequirePackage{hyperref}",
  );
  for (test, cs) in [
    (
      r"\edef\tpifx@x{\noexpand\Hy@SectionAnchorHref{a}}\tpifx@x",
      r"\Hy@SectionAnchorHref",
    ),
    (
      r"\expandafter\ifx\Hy@pdfversion\relax\else{\itshape\@dedication\par}\fi",
      r"\Hy@pdfversion",
    ),
  ] {
    let cls = hyperref.replace(IFX_TEST, test);
    assert_ne!(cls, hyperref, "the stand-in holds the condition");
    let (log, xml) = convert_files_with(IFX_TEX, &[("tpifx.cls", &cls)], Some(PRELOAD));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{test}\n{log}"
    );
    assert!(
      log.contains(&format!(
        "The \\maketitle body was not replayed: it uses {cs}, which is undefined here"
      )),
      "{test}\n{log}"
    );
    // (refused, the class's title page prints nothing here: its address is lost, as on the base)
    let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
      xml.contains(
        r#"<title>On Stability</title> <creator role="author"> <personname>Elena Braverman</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para> </document>"#
      ),
      "{test}\n{xml}"
    );
  }
}

/// 2405.17600: the full stop (or semicolon) closing an IEEE author block after a grade is no part of its last line of
/// names, after the author's note or before it, past a blank line, `\vspace` or `\\`, in ieeeconf (the witness's class)
/// and IEEEtran; the grade stays its author's. The stop goes only where no note changes hands by it, by the shared
/// reader's tests of a mark: with a label (`$^{1}$`, not `$^{*}$`) marking a name and leading a note (a `\let` alias
/// of `\textsuperscript` too, the 86d r2 re-review's B1), the notes are read by their marks (each author's affiliation); fully unmarked, they are the block's
/// — the document's notes, in order, none an author's by position (2112.14209, 2409.11755, 2510.15773; a compsoc
/// `\IEEEcompsocitemizethanks` alike); with marks on the names alone (`$^{*}$` over a correspondence note, the
/// re-review's m4), or names marked only by symbols over a note led by a number (the 86d r3 re-review's a9), the block
/// is left as the base reads it, its notes the document's.
#[test]
fn ieee_membership_kept_after_closing_stop() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_membership_after_closing_stop.tex"
  );
  let thanks = "affiliation=All are with the University of Texas at Austin.";
  let marked: &[&[&str]] = &[&["Susheela Sharma", thanks], &["Sarah Go", thanks], &[
    "Farshid Alambeigi",
    "membership=Member, IEEE",
    thanks,
  ]];
  let plain: &[&[&str]] = &[&["Susheela Sharma"], &["Sarah Go"], &[
    "Farshid Alambeigi",
    "membership=Member, IEEE",
  ]];
  let note = "\\thanks{$^{1}$All are with the University of Texas at Austin.}";
  let sharma = "\\thanks{S. Sharma is with the University of Texas at Austin.}";
  let alambeigi = "\\thanks{F. Alambeigi is with the University of Texas at Austin.}";
  let compsoc = "\\IEEEcompsocitemizethanks{\\IEEEcompsocthanksitem S. Sharma is with the University of Texas at Austin.}";
  let both: &[&str] = &[
    "S. Sharma is with the University of Texas at Austin.",
    "F. Alambeigi is with the University of Texas at Austin.",
  ];
  let as_written = author_block("$^{1}$", &format!("\n{note}."));
  assert!(tex.contains(&as_written), "the repro's author block");
  let journal = |tail: &str| {
    tex
      .replace("[conference]{ieeeconf}", "[journal]{IEEEtran}")
      .replace(&as_written, &author_block("", tail))
  };
  // (a mark the shared reader sees and a literal test did not: a `\let` alias of `\textsuperscript`, the 86d r2
  // re-review's B1; the base's reading by the marks, the grade kept)
  let aliased = tex
    .replace(
      &as_written,
      &author_block(
        "\\up{1}",
        ".\\thanks{\\up{1}All are with the University of Texas at Austin.}",
      ),
    )
    .replace(
      "\\IEEEoverridecommandlockouts",
      "\\IEEEoverridecommandlockouts\n\\let\\up\\textsuperscript",
    );
  // (one name marked by a symbol and no note leading with a mark: the block is left as the base reads it, its notes
  // the document's, none on the last author (the B2 shape); the stop still makes the last line an affiliation there.
  // That "and Farshid Alambeigi ." line is the base's residual, pinned on purpose: a shared-reader fix that ties a
  // note to a non-last author's mark (the observer's S2c item) flips it, and must update this guard)
  let partly_block = "Susheela Sharma$^{*}$, Sarah Go,\\\\ and Farshid Alambeigi \\IEEEmembership{Member, IEEE}.\\thanks{$^{*}$Corresponding author.}";
  let partly = tex.replace(
    &as_written,
    &format!("{partly_block}\\thanks{{Manuscript received May 1.}}}}"),
  );
  // (names marked by a symbol alone and a note leading with a number, the 86d r3 re-review's a9: no mark on the names
  // reads as a label, so the block is not both-marked and is left as the base reads it; read as both-marked, the
  // correspondence note went to the last author and the numbered one became a creator with no name; the same pinned
  // residual)
  let symbols = tex.replace(
    &as_written,
    &format!("{partly_block}\\thanks{{$^{{1}}$All are with the University of Texas at Austin.}}}}"),
  );
  // (2405.17600's own shape, the 86d r4 re-review's S1: the first name's mark, `$^{*1}$`, leads with a symbol, so only
  // a later name's `$^{1}$` makes the names labelled; a test of the first mark alone reads the block as the base does
  // and drops the grade. The base plus the grade: the `$^{*1}$` name holds no affiliation, as on the base and in the
  // witness, a shared-reader residual pinned on purpose)
  let star_first = tex.replace(
    "Susheela Sharma$^{1}$, Sarah Go$^{1}$",
    "Susheela Sharma$^{*1}$, Sarah Go$^{1}$",
  );
  let star_first_creators: &[&[&str]] = &[&["Susheela Sharma"], &["Sarah Go", thanks], &[
    "Farshid Alambeigi",
    "membership=Member, IEEE",
    thanks,
  ]];
  let stop_line = "affiliation=and Farshid Alambeigi .";
  let partly_creators: &[&[&str]] = &[&["Susheela Sharma *", stop_line], &["Sarah Go", stop_line]];
  let variants: [Variant; 11] = [
    (tex.to_string(), marked, &[]),
    (aliased, marked, &[]),
    (star_first, star_first_creators, &[]),
    (partly, partly_creators, &[
      "* Corresponding author.",
      "Manuscript received May 1.",
    ]),
    (symbols, partly_creators, &[
      "* Corresponding author.",
      "1 All are with the University of Texas at Austin.",
    ]),
    (
      tex.replace(&as_written, &author_block("$^{1}$", &format!(".{note}"))),
      marked,
      &[],
    ),
    (journal("."), plain, &[]),
    (journal(";"), plain, &[]),
    // what prints no text between the grade and the stop, or after the stop (2112.14209, 2510.15773)
    (
      journal(&format!(
        "\n\n\\vspace{{-10mm}}\n{sharma}\n{alambeigi}.\n\n"
      )),
      plain,
      both,
    ),
    (
      journal(&format!(".\n{sharma} \\\\\n{alambeigi}")),
      plain,
      both,
    ),
    (
      journal(&format!(".\n{compsoc}"))
        .replace("[journal]{IEEEtran}", "[journal,compsoc]{IEEEtran}"),
      plain,
      &both[..1],
    ),
  ];
  for (tex, expected, pubnotes) in variants {
    let (log, xml) = convert_with(&tex, Some(PRELOAD));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    assert_eq!(creators(&xml), expected, "{tex}\n{xml}");
    // (`creators` reads no `<note>`: every note's place, an author's or the document's)
    let expected_notes: Vec<(String, String)> = pubnotes
      .iter()
      .map(|note| ("pubnote".to_string(), note.to_string()))
      .collect();
    assert_eq!(note_places(&xml), expected_notes, "{tex}\n{xml}");
  }
}

/// 2405.10825: in an `\IEEEauthorblockN`, the stop after the last grade no longer makes the last line no names (its
/// authors had become every author's affiliation) nor, once the line continues the names, keeps the block's cautious
/// split from parting them (one creator holding seven names, an Error).
#[test]
fn ieee_block_names_kept_after_closing_stop() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_ieee_blockn_names_after_closing_stop.tex"
    ),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let fellow = "membership=Fellow, IEEE";
  let expected: &[&[&str]] = &[
    &["Hao Zhou"],
    &["Chengming Hu"],
    &["Can Chen"],
    &["Xue Liu", fellow],
    &["Charlie Zhang", fellow],
    &["Xianbin Wang", fellow],
    &["Jiangchuan Liu", fellow],
  ];
  assert_eq!(creators(&xml), expected, "{xml}");
}

/// A variant of the repro: its source, every creator ([`creators`]) and the document's notes, in order.
type Variant<'a> = (String, &'a [&'a [&'a str]], &'a [&'a str]);

/// The repro's author block with each author's `mark` and `tail` after the grade.
fn author_block(mark: &str, tail: &str) -> String {
  format!(
    "Susheela Sharma{mark}, Sarah Go{mark},\\\\ and Farshid Alambeigi{mark} \\IEEEmembership{{Member, IEEE}}{tail}}}"
  )
}

/// Every author's note and document note in `xml`, in order: the owner's name (or "pubnote") and the note's text.
fn note_places(xml: &str) -> Vec<(String, String)> {
  let mut out = Vec::new();
  for (start, _) in xml.match_indices("<creator ") {
    let creator = &xml[start..start + xml[start..].find("</creator>").expect("closed creator")];
    let name = creator.find("<personname>").map_or(String::new(), |at| {
      text(
        &creator[at..at
          + creator[at..]
            .find("</personname>")
            .expect("closed personname")],
      )
    });
    for (at, _) in creator.match_indices("<note ") {
      let close = at + creator[at..].find("</note>").expect("closed note");
      out.push((name.clone(), text(&creator[at..close])));
    }
  }
  for (at, _) in xml.match_indices("<pubnote ") {
    let close = at + xml[at..].find("</pubnote>").expect("closed pubnote");
    out.push(("pubnote".to_string(), text(&xml[at..close])));
  }
  out
}
