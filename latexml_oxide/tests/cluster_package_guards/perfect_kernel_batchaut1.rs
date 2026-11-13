//! Guards for AUT1 (OXIDIZED_DESIGN_DIVERGENCES #993 "AUT1"): a document macro of no parameters holding several names
//! (`\author{\Authors}`, `\author{\authorBlock}`) is read with its expansion in its place (`reading::unfold_held_names`),
//! the rehearsal having recorded it (core `FieldInvocation::Expanded`), so a held block reads as its inline twin: each
//! name, mark, note and contact written from its own tokens, its markup kept, no by-text Warning. One row per path: the
//! block reader's comma and word cuts with formula marks and a font (the Frontiers shape, 2103.13804), a note's argument
//! and the fonts in it, the gap-parted lines of a CVPR `\authorBlock` (2504.00496), nested macros unfolded in turn, a
//! contact line, a macro printing two names, a note and their marks (`cl9`'s shape, cond-mat9310042, 1602.03850), and a
//! block keeping its marks whose mark past a gap or a comma no written run prints, shown as a request (2209.05831).
//! Every creator whole; exact Error and Warning counts. The API route (acmart's call, OmniBus's and llncs's blocks) is
//! guarded by its own rows in `perfect_kernel_batchoc1` and `perfect_kernel_batchoc1c`.
//! GRP1's rows: a declaration the emitter replays is cut from the tokens the block was read from, as far as its call read.

use std::borrow::Cow;

use super::perfect_kernel_batchhg1::{Creators, Row, assert_rows};

/// A repro of the catalog (`tools/perfect_kernel/repros/<path>.tex`), its name the row's.
macro_rules! repro {
  ($path:literal) => {
    (
      $path,
      Cow::Borrowed(include_str!(concat!(
        "../../../tools/perfect_kernel/repros/",
        $path,
        ".tex"
      ))),
    )
  };
}

/// A held block's creators are its inline twin's, each from its own tokens.
#[test]
fn a_macro_of_names_is_read_as_its_inline_twin() {
  assert_rows(&[
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_macro_of_names_is_read_unfolded"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_macro_of_marked_names_keeps_markup"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup>1,∗</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob <text font="bold">Baker</text><sup>2</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Cole<sup>1</sup></personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_macro_of_names_with_thanks_keeps_markup"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">Univ A, <text font="typewriter">ann@a.edu</text></note></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob <emph font="italic">Baker</emph></personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_block_macro_of_lines_keeps_markup"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Jingbo Lu</personname><contact name="Affiliation: " role="affiliation"> University of Electronic Science</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Mu Li</personname><contact name="Affiliation: " role="affiliation"> Harbin Institute of Technology</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Shuhang Gu</personname><note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks">Corresponding <text font="bold">author</text>.</note><contact name="Affiliation: " role="affiliation"> University of Electronic Science</contact><contact name="Email: " role="email"><text font="typewriter" fontsize="90%">gu@gmail.com</text></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_nested_macros_of_names_unfold_each"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob <text font="bold">Baker</text></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Cole</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_macro_of_names_with_contacts_keeps_markup"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Email: " role="email"><text font="typewriter">ann@a.edu</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Univ B</contact></creator>"#,
      ]),
    },
    // (the macro printing two names and a note, `cl9`'s overlap shape until AUT1: each name its creator, Univ Z both's)
    Row {
      repro:    repro!("sectioning-frontmatter/cl9_shared_macro_note"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text fontsize="91%">Ann Able</text></personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks"><text fontsize="91%">Univ X.</text></note><contact name="Affiliation: " role="affiliation"><text fontsize="91%">Univ Z</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname><text fontsize="91%">Bob Baker</text></personname><contact name="Affiliation: " role="affiliation"><text fontsize="91%">Univ Z</text></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/cl9_shared_macro_note_and_marks"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname><text fontsize="91%">Ann Able</text></personname><note mark="1" role="footnotemark"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags></note><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks"><text fontsize="91%">Univ X.</text></note><contact name="Affiliation: " role="affiliation"><text fontsize="91%"> Univ Z</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname><text fontsize="91%">Bob Baker</text></personname><note mark="2" role="footnotemark"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags></note><contact name="Affiliation: " role="affiliation"><text fontsize="91%"> Univ Z</text></contact></creator>"#,
      ]),
    },
    // (a kept mark past a gap or a comma shown as a request; the glued ones' italic is RED
    // `omnibus_author_math_mark_in_the_name_run_is_italic_red`, pinned as today)
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/omnibus_author_macro_of_names_mark_after_a_gap_is_shown"
      ),
      files:    &[],
      counts:   (0, 1),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup>1</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker <sup><text font="italic">2</text></sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Cole<sup>3</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Dee Dunn<sup>4</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Eve Ell<sup><text font="italic">5</text></sup></personname></creator>"#,
      ]),
    },
  ]);
}

/// GRP1: a declaration the block makes is replayed whole, cut from the tokens its events were read from (the unfolded ones), as
/// far as its call read (`field_notes::note_call_read`), or taken back by a box of that call. One row per path: a macro's
/// `\ifthenelse` unfolded (2605.24599), babel's robust `\foreignlanguage` (2606.14878), a macro `\begin` running the kernel's
/// `{CJK*}` (2606.12938), and the safe side, a robust call defining after it printed, replayed for a later name.
#[test]
fn a_declaration_is_replayed_as_far_as_its_call_read() {
  assert_rows(&[
    // (the e-mail lines' entry cut is RED `amsart_author_macro_lines_with_email_array_split_red`: fragments)
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/amsart_author_macro_with_ifthenelse_declaration_is_cut_from_its_reading"
      ),
      files:    &[],
      counts:   (0, 4),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"Ann Fodor<sup>1</sup></text></personname><contact name="Email: " role="email">ann@ubb.ro</contact></creator>"#,
        r#"<personname><text fontsize="120%">Bob Minuta<sup>2</sup></text></personname>"#,
        r#"href="mailto:bob@ubb.ro">bob@ubb.ro</ref></XMText></XMApp></XMCell></XMRow></XMArray></XMath></Math></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_foreignlanguage_declaration_reads_its_arguments"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Võ Tiến Phong<sup>1,2</sup></personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_begin_macro_declares_then_opens_cjk_environment"
      ),
      files:    &[],
      counts:   (0, 1),
      lines:    &["a name its events don't spell"],
      // (the space after "(" is CAU1's too: a residual of the CJK name's plain text)
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Bonny Y. Wang ( 汪玥)<sup>1</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Leander Thiele<sup>2</sup></personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "class-bindings/author_one_block/article_author_robust_call_prints_then_defines_is_replayed"
      ),
      files:    &[],
      counts:   (0, 1),
      lines:    &[],
      // (the italic mark as pinned above: RED `omnibus_author_math_mark_in_the_name_run_is_italic_red`)
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Lee<sup>1</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Ray<sup><text font="italic">2</text></sup> (with Ann Lee)</personname></creator>"#,
      ]),
    },
  ]);
}
