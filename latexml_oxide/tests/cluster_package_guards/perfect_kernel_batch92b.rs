//! Guards for batch 92b (frontmatter refactor stage 7, the llncs and aa move onto the observer — the stage-6 gate
//! study's resolver pieces 3-6 —, and stage 4, label keys by their printed text; `docs/parity/
//! FRONTMATTER_DIGESTED_REFACTOR.md`, OXIDIZED_DESIGN_DIVERGENCES #837, #917-#920). Each guard converts its repros
//! under the shadow switch in process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::
//! assert_reading`), and asserts the shadow reading's tuples whole, the XML's creators whole where today's output is
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
  assert_reading(
    repro!("llncs_inner_email_lines_are_each_written_out.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email ann.able@x.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email bob.baker@x.org | api:labelseq api:inner-contact ruling3:written-out owner:spelled",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
      "B email office@x.org | api:labelseq api:inner-contact ruling3:written-out owner:block",
    ],
  );
  // (FIDELITY_CHARTER §4's corresponding step takes only the ONE address left on the block, once over every address
  // the post-pass placed (`Resolver::place_emails_left`, shared with the sole institute's lines): two left are each
  // the block's, though the corresponding Ann Able carries both entries — review rev92b_r1 B2, where both were hers)
  assert_reading(
    repro!("llncs_inner_emails_left_are_each_the_blocks.tex"),
    None,
    &[
      "C Ann Able [1,2] | line:names",
      "  a Univ X | api:labelseq",
      "  a Univ Y | api:labelseq",
      "  n Corresponding author. | note:adjacent",
      "C Bob Baker [1,2] | line:names",
      "  a Univ X | api:labelseq",
      "  a Univ Y | api:labelseq",
      "B email info@x.org | api:labelseq api:inner-contact owner:inner-email owner:block",
      "B email office@y.org | api:labelseq api:inner-contact owner:inner-email owner:block",
    ],
  );
  // (a template is every author's address and no one's own: the block's, never the corresponding author's —
  // 2408.14427's `\{first.last\}@uii-ai.com`, which went to Meng Zheng)
  assert_reading(
    repro!("llncs_inner_pattern_email_is_the_blocks.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  n Corresponding author. | note:adjacent",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Cid Cole [2] | line:names",
      "  a Univ Y | api:labelseq",
      "  c email cid.cole@y.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "B email {first.last}@x.org | api:labelseq api:inner-contact owner:inner-email owner:pattern",
    ],
  );
}

