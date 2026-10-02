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

/// 59e: fancyhdr's `\f@nch@setoffs`, re-run by a document after `\newgeometry` (the wheelchart manual),
/// sizes running heads only, and converts to nothing. Repro singletons/fancyhdr_setoffs_after_newgeometry.
#[test]
fn fancyhdr_setoffs_after_newgeometry() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/fancyhdr_setoffs_after_newgeometry.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Body.</p></para>"##,
  )]);
}

/// 59e: an `\author` after a `\maketitle` that left it live (KOMA ≥ 3.12) replaces the authors that
/// `\maketitle` digested, as the title is replaced. Repro sectioning-frontmatter/author_after_a_digesting_maketitle.
#[test]
fn author_after_a_digesting_maketitle() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_after_a_digesting_maketitle.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert_eq!(flat.matches("<creator").count(), 1, "one creator in {xml}");
  assert!(
    flat.contains("<creatorrole=\"author\"><personname>AliceA</personname></creator>"),
    "the last author alone in {xml}"
  );
  assert!(
    flat.contains("<title>T</title>") && !flat.contains("T0"),
    "the last title in {xml}"
  );
}

/// 59e: text right before an abstract, the cover its first flush wraps as `<titlepage>`, is closed
/// before it moves (was: `Error:malformed:ltx:document` at `\end{document}`). Repro
/// sectioning-frontmatter/abstract_after_body_text.
#[test]
fn abstract_after_body_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_after_body_text.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "abstract",
    "abstract1",
    r##"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>x</p></abstract>"##,
  )]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert!(
    flat.contains("<titlepage><blockxml:id=\"p1\"><p>foo</p></block></titlepage><abstract"),
    "the cover, then the abstract, in {xml}"
  );
}

/// 59e: an `\author` after a fallback flush (no later `\maketitle`) adds its author.
/// Repro sectioning-frontmatter/author_after_a_fallback_flush_is_added.
#[test]
fn author_after_a_fallback_flush_is_added() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_after_a_fallback_flush_is_added.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert_eq!(flat.matches("<creator").count(), 2, "two creators in {xml}");
  assert!(
    flat.contains(
      "<creatorrole=\"author\"><personname>AliceA</personname></creator><creatorbefore=\"\"role=\"author\"><personname>LateL</personname></creator>"
    ),
    "both authors in {xml}"
  );
}

/// 59e: a superseding title block's affiliation and emails go to its own authors.
/// Repro sectioning-frontmatter/author_superseded_keeps_contacts_with_owners.
#[test]
fn author_superseded_keeps_contacts_with_owners() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_superseded_keeps_contacts_with_owners.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert_eq!(flat.matches("<creator").count(), 2, "two creators in {xml}");
  for creator in [
    "<creatorrole=\"author\"><personname>AliceA</personname><contactname=\"Affiliation:\"role=\"affiliation\">InstA</contact><contactname=\"Email:\"role=\"email\">alice@a.org</contact></creator>",
    "<creatorbefore=\"\"role=\"author\"><personname>CarolC</personname><contactname=\"Affiliation:\"role=\"affiliation\">InstA</contact><contactname=\"Email:\"role=\"email\">carol@a.org</contact></creator>",
  ] {
    assert!(flat.contains(creator), "{creator} in {xml}");
  }
}

/// 59e: a superseding `\maketitle` hands the class's stores (`\@address`, `\@email`, set once) to its
/// authors again. Repro sectioning-frontmatter/superseding_maketitle_rehands_class_stores.
#[test]
fn superseding_maketitle_rehands_class_stores() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/superseding_maketitle_rehands_class_stores.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert_eq!(flat.matches("<creator").count(), 1, "one creator in {xml}");
  assert!(
    flat.contains(
      "<creatorrole=\"author\"><personname>AliceA</personname><contactname=\"Address:\"role=\"address\">SharedInst</contact><contactname=\"Email:\"role=\"email\">max@m.org</contact></creator>"
    ),
    "the stores with the new author in {xml}"
  );
}

