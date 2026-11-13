//! Guards for P28 (Option C P2-8a): IEEEtran's `\author` (`latexml_package/src/package/ieeetran_cls.rs`) reads a body's
//! author structure by meaning before it chooses its route and reads its grid — each top-level call of a document macro
//! holding names blocks, an `\and`-family separator or the grid's `\\` expanded (`blocks_as_printed`), each `\let` copy of
//! a block command written as it (`with_block_names`), the route whether the body then calls `\IEEEauthorblockN`
//! (`calls_names_block`), the grid's columns the separators the separator query reads (`grid_columns`) — and keeps
//! the grid's row-major order
//! (OXIDIZED_DESIGN_DIVERGENCES #128, witness 2403.16405). Code paths, one guard each: the grid transposed and the
//! blocks kept as written (pins of the order before P28), the plain route (a pin), a body held in a macro or written through `\\let` copies of the block commands (the route),
//! a wrapper's calls (the block route's token steps), the grid's row break or column by meaning (the transposition),
//! the body read again at the flush (P28m2, `\lx@IEEE@author@reread`), a marked block's address under a names call the
//! reading parts (P28m3).
//! Each converts its repros under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and
//! `--streaming`, and asserts the frontmatter whole, with exact Error and Warning counts.
use super::perfect_kernel_batchp26::converted_any;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A grid written column by column reads row by row (2x2, 3x2, with marks, a `\thanks` and a `\parbox`), and a single
/// row or a single column of blocks keeps its order: pins, the order the same on int158a and int168a (int168a, before and
/// after P28m, keeps in each affiliation the reference mark pdflatex prints before it: `<sup>*</sup>Univ A`).
#[test]
fn ieee_grids_read_by_rows_and_lists_keep_their_order() {
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_two_by_two_reads_by_rows"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ D</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_three_by_two_reads_by_rows"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ D</contact> </creator> <creator before=" " role="author"> <personname>Eve Ess</personname> <contact name="Affiliation: " role="affiliation">Univ E</contact> </creator> <creator before=" " role="author"> <personname>Fay Fox</personname> <contact name="Affiliation: " role="affiliation">Univ F</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_with_marks_and_a_note"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">*</sup>Univ A</contact> <contact name="Email: " role="email">ann@a.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3">†</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">†</sup>Univ C</contact> <contact name="Email: " role="email"><inline-block class="ltx_parbox" vattach="middle" width="85.4pt" xml:id="id5"> <p xml:id="id5.1">bob@c.org</p> </inline-block></contact> </creator> <creator before=" " role="author"> <personname>Cy Coe<sup xml:id="id6">†</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id7">†</sup>Univ C</contact> <note class="ltx_note_frontmatter ltx_thanks_funding" role="thanks" xml:id="id8">Cy is funded by X.</note> </creator> <creator before=" " role="author"> <personname>Dan Dee<sup xml:id="id9">*</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id10">*</sup>Univ A</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_blocks_one_row_keep_their_order"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A<break/>City A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B<break/>City B</contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C<break/>City C</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(
      repro!("ieeetran_author_blocks_one_column_keep_their_order"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A body with no names block and no document macro takes the plain route, each grade its author's: a pin. A names list
/// held in a document macro is read as it prints (RED on int158a: the grade lost, each name written as plain text with
/// a Warning; green on int168a too, the grade the reading's contact since P5a, OD #1016).
#[test]
fn ieee_plain_route_reads_names_and_grades() {
  assert_eq!(
    converted_any(
      repro!("ieeetran_author_plain_route_reads_names_and_grades"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact role="membership">Member, IEEE</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact role="membership">Fellow, IEEE</contact> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">A. Able is with Univ A.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_plain_route_names_in_a_macro"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact role="membership">Member, IEEE</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A body held whole in a document macro takes the block route (RED on int158a: one nameless creator, the names lost;
/// int168a: Ann Able twice, Bob Baker lost), a regular grid in it read by rows; so do blocks written through `\let`
/// copies of the block commands (RED on int158a and int168a: Bob Baker lost, Ann Able twice); journal blocks in one keep
/// each name's `\IEEEmembership` grade (P5a's contact on P2-8a's route; RED on int168a: Bob Baker and his grade lost).
#[test]
fn ieee_block_calls_by_meaning_take_the_block_route() {
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_in_a_macro"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_regular_in_a_macro"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ D</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(
      repro!("ieeetran_author_blocks_let_copies_of_the_block_commands"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(
      repro!("ieeetran_author_blocks_in_a_macro_keep_their_grades"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact role="membership">Member, IEEE</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact role="membership">Fellow, IEEE</contact> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A wrapper printing the separator (`\pair{…}{…}`) or each cell's blocks (`\cell{…}{…}`) is read as it prints before the
/// block route's token steps (RED on int158a: the calls' groups read as bare text, creators scrambled and doubled).
#[test]
fn ieee_wrapper_calls_are_read_as_they_print() {
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_through_a_wrapper"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_wrapper_cells_in_rows"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ D</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// The grid's row break held in a macro, or its column a `\let` copy of `\and`, transposes as written ones do (RED on
/// int158a: the column order).
#[test]
fn ieee_grid_row_and_column_by_meaning() {
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_row_break_in_a_macro"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ D</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_grid_let_copy_of_and"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ D</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// The body is read again at the flush, with the definitions `\maketitle` expands `\@author` under (P28m2,
/// `\lx@IEEE@author@reread`): a macro renewed after `\author` or in `\AtBeginDocument` (RED on P28m: Ann Able and Bob
/// Baker invented, Cy Coe lost; int168a: Cy Coe twice), one defined only after it (the plain block taken back, none of
/// its items kept; RED on int168a and P28m: Bob Baker lost, Ann Able twice), a plain names list renewed (RED on P28m).
#[test]
fn ieee_author_body_read_at_the_flush() {
  let cy_coe = r##"<title>T</title> <creator role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##;
  assert_eq!(
    converted_any(repro!("ieeetran_author_macro_renewed_after_author"), 0),
    cy_coe
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_macro_renewed_at_begin_document"), 0),
    cy_coe
  );
  assert_eq!(
    converted_any(repro!("ieeetran_author_macro_defined_after_author"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted_any(
      repro!("ieeetran_author_plain_macro_renewed_after_author"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Cy Coe</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// An address printed in an affiliation block led by a reference mark, under a names call the reading parts into several
/// names, is the one name's showing that mark (P28m3, `_bymark`, `Resolver::entry_owners`): IEEEtran's one names block
/// listing marked names, written or held in macros (2510.10414), and REVTeX's one `\author` call (RED on int169a: the
/// addresses no name spells on a nameless creator below the authors).
#[test]
fn marked_block_address_goes_to_the_name_showing_the_mark() {
  let ieee = r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">*</sup>Univ X, C, D</contact> <contact name="Email: " role="email">chteng@x.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3">†</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">†</sup>Univ Y, E, F</contact> <contact name="Email: " role="email">idbj.real@y.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##;
  assert_eq!(
    converted_any(repro!("ieeetran_author_names_list_marked_block_address"), 0),
    ieee
  );
  assert_eq!(
    converted_any(
      repro!("ieeetran_author_blocks_in_macros_marked_block_address"),
      0
    ),
    ieee
  );
  assert_eq!(
    converted_any(
      repro!("revtex_author_names_list_marked_affiliation_address"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ X, C, D</contact> <contact name="Email: " role="email">chteng@x.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3">2</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">2</sup>Univ Y, E, F</contact> <contact name="Email: " role="email">idbj.real@y.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}
