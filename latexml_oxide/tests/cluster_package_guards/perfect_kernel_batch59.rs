//! Red/green guards for perfect-kernel phase-59 batches: the TikZ performance levers (59a) and
//! the wheelchart pgfmath roots (59b).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with};

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

/// 59k: an `\index`/`\glossary` entry is the string `\@wrindex`/`\@wrglossary` write under `\@sanitize`
/// (latex.ltx:17720-17740, :1778), so its `&` is a literal & in the indexphrase and its key, not a
/// `Stray alignment` (robustsample.tex:45/:59, robustglossary's `formula&explanation`); `\@index`
/// reverts to nothing, as Perl's (latex_constructs.pool.ltxml:4409-4412), so a formula holding an
/// `\index` keeps its own tex=. The entry sits one brace level in, so in an alignment its `&` ends no
/// cell (an `align` lost the rest of the document), and its neutral font keeps the text encoding
/// (T1 `<`, not OT1 `¡`). Repro index-bib/index_entry_ampersand_is_literal.
#[test]
fn index_entry_ampersand_is_literal() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/index_entry_ampersand_is_literal.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>G: Text.<indexmark inlist="glo"><indexphrase key="B&amp;Borel subgroup"><Math mode="inline" tex="B" text="B" xml:id="p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">B</XMTok></XMath></Math>&amp;Borel subgroup</indexphrase></indexmark></p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>I: Text.<indexmark><indexphrase key="B&amp;Borel"><Math mode="inline" tex="B" text="B" xml:id="p2.m1"><XMath><XMTok font="italic" role="UNKNOWN">B</XMTok></XMath></Math>&amp;Borel</indexphrase></indexmark></p></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><p>Math <Math mode="inline" tex="x+y" text="x + y" xml:id="p3.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math><indexmark><indexphrase key="inmath">inmath</indexphrase></indexmark> end.</p></para>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><p><text font="bold">Bold<indexmark><indexphrase key="plainword">plainword</indexphrase></indexmark></text> text.</p></para>"##,
    ),
    // in an alignment cell the entry's `&` ends no cell: the `<{X}` template stays after the entry
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><tabular vattach="middle"><tbody><tr><td align="center"><text font="bold">e<indexmark><indexphrase key="y&amp;z">y&amp;z</indexphrase></indexmark>X</text></td><td align="center">f</td></tr></tbody></tabular></para>"##,
    ),
    // in `align`, both equations and the paragraph after it; under T1 the entry's `<` stays `<`
    (
      "para",
      "p6",
      r##"<para xml:id="p6"><equationgroup class="ltx_eqn_align" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><indexmark><indexphrase key="x&amp;y">x&amp;y</indexphrase></indexmark><MathFork><Math tex="\displaystyle a=b" text="a = b" xml:id="S0.E1.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle=b" text="absent = b" xml:id="S0.E1.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle c=d" text="c = d" xml:id="S0.E2.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle c" text="c" xml:id="S0.E2.m1"><XMath><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle=d" text="absent = d" xml:id="S0.E2.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup><p>After the alignment, T1: x<indexmark><indexphrase key="a&lt;b&gt;c">a&lt;b&gt;c</indexphrase></indexmark>.</p></para>"##,
    ),
  ]);
}

/// 59l: biblatex-ext's add-on packages (tabular bibliographies, open-access symbols) are raw-loaded over
/// the biblatex binding instead of re-running it, so `\defbibtabular`/`\printbibtabular`/`\oasymbol`
/// exist; the tabular worker prints the binding's bibliography (its rows filter with biblatex
/// internals the binding stands in for). The biblatex-ext manual lost its last two sections to the
/// undefined commands (10 errors -> 1). Repro index-bib/biblatex_ext_addons_load.
#[test]
fn biblatex_ext_addons_load() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_ext_addons_load.tex");
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Text <cite class="ltx_citemacro_cite"><bibref bibrefs="sigfridsson" separator=";" show="AuthorsPhrase1Year" yyseparator=","><bibrefphrase>, </bibrefphrase></bibref></cite>. Symbol: defined.</p></para>"##,
    ),
    (
      "bibliography",
      "bib",
      r##"<bibliography bibstyle="biblatex" citestyle="authoryear" files="biblatex-examples.bib" inlist="toc" xml:id="bib"><title>References</title></bibliography>"##,
    ),
  ]);
}