/// 59e: a `\thanks` in a superseding `\author` is the author's note, not another author.
/// Repro sectioning-frontmatter/thanks_in_a_superseding_author.
#[test]
fn thanks_in_a_superseding_author() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/thanks_in_a_superseding_author.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert_eq!(flat.matches("<creator").count(), 1, "one creator in {xml}");
  assert!(
    flat.contains(
      "<creatorrole=\"author\"><personname>AliceA</personname><noteclass=\"ltx_note_frontmatterltx_thanks_funding\"role=\"thanks\">FundedbyX.</note></creator>"
    ),
    "the note with its author in {xml}"
  );
}

/// 59e: a store set between two `\maketitle`s before the new `\author` goes to the new authors.
/// Repro sectioning-frontmatter/store_set_before_a_superseding_author.
#[test]
fn store_set_before_a_superseding_author() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_set_before_a_superseding_author.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert_eq!(flat.matches("<creator").count(), 1, "one creator in {xml}");
  assert!(
    flat.contains(
      "<creatorrole=\"author\"><personname>AliceA</personname><contactname=\"Address:\"role=\"address\">Shared</contact><contactname=\"Email:\"role=\"email\">alice@a.org</contact></creator>"
    ),
    "the stores with the new author in {xml}"
  );
}

/// 59e: a replacing `\author` makes `\thanks` live for its own argument only; the document's own
/// `\thanks` in body text stays inline. Repro sectioning-frontmatter/thanks_after_a_late_author_stays_the_documents.
#[test]
fn thanks_after_a_late_author_stays_the_documents() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/thanks_after_a_late_author_stays_the_documents.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert!(
    flat.contains("<paraxml:id=\"p1\"><p>Text.Moretext[Bracketnote]after.</p></para>"),
    "the note inline in {xml}"
  );
  assert!(
    !flat.contains("pubnote"),
    "no note in the title block in {xml}"
  );
}

/// 59e: a disabled `\and` (`\relax` after a `\maketitle`) splits a later `\author` at itself only, not
/// at every `\relax` alias. Repro sectioning-frontmatter/relax_aliases_do_not_split_a_late_author.
#[test]
fn relax_aliases_do_not_split_a_late_author() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/relax_aliases_do_not_split_a_late_author.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert_eq!(flat.matches("<creator").count(), 2, "two creators in {xml}");
  for creator in [
    "<creatorrole=\"author\"><personname>AliceAnders</personname></creator>",
    "<creatorbefore=\"\"role=\"author\"><personname>BobBrown</personname><noteclass=\"ltx_note_frontmatterltx_thanks_note\"role=\"thanks\">NoteB.</note></creator>",
  ] {
    assert!(flat.contains(creator), "{creator} in {xml}");
  }
}

/// 59f: an abstract ends the paragraph it interrupts (LaTeX's begins with `\par`): text after it is a
/// paragraph of its own, after the abstract. Repro sectioning-frontmatter/abstract_ends_the_open_paragraph.
#[test]
fn abstract_ends_the_open_paragraph() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_ends_the_open_paragraph.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert!(
    flat.contains(
      "<titlepage><blockxml:id=\"p1\"><p>foo</p></block></titlepage><abstractinlist=\"toc\"name=\"Abstract\"xml:id=\"abstract1\"><p>x</p></abstract><paraxml:id=\"p2\"><p>bar</p></para>"
    ),
    "foo, the abstract, then bar in {xml}"
  );
  // (the paragraph `\par` ends has no trailing newline)
  assert!(
    xml.contains("<p>foo</p>"),
    "foo's paragraph ends at the abstract in {xml}"
  );
}

/// 59g: a document's `\newcommand` of a name a fallback class (OmniBus) defined wins: `\abst` is the
/// document's absolute value, not OmniBus's abstract alias. Repro singletons/omnibus_alias_shadows_newcommand.
#[test]
fn omnibus_alias_yields_to_newcommand() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/omnibus_alias_shadows_newcommand.tex"
  );
  // (one Warning: the class has no binding, OmniBus stands in)
  let xml = assert_elements(tex, RAW, (0, 1), &[]);
  let flat: String = xml.chars().filter(|c| !c.is_whitespace()).collect();
  assert!(
    flat.contains(
      r#"<Mathmode="inline"tex="\frac{1}{|a|}"text="1/absolute-value@(a)"xml:id="p1.m1"><XMath><XMApp><XMTokmathstyle="text"meaning="divide"role="FRACOP"/><XMTokfontsize="70%"meaning="1"role="NUMBER">1</XMTok><XMDual><XMApp><XMTokmeaning="absolute-value"/><XMRefidref="p1.m1.1"/></XMApp><XMWrap><XMTokfontsize="70%"role="OPEN"stretchy="false">|</XMTok><XMTokfont="italic"fontsize="70%"role="UNKNOWN"xml:id="p1.m1.1">a</XMTok><XMTokfontsize="70%"role="CLOSE"stretchy="false">|</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#
    ),
    "the document's \\abst in {xml}"
  );
  assert!(!xml.contains("abstract"), "no abstract opened in {xml}");
}

