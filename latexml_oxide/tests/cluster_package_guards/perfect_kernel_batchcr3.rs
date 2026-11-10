//! Red/green guards for batch CR3: content-residual items — a recovered call's run of several creators placed without a
//! joined label, the shipped imsart copies read through their binding, an imsart `\address` after `\end{frontmatter}`
//! printed in place, and a date set inside an `\author` nested in `\title` kept. OXIDIZED_DESIGN_DIVERGENCES #985, #986.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

/// CR3 item 2 (review revCR2b finding 1 and P1): REVTeX 3's minipage `\address` (abstract and `\pacs` only) after
/// `\author{Ann Able and Bob Baker}`, then `\address{Univ A, Budapest}`. The recovered call's run is two creators, so it
/// goes out unlabelled (no `label=author:1,author:2` the insertion cannot find: no orphaned-annotation Warning), and its
/// empty entry records the run, so Univ A is both authors', as the old driver. Distinct from cr2's
/// `revtex3_minipage_address_keeps_contact_order` (one shared item label). Repro
/// sectioning-frontmatter/revtex3_minipage_address_first_joins_run_labels.
#[test]
fn revtex3_minipage_address_first_one_run() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_minipage_address_first_joins_run_labels.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ A, Budapest</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ A, Budapest</contact>\n  </creator>",
  ]);
  assert_eq!(elements(&xml, "abstract"), vec![
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\">\n    <p xml:id=\"abstract1.1\">We study a zebraword model.</p>\n  </abstract>",
  ]);
  assert_eq!(elements(&xml, "classification"), vec![
    "<classification name=\"PACS:\u{a0}\" scheme=\"pacs\">PACS numbers: 05.40.+j</classification>",
  ]);
}

/// CR3 item 3 (CR1 class C): a shipped style declaring itself `\ProvidesFile{imsart.sty}` under another name is routed by
/// its declaration to the imsart.sty binding and read after it as the stand-in's copy, so `\ead` inside `\author` is the
/// author's e-mail, `\address[A]` the authors' marked A, `\affiliation` the run's (1905.11425, 2101.10506 stylefile.sty;
/// math0608476 imsart_mod.sty; 1301.1466 imsart.sty under its own name). Distinct: the declaration route to a binding
/// that had none for the `.sty`, and the binding's author markup (`\fnms`/`\snm`) where no lock records a refused
/// `\author`. Repro sectioning-frontmatter/imsart_renamed_sty_copy_reads_through_its_binding.
#[test]
fn imsart_renamed_sty_copy_reads_through_its_binding() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/imsart_renamed_sty_copy_reads_through_its_binding.tex"
  );
  assert_eq!(elements(&convert_clean(tex), "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann@x.edu</text></contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ A, </contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Zebra Institute</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">bob@y.org</text></contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ B, </contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Zebra Institute</contact>\n  </creator>",
  ]);
}

/// CR3b item 1 (review revCR3 finding 1): imsart.sty's unlabelled `\address` after `\end{frontmatter}` (the `aap` option's
/// addresses at the end, 1301.1466) is no author's: each block prints where it is called, as imsart.sty's
/// `\printaddressnum` sets it, and no author takes it as the API's default owner (the last author took all three).
/// Distinct from `imsart_renamed_sty_copy_reads_through_its_binding` (labelled `\address[A]` inside `{aug}`, a contact):
/// the binding's own branch on the frontmatter's close. Each block's `\printead{eN}` row prints the author's e-mail as
/// imsart.sty's `\@@printead` does, "e-mail: …" (CR4, review revCR3b finding 1; the row was empty, the binding's
/// `\printead` printing nothing; the stand-in's own prints "(e1)", the locked binding replacing it). Repro
/// sectioning-frontmatter/imsart_address_after_frontmatter_prints_in_place.
#[test]
fn imsart_address_after_frontmatter_prints_in_place() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/imsart_address_after_frontmatter_prints_in_place.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann@x.edu</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">bob@y.org</text></contact>\n  </creator>",
  ]);
  assert_eq!(elements(&xml, "td"), vec![
    "<td align=\"left\" class=\"ltx_nopad_r\" xml:id=\"p3.1.1.1\">Ann Able</td>",
    "<td align=\"left\" class=\"ltx_nopad_r\" xml:id=\"p3.1.2.1\">Univ A</td>",
    "<td align=\"left\" class=\"ltx_nopad_r\" xml:id=\"p3.1.3.1\">e-mail: <text font=\"typewriter\" xml:id=\"p3.1.3.1.1\">ann@x.edu</text></td>",
    "<td align=\"left\" class=\"ltx_nopad_r\" xml:id=\"p4.1.1.1\">Bob Baker</td>",
    "<td align=\"left\" class=\"ltx_nopad_r\" xml:id=\"p4.1.2.1\">Univ B</td>",
    "<td align=\"left\" class=\"ltx_nopad_r\" xml:id=\"p4.1.3.1\">e-mail: <text font=\"typewriter\" xml:id=\"p4.1.3.1.1\">bob@y.org</text></td>",
    "<td align=\"left\" class=\"ltx_nopad_r\" xml:id=\"p5.1.1.1\">Cid Cole</td>",
    "<td align=\"left\" class=\"ltx_nopad_r\" xml:id=\"p5.1.2.1\">Univ C</td>",
  ]);
}

/// CR3b item 2 (CR2b item 5): `\date` inside an `\author` nested in `\title` (1411.6693) runs while the `\maketitle` flush
/// digests the title: the late block's rehearsal notes the `@now` entry as a queued command, and the emitter queues it at
/// the block's flush, so the date is kept, with no Warning (it was lost with none). Distinct from cr2's
/// `date_inside_author_is_the_date` (a top-level `\author`, its `\date` queued): the flush-time `@now` path. Repro
/// sectioning-frontmatter/date_inside_author_inside_title_is_the_date.
#[test]
fn date_inside_author_inside_title_is_the_date() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/date_inside_author_inside_title_is_the_date.tex"
  );
  let xml = convert_clean(tex);
  assert_eq!(elements(&xml, "date"), vec![
    "<date role=\"creation\"><sup xml:id=\"id3\">1</sup> School of Zebrawords, Northeast Normal University, Changchun 130024, China</date>",
  ]);
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able<sup xml:id=\"id2\">1</sup></personname>\n  </creator>",
  ]);
}