/// 59l: biblatex-ext-oa's `\apptocmd` patches of the `begentry` and `doi+eprint+url` bibmacros
/// (biblatex-ext-oa.sty:401, :429) find no bibmacro — the binding's `\newbibmacro` stores none — so
/// the package warns "Failed to patch" (an accurate warning: no open-access mark prints) under each
/// symbol package it loads, and converts otherwise. Repro index-bib/biblatex_ext_oa_patches_warn.
#[test]
fn biblatex_ext_oa_patches_warn() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_ext_oa_patches_warn.tex");
  for backend in ["tikz", "l3draw", "pict2e"] {
    let tex = tex.replace("symbolpackage=tikz", &format!("symbolpackage={backend}"));
    assert_elements_with(
      &tex,
      RAW,
      (0, 1),
      &["Failed to patch 'begentry' bibmacro"],
      &[
        (
          "para",
          "p1",
          r##"<para xml:id="p1"><p>Text <cite class="ltx_citemacro_cite"><bibref bibrefs="sigfridsson" separator=";" show="AuthorsPhrase1Year" yyseparator=","><bibrefphrase>, </bibrefphrase></bibref></cite>. Symbol: defined.</p></para>"##,
        ),
        (
          "bibliography",
          "bib",
          r##"<bibliography bibstyle="biblatex" citestyle="authoryear" files="biblatex-examples.bib" inlist="toc" xml:id="bib"><title>References</title></bibliography>"##,
        ),
      ],
    );
  }
}

/// 59l: an open-access symbol package loaded without biblatex (biblatex-ext.tex:3927, "stand-alone")
/// does not bring the biblatex binding in, so the document's natbib bibliography stays: only an
/// overlay that loads biblatex itself (biblatex-cv) falls back to the binding. Repro
/// index-bib/biblatex_ext_symbol_package_alone.
#[test]
fn biblatex_ext_symbol_package_alone_keeps_the_bibliography() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/biblatex_ext_symbol_package_alone.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Text. Symbol: defined.</p></para>"##,
    ),
    (
      "bibliography",
      "bib",
      r##"<bibliography bibstyle="plainnat" citestyle="authoryear" files="biblatex-examples" inlist="toc" xml:id="bib"><title>References</title></bibliography>"##,
    ),
  ]);
}

/// 59m: `\hypertarget`/`\hyperdef` around display material. In running text the anchor is a point
/// destination before the display — hyperref's default `\Hy@nestingfalse` (hyperref.sty:323) makes
/// `\hypertarget` `\hyper@@anchor{#1}{\relax}#2` (:4805-4810) — and the localizing walk wraps a node
/// only where its parent may hold the anchor, so a display in a `\parbox` (philex.sty:136 `\lb`)
/// leaves its destination in the next paragraph, not inside `ltx:equation`. Both shapes were
/// `<equation><anchor><Math/></anchor></equation>`, schema-invalid (philexmanual; Perl the same).
/// Repro block-model/hypertarget_display_text_anchors_before_it.
#[test]
fn hypertarget_display_text_anchors_before_it() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/block-model/hypertarget_display_text_anchors_before_it.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p><inline-block class="ltx_parbox" vattach="middle" width="142.3pt"><equation class="ltx_centering" xml:id="S0.Ex1"><Math mode="display" tex="x=1" text="x = 1" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math></equation><p align="center"><anchor xml:id="compo"/>.</p></inline-block></p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>A <anchor xml:id="d"/></p><equation xml:id="S0.Ex2"><Math mode="display" tex="y=2" text="y = 2" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></equation><p>b.</p></para>"##,
    ),
  ]);
}