/// 59h: a raw float end that calls the kernel's `\@endfloatbox` (floatrow's `\float@dblend` for a
/// `\DeclareNewFloatType`) on the `{@float}`/`{@dblfloat}` binding's own frame closes nothing the
/// binding did not open: the `\egroup\color@endbox` of `\@xfloat`'s box go with it (3 errors per
/// float before). Witness kaytannollista-latexia. Repro
/// captions-floats/floatrow_new_float_type_closes_its_box.
#[test]
fn floatrow_new_float_type_closes_its_box() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/floatrow_new_float_type_closes_its_box.tex"
  );
  // (floatrow's float names are `Esim~<n>`: a no-break space, but in the typed reference)
  assert_elements(tex, RAW, (0, 0), &[
    (
      "float",
      "esim1",
      "<float class=\"ltx_float_esim\" inlist=\"loesim\" xml:id=\"esim1\"><tags><tag>Esim\u{a0}1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Esim 1</tag></tags><p>x</p><toccaption><tag close=\" \">1</tag>A</toccaption><caption><tag close=\" \">Esim\u{a0}1</tag>A</caption></float>",
    ),
    (
      "float",
      "esim2",
      "<float class=\"ltx_float_esim\" inlist=\"loesim\" xml:id=\"esim2\"><tags><tag>Esim\u{a0}2</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Esim 2</tag></tags><p>y</p><toccaption><tag close=\" \">2</tag>B</toccaption><caption><tag close=\" \">Esim\u{a0}2</tag>B</caption></float>",
    ),
  ]);
}

/// 59h: a box a raw opener makes itself (`\color@vbox\normalcolor\vbox\bgroup`…`\@floatboxreset`,
/// commedit's and examplep's shape) keeps `\@endfloatbox`'s closers, in the text and in a figure.
/// Repro captions-floats/raw_float_box_keeps_its_closers.
#[test]
fn raw_float_box_keeps_its_closers() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/raw_float_box_keeps_its_closers.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p2",
      r##"<para vattach="bottom" xml:id="p2"><p>Inside the box.</p></para>"##,
    ),
    ("para", "p3", r##"<para xml:id="p3"><p>After.</p></para>"##),
    (
      "figure",
      "S0.F1",
      r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p vattach="bottom">Box in a figure.</p><toccaption><tag close=" ">1</tag>F</toccaption><caption><tag close=": ">Figure 1</tag>F</caption></figure>"##,
    ),
  ]);
}

/// 59i: a tcolorbox listing box's `hypertarget=` names its listing: the begin line's `*[boxb]`
/// (`{ s o }`, read by listings' raw reader alone) reaches the options' `#2`, and the box's phantom
/// code runs at the start of the listing's first line, as tcolorbox runs it inside the box; a box
/// without the optional makes no anchor (`\IfValueT` of xparse's no-value marker). Witness
/// jsonparse-doc (7 dangling links). Repro captions-floats/tcb_listing_keeps_its_hypertarget.
#[test]
fn tcb_listing_keeps_its_hypertarget() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/tcb_listing_keeps_its_hypertarget.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p2",
    r##"<para xml:id="p2"><listing class="ltx_lstlisting" data="fHN0b3JlIGlufD17PHRsPn0=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><anchor xml:id="boxb"/>|<text class="ltx_lst_identifier">store</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">in</text>|={&lt;<text class="ltx_lst_identifier">tl</text>&gt;}</listingline></listing><listing class="ltx_lstlisting" data="fGdsb2JhbHw=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx2"><anchor xml:id="boxc"/>|<text class="ltx_lst_identifier">global</text>|</listingline></listing><listing class="ltx_lstlisting" data="fG5vbmV8" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx3">|<text class="ltx_lst_identifier">none</text>|</listingline></listing><p>Links: <ref idref="boxa">A</ref>, <ref idref="boxb">B</ref>, <ref idref="boxc">C</ref>.</p></para>"##,
  )]);
  // the plain box keeps its own target
  assert_element(
    &xml,
    "anchor",
    &["xml:id=\"boxa\""],
    r#"<anchor xml:id="boxa"/>"#,
  );
}