/// Piece 2, 85b's `_share` port (llncs class S, 19 papers: 1912.11606, 2302.05745; OD #901): llncs's one `\institute`,
/// no `\and` in it (`\lx@llncs@institute@sole`, `_share=authors`), is printed unnumbered under the whole row of names,
/// every author's, unless the class's `\inst{n}` picks it; the shadow read its number 1, which no name requested, and
/// left it and its addresses on the block. Now every live author's (`api:share:authors`), its e-mail lines placed by
/// FIDELITY_CHARTER §4's chain as today's places them: the author it spells; the entry's addresses one per author in
/// their order (`{u1, u2, u3}@x.org`); the one address left of several authors the corresponding author's; else the
/// block. A mark that is no class request of a number picks nothing — a hand-typed `$^{*}$` or `$^{1}$` (2302.05745,
/// 2607.02596), `\inst{*}` (2207.04574) —; the class's `\inst{1}` picks it (2608.11332, the reading of 92a). Today's XML
/// agrees, and is pinned.
#[test]
fn a_sole_institute_no_inst_picks_is_every_authors() {
  assert_reading(
    repro!("llncs_sole_institute_is_every_authors_in_order.tex"),
    Some(&[
      &["Ann Able ∗", "affiliation=Univ X", "email=u1@x.org"],
      &["Bob Baker ∗", "affiliation=Univ X", "email=u2@x.org"],
      &["Cid Cole", "affiliation=Univ X", "email=u3@x.org"],
    ]),
    &[
      "C Ann Able [*] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email u1@x.org | api:labelseq api:inner-contact ruling3:written-out api:share:authors owner:order",
      "C Bob Baker [*] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email u2@x.org | api:labelseq api:inner-contact ruling3:written-out api:share:authors owner:order",
      "C Cid Cole [] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email u3@x.org | api:labelseq api:inner-contact ruling3:written-out api:share:authors owner:order",
    ],
  );
  assert_reading(
    repro!("llncs_sole_institute_emails_by_the_chain.tex"),
    Some(&[
      &["Ann Able *", "affiliation=Univ X", "email=ann.able@x.org"],
      &["Bob Baker", "affiliation=Univ X", "email=lab@x.org"],
      &["Cid Cole", "affiliation=Univ X"],
    ]),
    &[
      "C Ann Able [*] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email ann.able@x.org | api:labelseq api:inner-contact api:share:authors owner:spelled",
      "C Bob Baker [] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "  c email lab@x.org | api:labelseq api:inner-contact api:share:authors owner:corresponding",
      "  n Corresponding author. | note:adjacent",
      "C Cid Cole [] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
    ],
  );
  assert_reading(
    repro!("llncs_sole_institute_shared_address_is_the_blocks.tex"),
    Some(&[
      &["Ann Able 1", "affiliation=Univ X"],
      &["Bob Baker 1", "affiliation=Univ X"],
      &["email=info@x.org"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "C Bob Baker [1] | line:names",
      "  a Univ X | api:labelseq api:share:authors",
      "B email info@x.org | api:labelseq api:inner-contact api:share:authors owner:block",
    ],
  );
  assert_reading(
    repro!("llncs_sole_institute_picked_by_inst.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X", "email=ann.able@x.org"],
      &["Bob Baker"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email ann.able@x.org | api:labelseq api:inner-contact owner:inner-email owner:spelled",
      "C Bob Baker [] | line:names",
    ],
  );
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
      "  a Univ X | api:leading-mark api:split-by-marks",
      "C Bob Baker [2] | line:names",
      "  a Univ Y, Town | api:leading-mark api:split-by-marks",
      "C Cid Cole [1,2] | line:names",
      "  a Univ X | api:leading-mark api:split-by-marks",
      "  a Univ Y, Town | api:leading-mark api:split-by-marks",
    ],
  );
  // (a closing line of contacts alone continues the entry above it — 1507.05963's `\\ \email{cessen@phys.au.dk}`,
  // which held the split back, so the whole institute went to the names marked 1, tagged; the address its speller's)
  assert_reading(
    repro!("aa_institute_lines_led_by_marks_email_line_continues.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:leading-mark api:split-by-marks",
      "  c email ann.able@y.org | api:leading-mark api:split-by-marks api:inner-contact owner:inner-email \
       owner:spelled",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:leading-mark api:split-by-marks",
      "C Cid Cole [1,2] | line:names",
      "  a Univ X | api:leading-mark api:split-by-marks",
      "  a Univ Y | api:leading-mark api:split-by-marks",
    ],
  );
  // (a closing line of text holds the split back: the entry holds marks of its own no split read, so its number is no
  // reading of the names' hand marks — untagged, an honest disagreement; astro-ph0405186, 2208.05114)
  assert_reading(
    repro!("aa_institute_lines_led_by_marks_text_line_is_no_split.tex"),
    None,
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X Univ Y Contact: X Office | api:labelseq",
      "C Bob Baker [2] | line:names",
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
  assert_reading(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_dag_and_dagger_are_one_label.tex"
    ),
    Some(&[
      &["Ann Able", "affiliation=Univ A"],
      &["Cid Cole ⋄", "affiliation=Univ C"],
      &["Bob Baker †", "affiliation=Univ A"],
    ]),
    &[
      "C Ann Able [†] | line:names",
      "  a Univ A | ruling2:symbol-line",
      "C Cid Cole [⋄] | line:names sep:comma",
      "  a Univ C | ruling1:request",
      "C Bob Baker [†] | line:names sep:comma",
      "  a Univ A | ruling2:symbol-line",
    ],
  );
  // (a symbol no entry answers stays shown on its name, by its printed glyph: `\inst{1,\star}`, 2105.03212 — the ⋆ was
  // lost once its label was the printed `⋆`, which `insert::unanswered_mark_text` read only by spelling)
  assert_reading(
    repro!("aa_inst_unanswered_symbol_stays_shown.tex"),
    Some(&[&["Ann Able ⋆", "affiliation=Univ X"], &[
      "Bob Baker",
      "affiliation=Univ Y",
    ]]),
    &[
      "C Ann Able [1,⋆] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:labelseq",
    ],
  );
}