/// 59m (review): a `\hypertarget` around a block in vertical mode — a figure's content, a numbered `equation`,
/// an `align`, a display with `\text`, a `tabular` — keeps a visible destination, a bare anchor in the paragraph
/// after the block. The parent check alone let the walk pass a refused `ltx:Math`/`ltx:rule` and wrap generated
/// text: a figure's `typerefnum` tag (the HTML lost the id), an equation's refnum tag, an `align`'s or a tabular's
/// last cell, math `XMText` (where the math pass renamed the id: the link dangled). Refused anchor content is one
/// unit; `ltx:tags` and `ltx:MathBranch` are never entered. Repro
/// block-model/hypertarget_block_text_keeps_a_visible_destination.
#[test]
fn hypertarget_block_text_keeps_a_visible_destination() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/block-model/hypertarget_block_text_keeps_a_visible_destination.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "figure",
      "S0.F1",
      r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="autoref">Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><rule class="ltx_figure_panel" height="28.5pt" width="28.5pt"/><break class="ltx_break"/><p class="ltx_figure_panel"><anchor xml:id="f"/></p><toccaption><tag close=" ">1</tag>C</toccaption><caption><tag close=": ">Figure 1</tag>C</caption></figure>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="autoref">Equation 1</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="y=2" text="y = 2" xml:id="S0.E1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></equation><p><anchor xml:id="e"/>After e.</p></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><equationgroup class="ltx_eqn_align" xml:id="S0.EGx1"><equation xml:id="S0.E2"><tags><tag>(2)</tag><tag role="autoref">Equation 2</tag><tag role="refnum">2</tag></tags><MathFork><Math tex="\displaystyle y=2" text="y = 2" xml:id="S0.E2.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle y" text="y" xml:id="S0.E2.m1"><XMath><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle=2" text="absent = 2" xml:id="S0.E2.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation><equation xml:id="S0.E3"><tags><tag>(3)</tag><tag role="autoref">Equation 3</tag><tag role="refnum">3</tag></tags><MathFork><Math tex="\displaystyle z=3" text="z = 3" xml:id="S0.E3.m3"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">z</XMTok><XMTok meaning="3" role="NUMBER">3</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle z" text="z" xml:id="S0.E3.m1"><XMath><XMTok font="italic" role="UNKNOWN">z</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle=3" text="absent = 3" xml:id="S0.E3.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok meaning="3" role="NUMBER">3</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup><p><anchor xml:id="a"/>After a.</p></para>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><equation xml:id="S0.Ex1"><Math mode="display" tex="y\text{ if }z" text="y * [ if ] * z" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMText> if </XMText><XMTok font="italic" role="UNKNOWN">z</XMTok></XMApp></XMath></Math></equation><p><anchor xml:id="t"/>After t.</p></para>"##,
    ),
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><tabular vattach="middle"><tbody><tr><td align="center">a</td><td align="center">b</td></tr><tr><td align="center">c</td><td align="center">d</td></tr></tbody></tabular><p><anchor xml:id="b"/></p></para>"##,
    ),
  ]);
}

/// 59n: a tblr's outer `evaluate=`/`expand=` preprocess the body as tabularray does before it splits the
/// cells (`\__tblr_modify_table_body:`, tabularray.sty:3567-3575: the functional library's evaluation,
/// :8289-8306, `all` :8275-8277; each `expand=` macro once, :3579-3591; `expand+=` appends). The
/// binding read the table unprocessed, so a function returning rows that start with an empty cell ran
/// inside the first cell and its `&` met the cell's group ("Stray alignment": the tabularray manual's
/// last error, tabularray.tex:2859-2870). p4's body redefines its macros as it runs, so only an expansion
/// before the cells run gives 20/30, 50/60 (the kernel's own expansion gave X/Y, P/Q), with the default
/// `expand=\rowa` merged with the table's `expand+=\rowb`. Repro
/// alignment-bindings/tblr_evaluate_and_expand_preprocess_the_body.
#[test]
fn tblr_evaluate_and_expand_preprocess_the_body() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tblr_evaluate_and_expand_preprocess_the_body.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><tabular class="ltx_guessed_headers" vattach="middle"><tbody><tr><td/><td align="left">y</td><td align="left">y</td><td align="left">y</td></tr><tr><td thead="row"/><td align="left">y</td><td align="left">y</td><td align="left">y</td></tr><tr><td/><td align="left">y</td><td align="left">y</td><td align="left">y</td></tr></tbody></tabular></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="center">AA</td><td align="center">BB</td><td align="center">CC</td></tr><tr><td align="center">20</td><td align="center">30</td><td align="center">40</td></tr><tr><td align="center">50</td><td align="center">60</td><td align="center">70</td></tr><tr><td align="center">DD</td><td align="center">EE</td><td align="center">FF</td></tr></tbody></tabular></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><tabular vattach="middle"><tbody><tr><td align="left">Row1</td><td align="left">1</td></tr><tr><td align="left">All</td><td align="left">All</td></tr></tbody></tabular></para>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><tabular class="ltx_guessed_headers" vattach="middle"><thead><tr><td align="center" thead="column">a</td><td align="center" thead="column">b</td></tr></thead><tbody><tr><td align="center">20</td><td align="center">30</td></tr><tr><td align="center">50</td><td align="center">60</td></tr></tbody></tabular></para>"##,
    ),
  ]);
}

