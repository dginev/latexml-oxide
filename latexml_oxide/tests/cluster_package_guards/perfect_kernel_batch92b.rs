//! Guards for batch 92b (frontmatter refactor stage 7, the llncs and aa move onto the observer — the stage-6 gate
//! study's resolver pieces 3-6 —, and stage 4, label keys by their printed text; `docs/archive/
//! FRONTMATTER_DIGESTED_REFACTOR_2026-11-07.md`, OXIDIZED_DESIGN_DIVERGENCES #837, #917-#920). Each guard converts its repros
//! in process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::
//! assert_reading`), and asserts the reading's tuples whole, the XML's creators whole where today's output is
//! right (else only the same in every mode), and no Error or Warning. Every repro compiles clean with pdflatex (TL 2025;
//! the aa repros with 1707.06238's aa.cls, which TeX Live lacks) and is checked against Perl 0.8.8.

use super::perfect_kernel_batch92a::assert_reading;

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// Piece 1, the gate study's G3 (llncs class G, 27 papers: 1606.03774, 2204.03161; aa 0901.4618, 1401.5494): an
/// `\email` inside a numbered `\institute` entry printing one address is not the entry's label's — llncs and aa print
/// the list once under the names, the address under its institute, and today's (Perl too) copies it onto every name
/// carrying the label. FIDELITY_CHARTER §4's chain places it (`owner:inner-email`, a change tag): the author it spells,
/// any live author (`bob.baker@y.org` under institute 2 is Bob Baker's of institute 1); else the entry's only carrier
/// (`info@y.org`, Bob Baker's); else the carrier marked the corresponding author (`lab@x.org`, Bob Baker's "Corresponding
/// author." note); else the block, once (a group's `info@x.org` under the institute Ann Able and Bob Baker carry). The
/// sibling (2608.29307): a `\\` inside the call prints its addresses one a line — the rehearsal's break was read into
/// the address (`…@x.org\lx@fm@obs@breakbob.baker@…`, one address spelling its first author); now each is written out.
/// Today's XML copies each one-address call onto every carrier, so only the reading is pinned.
#[test]
fn an_entry_email_printing_one_address_is_placed_by_the_chain() {
  assert_reading(repro!("llncs_inner_email_is_its_spellers.tex"), None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | api:labelseq",
    "  c email ann.able@x.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
    "C Bob Baker [2] | line:names",
    "  a Univ Y | api:labelseq",
    "  c email info@y.org | api:labelseq api:inner-contact owner:inner-email owner:only-carrier",
    "C Cid Cole [1] | line:names",
    "  a Univ X | api:labelseq",
  ]);
  assert_reading(
    repro!("llncs_inner_shared_email_is_the_blocks.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email bob.baker@y.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
      "  c email cid@y.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "B email info@x.org | api:labelseq api:inner-contact owner:inner-email owner:block",
    ],
  );
  assert_reading(
    repro!("llncs_inner_email_is_the_corresponding_carriers.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email lab@x.org | api:labelseq api:inner-contact owner:inner-email owner:corresponding",
      "  n Corresponding author. | note:adjacent",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
    ],
  );
  assert_reading(repro!("aa_inner_email_is_its_spellers.tex"), None, &[
    "C Ann Able [1] | line:names",
    "  a Univ X | api:labelseq",
    "C Bob Baker [1] | line:names",
    "  a Univ X | api:labelseq",
    "  c email bob.baker@x.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
    "C Cid Cole [2] | line:names",
    "  a Univ Y | api:labelseq",
  ]);
}

/// Piece 3, labelseq owners by printed marks (aa class C, 13 papers: 1009.6047, 0904.2180; llncs class H: 2207.06953):
/// a numbered entry's owner found by a mark printed by hand (`$^{1}$`, `\textsuperscript{2}`), read off the digested
/// list, not by the class's own `\inst` (`Resolver::class_requests`), is ruling 1's reading — the marks shown on a name
/// are its requests —, tagged `ruling1:request`: today's (and Perl's) XML keeps the marks in the names and the
/// institutes on a nameless creator (Perl on two names by place), so the reading alone is pinned. The class's own
/// `\inst{1}` stays untagged (`a_sole_institute_no_inst_picks_is_every_authors`'s last row, the 92a guards).
#[test]
fn a_numbered_entry_owned_by_a_hand_mark_is_ruling_ones() {
  assert_reading(
    repro!("aa_hand_marks_request_numbered_institutes.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq ruling1:request",
      "C Bob Baker [1,2] | line:names",
      "  a Univ X | api:labelseq ruling1:request",
      "  a Univ Y | api:labelseq ruling1:request",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq ruling1:request",
    ],
  );
}

