//! Red/green guards for perfect-kernel phase-59 batches: the TikZ performance levers (59a) and
//! the wheelchart pgfmath roots (59b).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch57::{RAW, assert_elements};

/// 59a: number scanning keeps the token after the signs in hand (tex.web §440), but an undefined
/// conditional met there is read again as its error stub (`\iffalse`, State.pm:537-545), as Perl's
/// chain of reads does: `\ifnum\ifdraft 1\else 0\fi=0` is true. Kept in hand, it fell to "Missing
/// number". Repro expansion-primitives/undefined_conditional_in_a_number_expands.
#[test]
fn undefined_conditional_in_a_number_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/undefined_conditional_in_a_number_expands.tex"
  );
  assert_elements(tex, RAW, (1, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>yes.</p></para>"##,
  )]);
}

/// 59a: the same in a dimension scan (`read_dimension`): the stub is read again before the
/// internal dimension and glue are tested, so `\ifcompact\columnwidth\else\textwidth\fi` is
/// `\textwidth` (kept in hand it was 0pt and the left-over `\textwidth` assigned 0 to itself).
/// Repro expansion-primitives/undefined_conditional_in_a_dimension_expands.
#[test]
fn undefined_conditional_in_a_dimension_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/undefined_conditional_in_a_dimension_expands.tex"
  );
  assert_elements(tex, RAW, (3, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>A:[345.0pt]</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>B:[7.0pt]</p></para>"##,
    ),
    ("para", "p3", r##"<para xml:id="p3"><p>C:yes.</p></para>"##),
  ]);
}

/// 59a: an undefined conditional that ends a factor's digits is read again as its stub before
/// the decimal point is tested (Perl `readFactor`, Gullet.pm:876-880): `1\ifFa\else.5\fi pt` is
/// 1.5pt, not 1pt and an "Illegal unit". Repro
/// expansion-primitives/undefined_conditional_after_digits_expands.
#[test]
fn undefined_conditional_after_digits_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/undefined_conditional_after_digits_expands.tex"
  );
  assert_elements(tex, RAW, (1, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>A:[1.5pt]</p></para>"##,
  )]);
}

/// 59b (wheelchart root 1): pgfmath's `?:` binds looser than the comparisons and `&&`/`||`
/// (pgfmathparser.code.tex:898-916), chained ternaries group to the left; the chosen branch keeps
/// its form (`7`, `-1.0`, `5.0`), as does `ifthenelse`'s (an integer still ends an `\ifnum`).
/// wheelchart's `arc data dir={\WCmidangle<180?1:-1}` was 0 and walked a zero-length arc forever.
/// Repro graphics-tikz/pgfmath_ternary_after_a_comparison.
#[test]
fn pgfmath_ternary_after_a_comparison() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_ternary_after_a_comparison.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>[-1.0][7][0][5.0]</p></para>"##,
    ),
    ("para", "p2", r##"<para xml:id="p2"><p>[4]A[1]</p></para>"##),
  ]);
}

/// 59b (wheelchart root 2): `\pgfmathsetlength\reg{+…}` reads from its argument alone (pgf's
/// `#1#2\unskip`, pgfmathcalc.code.tex:30-38), so the `\ifdim` after it compares the new value;
/// a skip register takes glue, a mu register mu glue; tokens after the value are typeset after the
/// assignment. Repro graphics-tikz/pgfmathsetlength_reads_only_its_argument.
#[test]
fn pgfmathsetlength_reads_only_its_argument() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmathsetlength_reads_only_its_argument.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>N[1.0pt plus 2.0pt]</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>CXY[3.0pt]D[3.0mu]</p></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><p>EXF[2.0pt]</p></para>"##,
    ),
  ]);
}

/// 59b (wheelchart root 3): pgfmath's `width("…")`/`height("…")`/`depth("…")` measure the typeset
/// text (Perl pgfmath.code.tex.ltxml:523-537), as pdflatex does. Repro
/// graphics-tikz/pgfmath_width_of_a_string.
#[test]
fn pgfmath_width_of_a_string() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_width_of_a_string.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[13.05559][6.83331][1.94444]</p></para>"##,
  )]);
}

/// 59b (wheelchart root 4): a pgfmath string result holding an internal control word
/// (`\,` expanded to `\lx@thinspace`) is read back with `@` a letter in the document body; an
/// undefined `@`-name is not (`"\relax@x"` stays `\relax` then `@x`).
/// Repro graphics-tikz/pgfmath_string_result_keeps_internal_names.
#[test]
fn pgfmath_string_result_keeps_internal_names() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_string_result_keeps_internal_names.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      "<para xml:id=\"p1\"><p>C\u{2009}%D</p></para>",
    ),
    ("para", "p2", r##"<para xml:id="p2"><p>[@x]</p></para>"##),
  ]);
}

/// 59b: a user-declared pgfmath function keeps its result's form (an integer `\pgfmathresult`
/// stays an integer) and receives integer arguments as integers, so `?:`, `ifthenelse` and
/// `\ifnum` on its result behave as in pgf. Repro
/// graphics-tikz/pgfmath_user_function_keeps_integer_form.
#[test]
fn pgfmath_user_function_keeps_integer_form() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_user_function_keeps_integer_form.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[1][1][1.0][7][6.0]YES</p></para>"##,
  )]);
}