/// 59i: a listing box's `label=` names its own listing (its first line; run before the listing,
/// the label named the element before it), and `step=` steps its counter. Repro
/// captions-floats/tcb_listing_label_names_its_listing.
#[test]
fn tcb_listing_label_names_its_listing() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/tcb_listing_label_names_its_listing.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><listing class="ltx_lstlisting" data="YSBi" dataencoding="base64" datamimetype="text/plain"><listingline labels="LABEL:first" xml:id="lstnumberx1"><text class="ltx_lst_identifier">a</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">b</text></listingline></listing><listing class="ltx_lstlisting" data="YyBk" dataencoding="base64" datamimetype="text/plain"><listingline labels="LABEL:second" xml:id="lstnumberx2"><text class="ltx_lst_identifier">c</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">d</text></listingline></listing><listing class="ltx_lstlisting" data="ZSBm" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx3"><text class="ltx_lst_identifier">e</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">f</text></listingline></listing><p>See <ref labelref="LABEL:first"/>, <ref labelref="LABEL:second"/>; section 1.</p></para>"##,
  )]);
}

/// 59i review: a tcolorbox listing box's begin-line arguments read as xparse reads them — a brace
/// group hides a `]` (`[{a]b}]` was a Fatal: unbalanced stand-in), an absent `!O{dflt}` is its
/// default, a leading-optional signature (`{ o s }`) still reads a later `*` (the raw reader's
/// pushback branch), and an empty listing keeps its phantom code (anchor `boxe`, `step=demo`) on
/// one empty line, the only place a `listing` has. Repro
/// captions-floats/tcb_listing_begin_line_edge_cases.
#[test]
fn tcb_listing_begin_line_edge_cases() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/tcb_listing_begin_line_edge_cases.tex"
  );
  let listing = |data: &str, n: usize| {
    format!(
      r##"<listing class="ltx_lstlisting" data="{data}" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx{n}"><text class="ltx_lst_identifier">x</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>{n}</listingline></listing>"##
    )
  };
  let paras = [
    format!(r#"<para xml:id="p1">{}<p>A: nostar:a]b.</p></para>"#, listing("eCA9IDE=", 1)),
    format!(r#"<para xml:id="p2">{}<p>B: star:none.</p></para>"#, listing("eCA9IDI=", 2)),
    format!(r#"<para xml:id="p3">{}<p>C: dflt.</p></para>"#, listing("eCA9IDM=", 3)),
    format!(r#"<para xml:id="p4">{}<p>D: star.</p></para>"#, listing("eCA9IDQ=", 4)),
    r##"<para xml:id="p5"><listing class="ltx_lstlisting" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx5"><anchor xml:id="boxe"/></listingline></listing><p>E: 2, <ref idref="boxe">to the box</ref>.</p></para>"##.to_string(),
  ];
  assert_elements(tex, RAW, (0, 0), &[
    ("para", "p1", &paras[0]),
    ("para", "p2", &paras[1]),
    ("para", "p3", &paras[2]),
    ("para", "p4", &paras[3]),
    ("para", "p5", &paras[4]),
  ]);
}

/// 59j: a package that installs the kernel's checking `\fontencoding` (luatexja, lltjfont.sty:549-551,
/// :573-574 = latex.ltx:10490-10492) never sees LaTeXML's private `ASCII` font map, which `\UrlFont`
/// and `\verbatim@font` select through `\lx@fontencoding` — qworld's 7 "Encoding scheme `ASCII'
/// unknown" (one per `\url`), 33 in 11 luatexja manuals. `~`/`^` stay ASCII. Repro
/// fonts-nfss/url_font_survives_a_checking_fontencoding.
#[test]
fn url_font_survives_a_checking_fontencoding() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/url_font_survives_a_checking_fontencoding.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>A <ref class="ltx_nolink ltx_url" font="typewriter" href="https://ctan.org/pkg/q~orld">https://ctan.org/pkg/q~orld</ref> B</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>C <text font="typewriter">x~y^z</text> D</p></para>"##,
    ),
  ]);
}