/// Piece 4, the mark-led split of a numbered entry with no `\and` (aa class D, 5 papers: 1003.3020; aa's 1206.6282 and
/// astro-ph0308041 split so too; OD #918 — 0904.2180 is class C's, the gate table's 1312.5084 no such shape): one
/// `\institute{\inst{1} Univ X \\ \inst{2} Univ Y, Town}` whose every line a mark leads (`resolver::line_led_pieces`,
/// at the start or after a Break event) is each line its own entry, labelled by its mark — not the whole by the class's
/// number 1, which gave it to the `\inst{1}` names (today's XML: one affiliation on a nameless creator; Perl none, an
/// orphaned annotation Warning). The reading alone is pinned.
#[test]
fn a_numbered_entry_whose_lines_marks_lead_is_split_at_them() {
  assert_reading(
    repro!("aa_institute_lines_led_by_inst_are_each_an_entry.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:leading-mark",
      "C Bob Baker [2] | line:names",
      "  a Univ Y, Town | api:leading-mark",
      "C Cid Cole [1,2] | line:names",
      "  a Univ X | api:leading-mark",
      "  a Univ Y, Town | api:leading-mark",
    ],
  );
  // (a closing line of contacts alone continues the entry above it — 1507.05963's `\\ \email{cessen@phys.au.dk}`,
  // which held the split back, so the whole institute went to the names marked 1, tagged; the address its speller's —
  // since 99a the frontmatter API's Cut parts the call and labels the address for its speller, `api:label`, OD #954)
  assert_reading(
    repro!("aa_institute_lines_led_by_marks_email_line_continues.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:leading-mark",
      "  c email ann.able@y.org | api:label",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:leading-mark",
      "C Cid Cole [1,2] | line:names",
      "  a Univ X | api:leading-mark",
      "  a Univ Y | api:leading-mark",
    ],
  );
  // (a closing line of text holds the split back: the entry holds marks of its own no split read, so its number is no
  // reading of the names' hand marks; astro-ph0405186, 2208.05114 — and aa's sole institute is `_share=authors` since
  // 92c r2 (OD #932: aa.cls prints it unnumbered, only `\inst{n}` picks it), so the hand-marked names leave it the
  // block's, as today's XML; it was the first name's, untagged, an honest disagreement)
  // — since 99a the frontmatter API's Cut parts the call at its marks before either engine reads it (OD #954), so each
  // name takes its entry, the closing line continuing the entry above it as an unmarked line does in a marked list
  // (`affiliation_calls`, 2609.19448)
  assert_reading(
    repro!("aa_institute_lines_led_by_marks_text_line_is_no_split.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:leading-mark",
      "C Bob Baker [2] | line:names",
      "  a Univ Y Contact: X Office | api:leading-mark",
    ],
  );
}

/// Piece 5, refactor stage 4 (FIDELITY_CHARTER §4, 2026-10-29: frontmatter label keys are compared by their printed
/// text; OXIDIZED_DESIGN_DIVERGENCES #919): a mark's operand that prints through a command is keyed by the text it
/// prints, read off its digestion in a speculative run (`queue_digest::printed_label`), on the request side (`\inst`,
/// a name's `$^{…}$`, a symbol mark) and the set side (an affiliation's leading mark) alike — aa's `\inst{\ESO}` with
/// `\newcommand{\ESO}{2}` is entry 2's (1307.8124's `\inst{\MPIA}`: the label `affiliation:\ESO` matched no entry);
/// `\dag` and `\dagger` are one label (†). Today's output changes, and is pinned.
#[test]
fn a_mark_is_keyed_by_its_printed_text() {
  assert_reading(
    repro!("aa_inst_macro_is_its_printed_number.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=ESO"],
      &["Cid Cole", "affiliation=Univ X", "affiliation=ESO"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [2] | line:names",
      "  a ESO | api:labelseq",
      "C Cid Cole [1,2] | line:names",
      "  a Univ X | api:labelseq",
      "  a ESO | api:labelseq",
    ],
  );
  // (a number a macro prints with a symbol glued to it is each mark apart, `1∗` the 1 and the ∗: `$^{\ICQD,\Phys\ast}$`,
  // 2412.09857 — read whole, Ann Able lost Univ A)
  assert_reading(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_macro_mark_glued_to_a_symbol.tex"
    ),
    Some(&[&["Ann Able ∗", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ A",
    ]]),
    &[
      "C Ann Able [1,*] | line:names",
      "  a Univ A | ruling1:request ruling1:parted",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ A | ruling1:request",
    ],
  );
}