/// 59b: a user-declared pgfmath function whose result is not a number runs its body once and is
/// reported once (its counter steps once). Repro graphics-tikz/pgfmath_user_function_runs_once.
#[test]
fn pgfmath_user_function_runs_once() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_user_function_runs_once.tex"
  );
  assert_elements(tex, RAW, (1, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>[1]</p></para>"##,
  )]);
}

/// 59b: a pgfmath expression is evaluated in one group, as `\pgfmathparse` does
/// (pgfmathparser.code.tex:21, :145-148; KPE #421): a tikzmath function's parameter does not
/// outlive the parse (2605.28612's `\CircleCenterX1` read 111 cm), and a later call of the same
/// parse sees an earlier one's local definitions. Repro
/// graphics-tikz/pgfmath_function_body_is_grouped.
#[test]
fn pgfmath_function_body_is_grouped() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_function_body_is_grouped.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    ("para", "p1", r##"<para xml:id="p1"><p>[0]</p></para>"##),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>[5.0][1]U</p></para>"##,
    ),
  ]);
}

/// 59d: what code appends to `\@author` after `\author` stored it is the author block LaTeX prints
/// with the names; at `\maketitle` it is handed to the creators as unlabelled author-block rows
/// (`\lx@author@tail`, frontmatter_stores.rs), not dropped (Perl drops it). (Open: a row with a
/// superscript mark is placed by its label, after the unmarked rows, not in source order.) Repro
/// sectioning-frontmatter/author_block_appended_after_author_is_kept.
#[test]
fn author_block_appended_after_author_is_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_block_appended_after_author_is_kept.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>A. Author</personname><contact \
     role=\"authorblock\">Emails: a@b.c</contact><contact role=\"authorblock\">Affiliation \
     one</contact></creator>",
  );
}

/// 59d: the extended T2A letters print through t2aenc.def's `\DeclareTextSymbol`s; the fontenc
/// binding's empty stubs made the declarations refuse the bare commands, so ј љ њ ћ ђ џ і ї є ґ
/// vanished (serbian-def-cyr/proba, cmpj/template). Repro fonts-nfss/t2a_extended_letters_print.
#[test]
fn t2a_extended_letters_print() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t2a_extended_letters_print.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>jљњћђџ JЉЊЋЂЏ ґєiї ж</p></para>",
  )]);
}

/// 59d: a class whose `\author` appends (one `\author`/`\affil` pair per author) gives each author the
/// rows appended after it (`\lx@author@flush`), not the last author's rows to all. Repro
/// sectioning-frontmatter/author_block_follows_each_author.
#[test]
fn author_block_follows_each_author() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_block_follows_each_author.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  // (whole creator elements, whitespace aside)
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  for creator in [
    "<creatorrole=\"author\"><personname>AliceA</personname><contactrole=\"authorblock\">InstX</contact></creator>",
    "<creatorbefore=\"\"role=\"author\"><personname>BobB</personname><contactrole=\"authorblock\">InstY</contact></creator>",
  ] {
    assert!(flat.contains(creator), "{creator} in {xml}");
  }
}

/// 59d: the author block after a second, replacing `\author` goes to every creator that call made
/// (counted after the earlier authors are removed). Repro
/// sectioning-frontmatter/author_block_after_a_replaced_author.
#[test]
fn author_block_after_a_replaced_author() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_block_after_a_replaced_author.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  for creator in [
    "<creatorrole=\"author\"><personname>AliceA</personname><contactrole=\"authorblock\">Email:all@y.org</contact></creator>",
    "<creatorbefore=\"\"role=\"author\"><personname>BobB</personname><contactrole=\"authorblock\">Email:all@y.org</contact></creator>",
    "<creatorbefore=\"\"role=\"author\"><personname>CarolC</personname><contactrole=\"authorblock\">Email:all@y.org</contact></creator>",
  ] {
    assert!(flat.contains(creator), "{creator} in {xml}");
  }
  // the replaced author is gone: exactly the three creators of the second call
  assert_eq!(
    flat.matches("<creator").count(),
    3,
    "three creators in {xml}"
  );
  assert!(
    !flat.contains("Xavier"),
    "the replaced author is dropped in {xml}"
  );
}

/// 59d: an author-block row whose mark matches no author is kept with the shared creator
/// (`authorblock` is a shared contact role). Repro sectioning-frontmatter/author_block_orphan_mark_is_kept.
#[test]
fn author_block_orphan_mark_is_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_block_orphan_mark_is_kept.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  for creator in [
    "<creatorrole=\"author\"><personname>A.Author</personname><contactrole=\"authorblock\">Affiliationone</contact></creator>",
    "<creatorrole=\"author\"><contactrole=\"authorblock\">Affiliationtwo</contact></creator>",
  ] {
    assert!(flat.contains(creator), "{creator} in {xml}");
  }
}