/// 59o: a g-brief letter's sender and addressee are its frontmatter (user ruling 2026-10-01). The classes print them
/// only in `\ps@firstpage`'s head and foot (g-brief2.cls:304-426, g-brief.cls:306-360), which LaTeXML never
/// typesets, so both engines lost every field (SHARED); the bindings hand them to the frontmatter at `\begin{g-brief}`:
/// a `creator` of role `sender` (name, then a contact per block, labelled as the class labels it; no label where the
/// class prints none) and one of role `addressee` (postal note, address), with the fields' values at the letter —
/// setters after it change nothing (the queue digests later). Blank fields leave no contact; g-brief.cls's bank
/// block needs bank, BLZ and account (its own condition); a return address only when one was set. The HTML renders
/// both roles in place, each label apart from its value. (`e--Mail:`: a label reaches the `name` attribute untypeset,
/// RED `sectioning-frontmatter/contact_name_label_keeps_its_ligatures`.) OXIDIZED_DESIGN_DIVERGENCES #412. Repros
/// sectioning-frontmatter/{gbrief2_letter_sender_and_addressee, gbrief_letter_sender_and_addressee,
/// gbrief_fields_are_read_at_the_letter}.
#[test]
fn gbrief_letter_sender_and_addressee_are_frontmatter() {
  for (tex, sender, addressee) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/gbrief2_letter_sender_and_addressee.tex"
      ),
      r##"<creator role="sender"><personname>Otto Raffzahn</personname><contact name="Adresse:" role="address">Wiesenacker 25a</contact><contact name="Telefon:" role="phone">+49 000 0000000</contact><contact name="Internet:" role="internet">raffzahn@example.com</contact></creator>"##,
      r##"<creator role="addressee"><contact role="address">Frau<break/>Else Mittellos</contact></creator>"##,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/gbrief_letter_sender_and_addressee.tex"
      ),
      r##"<creator role="sender"><personname>Otto Raffzahn</personname><contact role="address">Wiesenacker 25a<break/>D-99533 Weitewelt</contact><contact name="Telefon:" role="phone">+49 000 00000</contact><contact name="e--Mail:" role="email">raffzahn@example.com</contact><contact name="HTTP:" role="http">http://www.example.com</contact><contact name="Bankverbindung:" role="bank">Bankhaus Skrupellos<break/><text fontsize="82%">BLZ</text> 000.000.00<break/><text fontsize="82%">Kto.</text> 000.000.000</contact></creator>"##,
      r##"<creator role="addressee"><contact role="postal_note">EINSCHREIBEN</contact><contact role="address">Frau<break/>Else Mittellos</contact></creator>"##,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/gbrief_fields_are_read_at_the_letter.tex"
      ),
      r##"<creator role="sender"><personname>Otto</personname></creator>"##,
      r##"<creator role="addressee"><contact role="address">Frau Else</contact></creator>"##,
    ),
  ] {
    let xml = assert_elements(tex, RAW, (0, 0), &[]);
    assert_element(&xml, "creator", &[r#"role="sender""#], sender);
    assert_element(&xml, "creator", &[r#"role="addressee""#], addressee);
  }
  // The structure XSLT renders both roles in place (the plain `creator` template renders nothing outside a title).
  let (_, html) = super::perfect_kernel_batch46::convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/gbrief2_letter_sender_and_addressee.tex"
  ));
  assert_element(
    &html,
    "div",
    &[r#"class="ltx_creator ltx_role_sender""#],
    r##"<div class="ltx_creator ltx_role_sender">
<span class="ltx_personname">Otto Raffzahn
</span>
<span class="ltx_contact ltx_role_address"><span class="ltx_contact_name">Adresse:</span> Wiesenacker 25a</span>
<span class="ltx_contact ltx_role_phone"><span class="ltx_contact_name">Telefon:</span> +49 000 0000000</span>
<span class="ltx_contact ltx_role_internet"><span class="ltx_contact_name">Internet:</span> raffzahn@example.com</span>
</div>"##,
  );
  assert_element(
    &html,
    "div",
    &[r#"class="ltx_creator ltx_role_addressee""#],
    r##"<div class="ltx_creator ltx_role_addressee">
<span class="ltx_contact ltx_role_address">Frau
<br class="ltx_break">Else Mittellos</span>
</div>"##,
  );
}

/// 59p: a raw class's `\maketitle` body (replayed by `\lx@deposit@maketitle`) keeps its title-page content. The
/// replay gate no longer checks a `\newif` switch's dead branch (uantwerpencoursetext's
/// `\if@copyright\backgroundsetup{…}\fi`, uantwerpencoursetext.cls:474-477; `\unless` inverts it) nor the inside of a
/// `tikzpicture` (uantwerpenletter's `\path`, :285-292; the tikz binding marks it `replay_gate_scoped_vocabulary:`),
/// and eso-pic's one-shot overlay inside the replay is the title page's picture (uantwerpenexam.cls:282-401; user
/// ruling 2026-10-02, OXIDIZED_DESIGN_DIVERGENCES #413), its page positions `\put`s (lni.cls:544-553) and its
/// arguments checked by its own diagnostics hold, not the gate (the `\def` inside it). Repros
/// sectioning-frontmatter/{maketitle_replay_skips_a_dead_branch, maketitle_replay_reads_an_environment_as_a_unit,
/// title_page_overlay_is_the_page_picture, title_page_overlay_places_page_positions}.
#[test]
fn maketitle_replay_keeps_its_title_page_content() {
  for (tex, p1) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/maketitle_replay_skips_a_dead_branch.tex"
      ),
      r##"<para xml:id="p1"><p>CONFIDENTIAL AND PROPRIETARY.Body.</p></para>"##,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/maketitle_replay_reads_an_environment_as_a_unit.tex"
      ),
      r##"<para xml:id="p1"><p>CONFIDENTIAL AND PROPRIETARY.
<picture height="21.52" width="81.1" xml:id="p1.pic1"><svg:svg height="21.52" overflow="visible" version="1.1" viewBox="0 0 81.1 21.52" width="81.1"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,21.52) matrix(1 0 0 -1 0 0) translate(40.55,0) translate(0,10.76) matrix(1.0 0.0 0.0 1.0 -35.94 -3.46)"><svg:foreignObject height="12.3" overflow="visible" style="--ltx-fo-width:5.14em;--ltx-fo-height:0.69em;--ltx-fo-depth:0.19em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="71.11">Subject line</svg:foreignObject></svg:g></svg:svg></picture>Body.</p></para>"##,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_page_overlay_is_the_page_picture.tex"
      ),
      r##"<para xml:id="p1"><picture fill="none" height="795.0pt" stroke="none" unitlength="1.0pt" width="614.3pt" xml:id="p1.pic1"><g innerdepth="0.0pt" innerheight="0.0pt" transform="translate(0,0)"><picture fill="none" height="0.0pt" stroke="none" unitlength="1.0pt" width="0.0pt" xml:id="p1.pic1.pic1"><g innerdepth="1.9pt" innerheight="6.9pt" innerwidth="167.5pt" transform="translate(99.63,968.59)"><text>Course 5-Bistrologie, exam 2018-01-29</text></g></picture></g></picture><p>Extra info.Body.</p></para>"##,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_page_overlay_places_page_positions.tex"
      ),
      r##"<para xml:id="p1"><picture fill="none" height="795.0pt" stroke="none" unitlength="1.0pt" width="614.3pt" xml:id="p1.pic1"><g innerdepth="0.0pt" innerheight="6.9pt" innerwidth="37.9pt" transform="translate(0,0)"><g innerdepth="0.0pt" innerheight="6.9pt" innerwidth="37.9pt" transform="translate(99.63,99.63)"><text>DOI line</text></g></g></picture><p>Extra info.Body.</p></para>"##,
    ),
    (
      // a definition inside the overlay is the overlay's hold's to check (pdflatex 0 errors)
      "\\documentclass{article}\n\\usepackage{eso-pic}\n\\renewcommand\\maketitle{%\n  \\AddToShipoutPicture*{\\put(72,700){\\def\\lbl{Course}\\lbl}}%\n  Extra info.}\n\\title{T}\n\\begin{document}\n\\maketitle\nBody.\n\\end{document}\n",
      r##"<para xml:id="p1"><picture fill="none" height="795.0pt" stroke="none" unitlength="1.0pt" width="614.3pt" xml:id="p1.pic1"><g innerdepth="0.0pt" innerheight="6.8pt" innerwidth="30.1pt" transform="translate(99.63,968.59)"><text>Course</text></g></picture><p>Extra info.Body.</p></para>"##,
    ),
    (
      // `\unless` inverts the switch: the live branch is the shown one (pdflatex 0 errors, "SHOWN")
      "\\documentclass{article}\n\\newif\\ifcopyright\n\\renewcommand\\maketitle{\\unless\\ifcopyright SHOWN\\else\\undefinedthing\\fi}\n\\title{T}\n\\begin{document}\n\\maketitle\nBody.\n\\end{document}\n",
      r##"<para xml:id="p1"><p>SHOWNBody.</p></para>"##,
    ),
  ] {
    assert_elements(tex, RAW, (0, 0), &[("para", "p1", p1)]);
  }
}