/// Piece 5's spacing, review rev92b_r1 B1 (2403.08629, 2601.02539, 1704.04058; FIDELITY_CHARTER §4): in a mark's
/// printed text any spacing parts its labels, as a comma does — white space, and every item Perl marks `isSpace`
/// (`util_text::is_spacing_item`, the reader `first_ink` uses: `\,` `\!` `\:` `\ ` `\hspace` `\kern` `\mskip`, in a
/// formula and in `\textsuperscript`) — but what prints no gap (`util_text::is_blank_item`: `\-`'s empty box, the
/// italic correction `\/`, review rev92b_r3 P1: `1\-2` is the label 12) —, and a format character (a zero-width space: `util_text::keeps_label_char`) is
/// no label and no shown symbol: the kernel's label keys (`queue_digest::label_ink`) and the observer's mark reader
/// (`events::script_ink`) alike. A combining mark is part of the letter it is on (`\={x}` is `x̄`, review rev92b_r2
/// B1), and a pgf/TikZ picture prints its text nodes only (`util_text::picture_text`: a circled `a` is `a`, review
/// rev92b_r2 B2). `$^{1\,2}$` printed one
/// label `1_2`, which nothing answered and no name showed (each mark lost; Nan Jiang's `$^{1,2\,*}$` lost BIGAI and the
/// ∗); `\up{\!1,2}` (2209.14855) showed an empty superscript holding U+200B on each name. Today's output changes, and
/// is pinned.
#[test]
fn spacing_parts_a_marks_labels() {
  assert_reading(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_mark_labels_parted_by_spacing.tex"
    ),
    Some(&[
      &["Ann Able 1,2"],
      &["Bob Baker ∗,†"],
      &["Cid Cole 1,†"],
      &["Dee Dunn a,b"],
      &["Eve Esse 1,2"],
      &["Fay Fox 1,2"],
      &["Gus Gray 1,2"],
      &["Hal Hill 1,2"],
      &["Ida Ide 1,2"],
      &["Jo Jay 12"],
      &["Kit Kay 12"],
      &["Lee Lin 12"],
      &["Max Moe 12"],
      &["Ned Nye 12"],
    ]),
    &[
      "C Ann Able [1,2] | line:names",
      "C Bob Baker [*,†] | line:names sep:column",
      "C Cid Cole [1,†] | line:names sep:column",
      "C Dee Dunn [a,b] | line:names sep:column",
      "C Eve Esse [1,2] | line:names sep:column",
      "C Fay Fox [1,2] | line:names sep:column",
      "C Gus Gray [1,2] | line:names sep:column",
      "C Hal Hill [1,2] | line:names sep:column",
      "C Ida Ide [1,2] | line:names sep:column",
      "C Jo Jay [12] | line:names sep:column",
      "C Kit Kay [12] | line:names sep:column",
      "C Lee Lin [12] | line:names sep:column",
      "C Max Moe [12] | line:names sep:column",
      "C Ned Nye [12] | line:names sep:column",
    ],
  );
  assert_reading(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_mark_spaced_labels_answer_their_lines.tex"
    ),
    Some(&[
      &["Ann Able ∗", "affiliation=Univ One", "affiliation=Univ Two"],
      &["Bob Baker", "affiliation=Univ Two"],
    ]),
    &[
      "C Ann Able [1,2,*] | line:names",
      "  a Univ One | ruling1:request",
      "  a Univ Two | ruling1:request",
      "C Bob Baker [2] | line:names sep:column",
      "  a Univ Two | ruling1:request",
    ],
  );
}