/// Piece 6, each binding's `\author` is its class's (review rev92a_r5; user 2026-10-30; OXIDIZED_DESIGN_DIVERGENCES
/// #53, #920), where the binding owns the call: ptephy's binding serves ptephy.cls, whose `\author` replaces
/// (ptephy.cls:427, 1412.6580: only "Bob Baker" is printed), and ptephy_v1, whose authblk-TI stacks (64qa's
/// `ptephy_v1_authblk_interleaved`); it had both stack through inst_support's `\author`. Univ A, which no printed name
/// requests, is the block's. (inst_support's own `\author` keeps stacking: restoring Perl's per-call clear changed 162
/// of 1,305 OmniBus papers, most of them stacking classes — an open question for the main session, OD #920.)
#[test]
fn each_class_author_replaces_or_stacks_as_its_class() {
  assert_reading(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ptephy_author_given_twice_keeps_the_last.tex"
    ),
    Some(&[&["Bob Baker", "affiliation=Univ B"], &[
      "affiliation=Univ A",
    ]]),
    &[
      "C Bob Baker [affiliation:2] | line:names",
      "  a Univ B | api:leading-mark",
    ],
  );
  // (ptephy_v2 loads authblk-TI as ptephy_v1 does (ptephy_v2.cls:485): its calls stack — 2511.03250's six
  // `\author[1]{R.~Miyata}` calls, of which the binding's `\@currname` test kept the last (review rev92b_r1 S1). The
  // shadow does not read authblk-TI's `\author[n]` labels (ptephy is not a written family): its institutes the block's.)
  assert_reading(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ptephy_v2_author_calls_stack.tex"
    ),
    Some(&[&["Ann Able", "affiliation=Univ A"], &[
      "Bob Baker",
      "affiliation=Univ B",
    ]]),
    &[
      "C Ann Able [] | line:names",
      "C Bob Baker [] | line:names",
      "B affiliation Univ A | api:leading-mark label:unrequested owner:block",
      "B affiliation Univ B | api:leading-mark label:unrequested owner:block",
    ],
  );
}

/// Piece 5's spacing, review rev92b_r1 B1 (2403.08629, 2601.02539, 1704.04058; FIDELITY_CHARTER §4): in a mark's
/// printed text any spacing parts its labels, as a comma does — white space, and every item Perl marks `isSpace`
/// (`util_text::is_spacing_item`, the reader `first_ink` uses: `\,` `\!` `\:` `\ ` `\hspace` `\kern` `\mskip`, in a
/// formula and in `\textsuperscript`) —, and an invisible character (a zero-width space, a format or combining
/// character: `util_text::is_invisible_char`) is no label and no shown symbol: the kernel's label keys
/// (`queue_digest::label_ink`) and the observer's mark reader (`events::script_ink`) alike. `$^{1\,2}$` printed one
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
  assert_reading(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_mark_zero_width_space_is_no_label.tex"
    ),
    Some(&[
      &["Ann Able", "affiliation=Univ One", "affiliation=Univ Two"],
      &["Bob Baker *", "affiliation=Univ Two"],
    ]),
    &[
      "C Ann Able [1,2] | line:names",
      "  a Univ One | ruling1:request",
      "  a Univ Two | ruling1:request",
      "C Bob Baker [*,2] | line:names sep:column",
      "  a Univ Two | ruling1:request",
    ],
  );
  // (a picture prints no label (`util_text::is_picture`): an SVG path's data is no text of the mark — 2605.02769's TikZ
  // ORCID logo in `\inst{\ref{leiden}\orcidicon{…}}`, parted at its spaces, was the marks M, C, 0, L, Z on every
  // name; the shadow read it as the requests `M`, `0`, `L`, … already)
  assert_reading(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_mark_picture_is_no_label.tex"
    ),
    Some(&[&["Ann Able", "affiliation=Univ One"], &[
      "Bob Baker",
      "affiliation=Univ Two",
    ]]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ One | ruling1:request",
      "C Bob Baker [2] | line:names sep:column",
      "  a Univ Two | ruling1:request",
    ],
  );
}