/// 59p: a title-page deposit is kept only when it typesets content (`typesets_content`): a body that lays out only
/// the title fields — all in the frontmatter, emptied for the replay — adds no empty paragraph (KOMA, boek, rapport
/// title pages; a whatsit's string is its reversion, so `\noindent\par\null` read as text), and titling's
/// `\thetitle`/`\theauthor`/`\thedate` are emptied with `\@title` (uol-physics-report typeset the title twice).
/// Repros sectioning-frontmatter/{maketitle_replay_shows_nothing_from_an_emptied_title_page,
/// maketitle_replay_empties_titling_copies}.
#[test]
fn maketitle_replay_adds_nothing_the_frontmatter_has() {
  for tex in [
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/maketitle_replay_shows_nothing_from_an_emptied_title_page.tex"
    ),
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/maketitle_replay_empties_titling_copies.tex"
    ),
  ] {
    assert_elements(tex, RAW, (0, 0), &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Body.</p></para>"##,
    )]);
  }
}

/// 59p: a replay or overlay whose errors discard it (`DiagnosticsHold::discard`) takes back the stubs standing in for
/// them: the document's own later use of the command reports its error (it was an uncounted `<ERROR>`). A mechanism
/// probe (pdflatex reports both uses). The groups such a replay leaves open stay open (RED
/// sectioning-frontmatter/discarded_maketitle_replay_leaves_its_groups_open; closing them broke oegatb).
#[test]
fn discarded_replay_takes_back_its_stubs() {
  // the overlay's error is discarded with its stub: the body's own use is reported
  assert_elements_with(
    "\\documentclass{article}\n\\usepackage{eso-pic}\n\\renewcommand\\maketitle{%\n  \\AddToShipoutPicture*{\\put(72,700){\\undefinedthing}}%\n  Extra info.}\n\\title{T}\n\\begin{document}\n\\maketitle\nBody \\undefinedthing{} end.\n\\end{document}\n",
    RAW,
    (1, 0),
    &["The token T_CS[\\undefinedthing] is not defined."],
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Extra info.Body <ERROR class="undefined">\undefinedthing</ERROR> end.</p></para>"##,
    )],
  );
  // a class body that reaches an undefined internal through a defined helper (the gate checks the body's own
  // vocabulary): the replay is discarded with its stub
  assert_elements_with(
    "\\documentclass{article}\n\\def\\helper{\\undefinedthing}\n\\renewcommand\\maketitle{\\helper Extra info.}\n\\title{T}\n\\begin{document}\n\\maketitle\nBody \\undefinedthing{} end.\n\\end{document}\n",
    RAW,
    (1, 0),
    &["The token T_CS[\\undefinedthing] is not defined."],
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Body <ERROR class="undefined">\undefinedthing</ERROR> end.</p></para>"##,
    )],
  );
}
