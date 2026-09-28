//! Red/green guards for perfect-kernel phase-57 batches from 57ah on: the K13 stage-2 binding
//! audit's residue (bindings that read differently from their real macro, 57ah) and the math
//! follow-ups of the 57ag classification against Perl (57ai).
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// 57ah: amsart's `\authors`, `\shortauthors` and `\addresses` are parameterless storage macros
/// that `\author[#1]{#2}` accumulates (amscls/amsart.cls:460-477; the short form defaults to the
/// full name, `\@dblarg`, and an explicit `[]` adds none). The binding read an argument (Perl
/// ams_support.sty.ltxml:82-84, KPE #366), so `\authors` swallowed the next token — the text after
/// it, or a tabular's `\end` (2605.03453). amsart's `\maketitle` keeps `\and`, which the kernel title
/// code clears, so the names still read "A and B" after the title and a section.
/// Repro sectioning-frontmatter/ams_authors_are_storage_macros.
#[test]
fn ams_authors_are_storage_macros() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ams_authors_are_storage_macros.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let names = "Ann Author and Bob Writer and Cy Empty";
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    &format!(
      "<para xml:id=\"p1\"><p>Written by {names} (short: A.\u{a0}Author and B.\u{a0}Writer).\n\
       <tabular vattach=\"middle\"><tbody><tr><td align=\"center\">{names}</td></tr></tbody></tabular></p></para>"
    ),
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.p1""#],
    &format!("<para xml:id=\"S1.p1\"><p>Again: {names}.</p></para>"),
  );
  // amsart's `\maketitle` never clears `\and`, so a document's own `\and` joins the names after
  // the title too (57ah review: the class restored the kernel `\and` over it).
  let (stderr, xml) = convert_with(
    &include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ams_authors_are_storage_macros.tex"
    )
    .replace(r"\title{Storage}", r"\renewcommand{\and}{, }\title{Storage}"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.p1""#],
    r#"<para xml:id="S1.p1"><p>Again: Ann Author, Bob Writer, Cy Empty.</p></para>"#,
  );
}

/// 57ai: amsart's list joiners over its author storage (amsart.cls:580-598, :803-807):
/// `\author@andify\authors` gives "A, B, and C" and `\andify` "A and B". ams_support's
/// `\author@andify` was an inert stub and `\andify` undefined, so a derived class printing
/// `\authors` (resphilosophica.cls:323) ran the names together. Repro
/// sectioning-frontmatter/ams_author_andify_joins_the_names.
#[test]
fn ams_author_andify_joins_the_names() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ams_author_andify_joins_the_names.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>Joined: Ann, Bob, and\u{a0}Cy. Pair: Ann and\u{a0}Bob.</p></para>",
  );
}

/// 57ah: varioref's macros read optionals — `\vref*[text]{l}`, `\vpageref*[here][far]{l}`,
/// `\vrefrange[here]{a}{b}`, `\vpagerefrange*[here]{a}{b}` — and `\fullref`/`\reftextfaraway`
/// read a label (tools/varioref.sty:803-966, :123-125). The binding read none (Perl
/// varioref.sty.ltxml:24-27, KPE #367): `[here]` became the label `[`, the label printed as text,
/// and the two-parameter range texts used `#2`/`#3`. The page text stays dropped (no pages).
/// Repro singletons/varioref_reads_its_optionals.
#[test]
fn varioref_reads_its_optionals() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/varioref_reads_its_optionals.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let r = r#"<ref labelref="LABEL:sec:a"/>"#;
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.p1""#],
    &format!(
      "<para xml:id=\"S1.p1\"><p>See {r}, {r},\n{r}\u{2013}{r}, {r} and .\n\
       Also {r}, {r}, {r}\u{2013}{r} and {r}\u{2013}{r}.</p></para>"
    ),
  );
}

/// 57ah: attachfile loads hyperref (`\RequirePackageWithOptions{hyperref}`, attachfile.sty:40);
/// the binding did not (Perl attachfile.sty.ltxml:19-22, KPE #368), so a document loading only
/// attachfile had no `\href` or `\autoref`. Repro singletons/attachfile_loads_hyperref.
#[test]
fn attachfile_loads_hyperref() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/singletons/attachfile_loads_hyperref.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.p1""#],
    concat!(
      r#"<para xml:id="S1.p1"><p>See <ref class="ltx_refmacro_autoref" labelref="LABEL:sec:a" show="autoref"/>"#,
      r#" and <ref class="ltx_href" href="https://example.org">the site</ref>.</p></para>"#
    ),
  );
}

/// 57ah: svn-multi loads raw. The contrib stub made `\ifsvnmodified` a TeX conditional (real: a
/// two-argument chooser, svn-multi.sty:256), gave the storage macro `\svnurl` an argument (:269)
/// and made `\svnidlong` a no-op, so the keyword groups' `$…$` became math and the revision line
/// took `Error:expected:\fi`. Repro singletons/svn_multi_loads_raw.
#[test]
fn svn_multi_loads_raw() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/singletons/svn_multi_loads_raw.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<Math"), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Revision -2 by  (clean); url [].</p></para>"#,
  );
  // The document keywords are pdflatex's first pass (pass 2 reads them from the .aux); a file's
  // `\svnid` sets its keywords at once.
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>File 7 by bob.</p></para>"#,
  );
}

/// 57ah: savetrees processes its options. Under `bibnotes=tight` (the default; `subtle` clears
/// it) `\savetreesbibnote` drops the note and the token after it (savetrees.sty:342-346); the
/// binding read no option and always printed the note. Repro
/// singletons/savetrees_reads_its_options, default and `[subtle]`.
#[test]
fn savetrees_reads_its_options() {
  let source =
    include_str!("../../../tools/perfect_kernel/repros/singletons/savetrees_reads_its_options.tex");
  for (options, text) in [("", "2020."), ("[subtle]", "2020. The note.")] {
    let (stderr, xml) = convert_with(
      &source.replace(r"\usepackage{", &format!(r"\usepackage{options}{{")),
      None,
    );
    assert_eq!(error_count(&stderr), 0, "{options}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{options}: {stderr}");
    latexml::util::test::assert_element(
      &xml,
      "bibblock",
      &[],
      &format!(r#"<bibblock> A. Author, <emph font="italic">Title</emph>, {text}</bibblock>"#),
    );
  }
}

/// 57ah: `\FloatBarrier` begins with `\par` (placeins.sty:30); the binding made it a no-op (Perl
/// placeins.sty.ltxml:24, KPE #369), so the text around it ran together. Repro
/// singletons/floatbarrier_ends_the_paragraph.
#[test]
fn floatbarrier_ends_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/floatbarrier_ends_the_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>First paragraph text.</p></para>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>Second paragraph text.</p></para>"#,
  );
}

/// 57ah: the kernel's case changers read `O{} +m` (latex.ltx:22367-22378; the optional is the
/// locale keys). The engine's read `[1]` (Perl latex_constructs.pool.ltxml:5914-5935, KPE #370),
/// so `[lang=en]` was the argument. Repro singletons/case_changers_read_their_locale.
#[test]
fn case_changers_read_their_locale() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/case_changers_read_their_locale.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>A WORD B word C Word\nD WORD.</p></para>",
  );
  // The fronts re-brace what they read: `\MakeUppercase\foo` takes `\foo` whole, also inside
  // another case changer and a `\protected@edef` (latex.ltx:22367-22378); the inner command is
  // e-TeX protected, so a plain `\edef` keeps the call (57ah re-review: 94 errors).
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>E x ABC y F abcZ G ABC. H ABC.</p></para>"#,
  );
}

/// 57ah: `\listoftodos[1][…]` reads its heading (todonotes.sty:323); the binding read nothing
/// (Perl todonotes.sty.ltxml:38, KPE #372), so the heading was printed as "[My Notes]". Repro
/// singletons/listoftodos_reads_its_heading.
#[test]
fn listoftodos_reads_its_heading() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/listoftodos_reads_its_heading.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>C.</p></para>"#,
  );
}

/// 57ah: every supertabular variant reads a `[pos]` it does not use (supertabular.sty:352-400);
/// the binding read none (Perl supertabular.sty.ltxml:24, :40, :60, :70; KPE #371), so `[t]`
/// became the column template and each row took two `Extra alignment tab` errors. Repro
/// alignment-bindings/supertabular_reads_its_position.
#[test]
fn supertabular_reads_its_position() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/supertabular_reads_its_position.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for id in ["tab1", "tab2"] {
    latexml::util::test::assert_element(
      &xml,
      "table",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        r#"<table xml:id="{id}"><tabular><tr><td align="left">a</td><td align="left">b</td></tr></tabular></table>"#
      ),
    );
  }
}

/// 57ai: a comma list left of a relation is the ruled distributed dual, whatever the right side:
/// Perl's Formulae rules never build a list holding a relation (MathGrammar:141-170), and the
/// root pragma dropped only a list ENDING in one, so `a_1,b_2\vdash c,d` kept
/// `list@(a_1, b_2⊢c, d)` (57ag classification R1; 2605.14476). The presentation is Perl's
/// relation over the list. Repro math-parse/sequent_list_keeps_the_relation_outside.
#[test]
fn sequent_list_keeps_the_relation_outside() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/sequent_list_keeps_the_relation_outside.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for math in [
    r##"<Math mode="inline" tex="a_{1},b_{2}\vdash c,d" text="formulae@(a _ 1 proves list@(c, d), b _ 2 proves list@(c, d))" xml:id="p1.m1"><XMath><XMDual><XMApp><XMTok meaning="formulae"/><XMApp><XMRef idref="p1.m1.1"/><XMRef idref="p1.m1.2"/><XMRef idref="p1.m1.4"/></XMApp><XMApp><XMRef idref="p1.m1.1"/><XMRef idref="p1.m1.3"/><XMRef idref="p1.m1.4"/></XMApp></XMApp><XMApp><XMTok meaning="proves" name="vdash" role="METARELOP" xml:id="p1.m1.1">⊢</XMTok><XMWrap><XMApp xml:id="p1.m1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="PUNCT">,</XMTok><XMApp xml:id="p1.m1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMWrap><XMDual xml:id="p1.m1.4"><XMApp><XMTok meaning="list"/><XMRef idref="p1.m1.5"/><XMRef idref="p1.m1.6"/></XMApp><XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.5">c</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.6">d</XMTok></XMWrap></XMDual></XMApp></XMDual></XMath></Math>"##,
    r##"<Math mode="inline" tex="S_{1},S_{2}\vdash A,B,C" text="formulae@(S _ 1 proves list@(A, B, C), S _ 2 proves list@(A, B, C))" xml:id="p1.m2"><XMath><XMDual><XMApp><XMTok meaning="formulae"/><XMApp><XMRef idref="p1.m2.1"/><XMRef idref="p1.m2.2"/><XMRef idref="p1.m2.4"/></XMApp><XMApp><XMRef idref="p1.m2.1"/><XMRef idref="p1.m2.3"/><XMRef idref="p1.m2.4"/></XMApp></XMApp><XMApp><XMTok meaning="proves" name="vdash" role="METARELOP" xml:id="p1.m2.1">⊢</XMTok><XMWrap><XMApp xml:id="p1.m2.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">S</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="PUNCT">,</XMTok><XMApp xml:id="p1.m2.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">S</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMWrap><XMDual xml:id="p1.m2.4"><XMApp><XMTok meaning="list"/><XMRef idref="p1.m2.5"/><XMRef idref="p1.m2.6"/><XMRef idref="p1.m2.7"/></XMApp><XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.5">A</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.6">B</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.7">C</XMTok></XMWrap></XMDual></XMApp></XMDual></XMath></Math>"##,
    r##"<Math mode="inline" tex="a_{1},b_{2}\leq c" text="formulae@(a _ 1 &lt;= c, b _ 2 &lt;= c)" xml:id="p1.m3"><XMath><XMDual><XMApp><XMTok meaning="formulae"/><XMApp><XMRef idref="p1.m3.1"/><XMRef idref="p1.m3.2"/><XMRef idref="p1.m3.4"/></XMApp><XMApp><XMRef idref="p1.m3.1"/><XMRef idref="p1.m3.3"/><XMRef idref="p1.m3.4"/></XMApp></XMApp><XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP" xml:id="p1.m3.1">≤</XMTok><XMWrap><XMApp xml:id="p1.m3.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="PUNCT">,</XMTok><XMApp xml:id="p1.m3.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="p1.m3.4">c</XMTok></XMApp></XMDual></XMath></Math>"##,
  ] {
    let id = math
      .split("xml:id=\"")
      .nth(1)
      .and_then(|s| s.split('"').next())
      .unwrap();
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57ai: a scripted operator nests over a following function (Perl `OPERATOR addScripts
/// nestOperators`, MathGrammar:312-313, :663-671): `\nabla_x\log p(y)` is ((∇_x)@(log))@(p)·y, not
/// ∇_x·log·p·y, and `\nabla_x\sin y` parses; taking no argument it multiplies a big operator's
/// application (`\nabla_x\log\det(A)` is (∇_x)@(log)·det(A): 2605.03984, 2605.24401, 2605.25592,
/// 2605.14289); the inline formulas are Perl's (57ag classification R2b). Repro
/// math-parse/scripted_operator_nests_over_a_function.
#[test]
fn scripted_operator_nests_over_a_function() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/scripted_operator_nests_over_a_function.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for math in [
    r##"<Math mode="inline" tex="\nabla_{x}\log p(y)" text="((nabla _ x)@(logarithm))@(p) * y" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">y</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla^{2}\exp x" text="((nabla ^ 2)@(exponential))@(x)" xml:id="p1.m2"><XMath><XMApp><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok meaning="exponential" role="OPFUNCTION">exp</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}\sin y" text="((nabla _ x)@(sine))@(y)" xml:id="p1.m3"><XMath><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="sine" role="TRIGFUNCTION">sin</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}\log p(y\mid x)\approx 0" text="((nabla _ x)@(logarithm))@(p) * conditional@(y, x) approximately-equals 0" xml:id="p1.m4"><XMath><XMApp><XMTok meaning="approximately-equals" name="approx" role="RELOP">≈</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp><XMDual><XMApp><XMTok meaning="conditional"/><XMRef idref="p1.m4.1"/><XMRef idref="p1.m4.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m4.1">y</XMTok><XMTok name="mid" role="MIDDLE">∣</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m4.2">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="0" role="NUMBER">0</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\log p(y)" text="(nabla@(logarithm))@(p) * y" xml:id="p1.m5"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp><XMDual><XMRef idref="p1.m5.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m5.1">y</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}\log\det(A)" text="(nabla _ x)@(logarithm) * determinant@(A)" xml:id="p1.m6"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok meaning="determinant" role="LIMITOP" scriptpos="post">det</XMTok><XMDual><XMRef idref="p1.m6.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m6.1">A</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\operatorname*{arg\,max}_{s}\log\det(L_{s})" text="((arg * max) _ s)@(logarithm) * determinant@(L _ s)" xml:id="p1.m7"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMApp role="OPERATOR" scriptpos="mid"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok role="UNKNOWN" rpadding="1.7pt">arg</XMTok><XMTok role="UNKNOWN">max</XMTok></XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">s</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok meaning="determinant" role="LIMITOP" scriptpos="post">det</XMTok><XMDual><XMRef idref="p1.m7.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m7.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">L</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">s</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="display" tex="\begin{split}\nabla_{x}\log p(y)\end{split}" text="((nabla _ x)@(logarithm))@(p) * y" xml:id="S0.Ex1.m1"><XMath><XMDual><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post6"/><XMRef idref="S0.Ex1.m1.1"/><XMRef idref="S0.Ex1.m1.2"/></XMApp><XMRef idref="S0.Ex1.m1.3"/></XMApp><XMRef idref="S0.Ex1.m1.4"/></XMApp><XMRef idref="S0.Ex1.m1.6"/></XMApp><XMArray colsep="0pt" name="aligned"><XMRow><XMCell align="right"><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post6"/><XMTok name="nabla" role="OPERATOR" xml:id="S0.Ex1.m1.1">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.Ex1.m1.2">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION" xml:id="S0.Ex1.m1.3">log</XMTok></XMApp><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.4">p</XMTok></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.6">y</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math>"##,
  ] {
    let id = math
      .split("xml:id=\"")
      .nth(1)
      .and_then(|s| s.split('"').next())
      .unwrap();
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57aj: relsize's `\mathlarger`/`\mathsmaller` read their atom, gather the scripts after it and
/// size only that (relsize.sty:263-310); the binding's open `\relsize{±1}` (Perl
/// relsize.sty.ltxml:45-46, KPE #373) sized the rest of the formula too. Repro
/// fonts-nfss/mathlarger_sizes_only_its_atom.
#[test]
fn mathlarger_sizes_only_its_atom() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/mathlarger_sizes_only_its_atom.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for math in [
    r##"<Math mode="inline" tex="{\sum_{i}}x_{i}+y" text="(sum _ i)@(x _ i) + y" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post2"/><XMTok fontsize="120%" mathstyle="text" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok><XMTok font="italic" fontsize="84%" role="UNKNOWN">i</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="a{b}c" text="a * b * c" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" fontsize="83%" role="UNKNOWN">b</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math>"##,
  ] {
    let id = math
      .split("xml:id=\"")
      .nth(1)
      .and_then(|s| s.split('"').next())
      .unwrap();
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57aj: caption's `\DeclareCaptionOption` defines its key with keyval (caption3.sty:209, :221-236;
/// the star only undefines it at the end of the declaring package) and `\captionsetup` runs it
/// (:244-259); a typed `\captionsetup[figure]` only stores them (:252-262). The binding loaded no
/// keyval, its star branch gobbled its own helper, and `\captionsetup` only stored the keys (Perl:
/// keys not run, KPE #374). Repro
/// captions-floats/declared_caption_option_runs_its_code.
#[test]
fn declared_caption_option_runs_its_code() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/declared_caption_option_runs_its_code.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Values: yes, bee. Typed: unset.</p></para>"##,
  );
}

/// 57aj: siunitx's `\SI[opts]{n}[pre]{u}` prints the pre-unit before the quantity
/// (siunitx.sty:9535 `O{} m o m`); the binding read none (Perl siunitx.sty.ltxml:1150, KPE #375), so
/// `[\$]` was taken for the unit. Repro singletons/siunitx_si_reads_its_pre_unit.
#[test]
fn siunitx_si_reads_its_pre_unit() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/siunitx_si_reads_its_pre_unit.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for math in [
    r##"<Math mode="inline" tex="\$10\text{\,}{\mathrm{kg}}^{-1}" text="currency-dollar@(10 * power@(kilogram, - 1))" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="currency-dollar" role="OPERATOR">$</XMTok><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m1.1"> </XMText><XMTok meaning="10" role="NUMBER">10</XMTok><XMApp xml:id="p1.m1.3"><XMTok meaning="power" role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok class="ltx_unit" meaning="kilogram" role="ID">kg</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="5\text{\,}\mathrm{m}" text="5 * meter" xml:id="p1.m2"><XMath><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m2.1"> </XMText><XMTok meaning="5" role="NUMBER">5</XMTok><XMTok class="ltx_unit" meaning="meter" role="ID">m</XMTok></XMApp></XMath></Math>"##,
  ] {
    let id = math
      .split("xml:id=\"")
      .nth(1)
      .and_then(|s| s.split('"').next())
      .unwrap();
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57aj: a unit's qualifier (by `qualifier-mode`) and `\highlight` colour apply, as Perl's
/// `six_format_1unit` (siunitx.sty.ltxml:976-1007, :1373-1374): `gram-polymer` and a red m, Perl's
/// element for element (RUST-ONLY: the port read neither). Repro
/// singletons/siunitx_qualifier_and_highlight_apply.
#[test]
fn siunitx_qualifier_and_highlight_apply() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/siunitx_qualifier_and_highlight_apply.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for math in [
    r##"<Math mode="inline" tex="\mathrm{g}_{\mathrm{pol}}" text="gram-polymer" xml:id="p1.m1"><XMath><XMApp class="ltx_unit" meaning="gram-polymer" role="ID"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok role="UNKNOWN">g</XMTok><XMTok fontsize="70%" role="UNKNOWN">pol</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="{\color[rgb]{1,0,0}\mathrm{m}}" text="meter" xml:id="p1.m2"><XMath><XMTok class="ltx_unit" color="#FF0000" meaning="meter" role="ID">m</XMTok></XMath></Math>"##,
  ] {
    let id = math
      .split("xml:id=\"")
      .nth(1)
      .and_then(|s| s.split('"').next())
      .unwrap();
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57ak: Perl's OPERATOR is a Factor (MathGrammar:312-313) applying to `APPLYOP(?) barearg`
/// (:553-558), a left-associative chain of `aBarearg`s joined by juxtaposition or a MulOp
/// (:321-337) — no fence but `|…|`, no operator or big operator, no `f(x)` — and it applies
/// mid-term too; the grammar had only `operator factor` at a term's start and pruned the rest
/// (unparsed `\nabla u\cdot v`, `\eta\nabla L(\theta)`; witnesses 2605.19037, 2605.06657,
/// 2605.25194, 2605.02202). Repro math-parse/operator_takes_a_bare_argument.
#[test]
fn operator_takes_a_bare_argument() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/operator_takes_a_bare_argument.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for math in [
    r##"<Math mode="inline" tex="\nabla u\cdot v" text="nabla@(u cdot v)" xml:id="p1.m1"><XMath><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla uv" text="nabla@(u * v)" xml:id="p1.m2"><XMath><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla u+v" text="nabla@(u) + v" xml:id="p1.m3"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}\log p(y)" text="((nabla _ x)@(logarithm))@(p) * y" xml:id="p1.m4"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp><XMDual><XMRef idref="p1.m4.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m4.1">y</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="a\cdot\nabla u" text="a cdot nabla@(u)" xml:id="p1.m5"><XMath><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla u\cdot vw" text="nabla@((u cdot v) * w)" xml:id="p1.m6"><XMath><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">w</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla u\cdot\nabla v" text="nabla@(u) cdot nabla@(v)" xml:id="p1.m7"><XMath><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla f(x)\cdot d" text="(nabla@(f) * x) cdot d" xml:id="p1.m8"><XMath><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp><XMDual><XMRef idref="p1.m8.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m8.1">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla(u)\cdot v" text="nabla@(u) cdot v" xml:id="p1.m9"><XMath><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMDual><XMRef idref="p1.m9.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m9.1">u</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\eta\nabla L(\theta)" text="eta * nabla@(L) * theta" xml:id="p1.m10"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="eta" role="UNKNOWN">η</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">L</XMTok></XMApp><XMDual><XMRef idref="p1.m10.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" name="theta" role="UNKNOWN" xml:id="p1.m10.1">θ</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla|u|v" text="nabla@(absolute-value@(u) * v)" xml:id="p1.m11"><XMath><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m11.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m11.1">u</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla^{2}u\cdot v" text="(nabla ^ 2)@(u cdot v)" xml:id="p1.m12"><XMath><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\log p\cdot v" text="(nabla@(logarithm))@(p cdot v)" xml:id="p1.m13"><XMath><XMApp><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\phi\left|\nabla\phi\right|" text="nabla@(phi * absolute-value@(nabla@(phi)))" xml:id="p1.m14"><XMath><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="phi" role="UNKNOWN">ϕ</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m14.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="true">|</XMTok><XMApp xml:id="p1.m14.1"><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" name="phi" role="UNKNOWN">ϕ</XMTok></XMApp><XMTok role="CLOSE" stretchy="true">|</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla u\left|v\right|" text="nabla@(u * absolute-value@(v))" xml:id="p1.m15"><XMath><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m15.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="true">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m15.1">v</XMTok><XMTok role="CLOSE" stretchy="true">|</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla u\lvert v\rvert" text="nabla@(u) * absolute-value@(v)" xml:id="p1.m16"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m16.1"/></XMApp><XMWrap><XMTok name="lvert" role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m16.1">v</XMTok><XMTok name="rvert" role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla u\|v\|w" text="nabla@(u * norm@(v) * w)" xml:id="p1.m17"><XMath><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m17.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m17.1">v</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">w</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla u||v||" text="nabla@(u) * norm@(v)" xml:id="p1.m18"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m18.1"/></XMApp><XMWrap><XMTok role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m18.1">v</XMTok><XMTok role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla u||v||+1" text="nabla@(u) * norm@(v) + 1" xml:id="p1.m19"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m19.1"/></XMApp><XMWrap><XMTok role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m19.1">v</XMTok><XMTok role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\eta\nabla L||w||^{2}" text="eta * nabla@(L) * (norm@(w)) ^ 2" xml:id="p1.m20"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="eta" role="UNKNOWN">η</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m20.1"/></XMApp><XMWrap><XMTok role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m20.1">w</XMTok><XMTok role="CLOSE">‖</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="display" tex="\begin{split}\nabla u\cdot v\end{split}" text="nabla@(u cdot v)" xml:id="S0.Ex1.m1"><XMath><XMDual><XMApp><XMRef idref="S0.Ex1.m1.1"/><XMApp><XMRef idref="S0.Ex1.m1.2"/><XMRef idref="S0.Ex1.m1.3"/><XMRef idref="S0.Ex1.m1.4"/></XMApp></XMApp><XMArray colsep="0pt" name="aligned"><XMRow><XMCell align="right"><XMApp><XMTok name="nabla" role="OPERATOR" xml:id="S0.Ex1.m1.1">∇</XMTok><XMApp><XMTok name="cdot" role="MULOP" xml:id="S0.Ex1.m1.2">⋅</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.3">u</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.4">v</XMTok></XMApp></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math>"##,
  ] {
    let id = math
      .split("xml:id=\"")
      .nth(1)
      .and_then(|s| s.split('"').next())
      .unwrap();
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57ak: Perl's `nestOperators` (MathGrammar:663-671) nests operators until a function, scripted
/// ones too, and an operator taking no argument is a Factor of its own: `\nabla\nabla f` is
/// (∇@∇)@(f), `\nabla_x f^2` (∇_x)@(f²), `(u\cdot\nabla)u` (u·∇)·u; a closed nest multiplies an
/// operator or big operator after it and stands mid-term (`\eta\nabla_\theta\log\det(A)` is
/// η·(∇_θ)@(log)·det(A), the 57ak review rows). Repro math-parse/operator_nests_over_an_operator.
#[test]
fn operator_nests_over_an_operator() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/operator_nests_over_an_operator.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for math in [
    r##"<Math mode="inline" tex="\nabla\nabla f" text="(nabla@(nabla))@(f)" xml:id="p1.m1"><XMath><XMApp><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\nabla^{2}u" text="(nabla@(nabla ^ 2))@(u)" xml:id="p1.m2"><XMath><XMApp><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}f^{2}" text="(nabla _ x)@(f ^ 2)" xml:id="p1.m3"><XMath><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">f</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}\sin^{2}x" text="((nabla _ x)@(sine ^ 2))@(x)" xml:id="p1.m4"><XMath><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok meaning="sine" role="TRIGFUNCTION">sin</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="(u\cdot\nabla)u" text="(u cdot nabla) * u" xml:id="p1.m5"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMRef idref="p1.m5.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m5.1"><XMTok name="cdot" role="MULOP">⋅</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="a\nabla" text="a * nabla" xml:id="p1.m6"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\times\nabla\times u" text="nabla * nabla * u" xml:id="p1.m7"><XMath><XMApp><XMTok meaning="times" role="MULOP">×</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\nabla\cdot u" text="nabla@(nabla) cdot u" xml:id="p1.m8"><XMath><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}^{2}u" text="((nabla _ x) ^ 2)@(u)" xml:id="p1.m9"><XMath><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}^{2}\log p" text="(((nabla _ x) ^ 2)@(logarithm))@(p)" xml:id="p1.m10"><XMath><XMApp><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\nabla\log\sum_{i}p_{i}" text="nabla@(nabla@(logarithm)) * (sum _ i)@(p _ i)" xml:id="p2.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok mathstyle="text" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}^{2}\log\sum_{i}p_{i}" text="((nabla _ x) ^ 2)@(logarithm) * (sum _ i)@(p _ i)" xml:id="p2.m2"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok mathstyle="text" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\eta\nabla_{\theta}\log\det(A)" text="eta * (nabla _ theta)@(logarithm) * determinant@(A)" xml:id="p2.m3"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="eta" role="UNKNOWN">η</XMTok><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok meaning="determinant" role="LIMITOP" scriptpos="post">det</XMTok><XMDual><XMRef idref="p2.m3.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p2.m3.1">A</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\log\max_{i}p_{i}" text="(nabla@(logarithm))@((maximum _ i)@(p _ i))" xml:id="p2.m4"><XMath><XMApp><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok meaning="maximum" role="OPFUNCTION" scriptpos="post">max</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla\log\nabla^{2}u" text="nabla@(logarithm) * (nabla ^ 2)@(u)" xml:id="p2.m5"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="k\nabla\sin\int f" text="k * nabla@(sine) * integral@(f)" xml:id="p2.m6"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">k</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok meaning="sine" role="TRIGFUNCTION">sin</XMTok></XMApp><XMApp><XMTok mathstyle="text" meaning="integral" name="int" role="INTOP">∫</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="u\nabla\nabla\cdot v" text="(u * nabla@(nabla)) cdot v" xml:id="p2.m7"><XMath><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla_{x}\nabla_{y}u" text="((nabla _ x)@(nabla _ y))@(u)" xml:id="p2.m8"><XMath><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">y</XMTok></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla^{2}!" text="(nabla ^ 2)factorial" xml:id="p2.m9"><XMath><XMApp><XMTok meaning="factorial" role="POSTFIX">!</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\nabla^{2}|_{x=0}" text="evaluated-at@(nabla ^ 2, x = 0)" xml:id="p2.m10"><XMath><XMDual><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="p2.m10.1"/><XMRef idref="p2.m10.2"/></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMApp xml:id="p2.m10.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMApp xml:id="p2.m10.2"><XMTok fontsize="70%" meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="0" role="NUMBER">0</XMTok></XMApp></XMApp></XMDual></XMath></Math>"##,
    r##"<Math mode="inline" tex="\log\nabla^{2}" text="logarithm * nabla ^ 2" xml:id="p2.m11"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math>"##,
    r##"<Math mode="inline" tex="\sin\nabla" text="sine * nabla" xml:id="p2.m12"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="sine" role="TRIGFUNCTION">sin</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok></XMApp></XMath></Math>"##,
  ] {
    let id = math
      .split("xml:id=\"")
      .nth(1)
      .and_then(|s| s.split('"').next())
      .unwrap();
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57ak: the hybrid route's tree-iterator fallback stopped after 16 trees in a row adding no new
/// parse, and libmarpa varies the root choice — how the bars pair up — last, so `|…| = a|…| \le c`
/// read its bars as `conditional` bars where Perl and the ASF route pair them (2605.13278 A4.Ex64,
/// the 57aj A/B). Both routes read Perl's pairing. Repro
/// math-parse/bars_pair_across_the_whole_formula.
#[test]
fn bars_pair_across_the_whole_formula() {
  // The tree-iterator fallback (a limit every bocage exceeds), then pure ASF (no limit).
  for limit in [Some(1), None] {
    let (stderr, xml, ()) = super::perfect_kernel_batch46::convert_with_setup_then(
      include_str!(
        "../../../tools/perfect_kernel/repros/math-parse/bars_pair_across_the_whole_formula.tex"
      ),
      None,
      move || latexml_math_parser::set_hybrid_and_node_limit_override(Some(limit)),
      |_| (),
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // The `|…|` formula's fallback forest is large (43 trees), so that route reports it as
    // `ambiguous_math`; ASF reads both formulas without one.
    assert_eq!(
      warning_count(&stderr),
      if limit.is_some() { 1 } else { 0 },
      "limit {limit:?}: {stderr}"
    );
    assert_eq!(
      stderr.matches("Warning:ambiguous_math:").count(),
      warning_count(&stderr),
      "{stderr}"
    );
    for math in [
      r##"<Math mode="inline" tex="|\log x-\log y|=a|u|\leq c" text="absolute-value@(logarithm@(x) - logarithm@(y)) = a * absolute-value@(u) &lt;= c" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="multirelation"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="p1.m1.1"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.2">u</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math>"##,
      r##"<Math mode="inline" tex="\|\nabla_{x}\log\pi(x)-\nabla_{x}\log\rho(x)\|=a\|\mu-u\|\leq c" text="norm@(((nabla _ x)@(logarithm))@(pi) * x - ((nabla _ x)@(logarithm))@(rho) * x) = a * norm@(mu - u) &lt;= c" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="multirelation"/><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m2.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMApp xml:id="p1.m2.1"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok></XMApp><XMDual><XMRef idref="p1.m2.2"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.2">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMTok font="italic" name="rho" role="UNKNOWN">ρ</XMTok></XMApp><XMDual><XMRef idref="p1.m2.3"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.3">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m2.4"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMApp xml:id="p1.m2.4"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok font="italic" name="mu" role="UNKNOWN">μ</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math>"##,
    ] {
      let id = math
        .split("xml:id=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap();
      latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
    }
  }
}

/// 57ak re-review: sums of operator terms lost every parse on the hybrid route's tree-iterator
/// fallback — each term's derivations pruned only after the tree was built multiplied across the
/// sum (7, 49, 343, 2,401 trees for 1-4 terms). An argument-less operator is a `bare_op_term`, no
/// factor follows it, and a bare argument's items are built from bare shapes: one derivation per
/// term. Both routes read Perl's text. Repro math-parse/operator_terms_in_a_long_sum.
#[test]
fn operator_terms_in_a_long_sum() {
  // The tree-iterator fallback (a limit every bocage exceeds), then pure ASF (no limit).
  for limit in [Some(1), None] {
    let (stderr, xml, ()) = super::perfect_kernel_batch46::convert_with_setup_then(
      include_str!(
        "../../../tools/perfect_kernel/repros/math-parse/operator_terms_in_a_long_sum.tex"
      ),
      None,
      move || latexml_math_parser::set_hybrid_and_node_limit_override(Some(limit)),
      |_| (),
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // The fallback still enumerates the second sum's few pruned readings (`ambiguous_math`).
    assert_eq!(
      warning_count(&stderr),
      if limit.is_some() { 1 } else { 0 },
      "limit {limit:?}: {stderr}"
    );
    assert_eq!(
      stderr.matches("Warning:ambiguous_math:").count(),
      warning_count(&stderr),
      "{stderr}"
    );
    for math in [
      r##"<Math mode="inline" tex="g=\nabla_{\theta}\log p_{\theta}(x_{1})+\nabla_{\theta}\log p_{\theta}(x_{2})+\nabla_{\theta}\log p_{\theta}(x_{3})+\nabla_{\theta}\log p_{\theta}(x_{4})+\nabla_{\theta}\log p_{\theta}(x_{5})" text="g = ((nabla _ theta)@(logarithm))@(p _ theta) * x _ 1 + ((nabla _ theta)@(logarithm))@(p _ theta) * x _ 2 + ((nabla _ theta)@(logarithm))@(p _ theta) * x _ 3 + ((nabla _ theta)@(logarithm))@(p _ theta) * x _ 4 + ((nabla _ theta)@(logarithm))@(p _ theta) * x _ 5" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">g</XMTok><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMRef idref="p1.m1.2"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMRef idref="p1.m1.3"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMRef idref="p1.m1.4"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m1.4"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="4" role="NUMBER">4</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMRef idref="p1.m1.5"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m1.5"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="5" role="NUMBER">5</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMApp></XMath></Math>"##,
      r##"<Math mode="inline" tex="\eta\nabla_{\theta}\log\pi_{\theta}(a_{1}|s_{1})A_{1}+\eta\nabla_{\theta}\log\pi_{\theta}(a_{2}|s_{2})A_{2}+\eta\nabla_{\theta}\log\pi_{\theta}(a_{3}|s_{3})A_{3}+\eta\nabla_{\theta}\log\pi_{\theta}(a_{4}|s_{4})A_{4}" text="eta * ((nabla _ theta)@(logarithm))@(pi _ theta) * conditional@(a _ 1, s _ 1) * A _ 1 + eta * ((nabla _ theta)@(logarithm))@(pi _ theta) * conditional@(a _ 2, s _ 2) * A _ 2 + eta * ((nabla _ theta)@(logarithm))@(pi _ theta) * conditional@(a _ 3, s _ 3) * A _ 3 + eta * ((nabla _ theta)@(logarithm))@(pi _ theta) * conditional@(a _ 4, s _ 4) * A _ 4" xml:id="p2.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="eta" role="UNKNOWN">η</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="conditional"/><XMRef idref="p2.m1.1"/><XMRef idref="p2.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="MIDDLE" stretchy="false">|</XMTok><XMApp xml:id="p2.m1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">s</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">A</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="eta" role="UNKNOWN">η</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="conditional"/><XMRef idref="p2.m1.3"/><XMRef idref="p2.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="MIDDLE" stretchy="false">|</XMTok><XMApp xml:id="p2.m1.4"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">s</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">A</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="eta" role="UNKNOWN">η</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="conditional"/><XMRef idref="p2.m1.5"/><XMRef idref="p2.m1.6"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.5"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp><XMTok role="MIDDLE" stretchy="false">|</XMTok><XMApp xml:id="p2.m1.6"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">s</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">A</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="eta" role="UNKNOWN">η</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="conditional"/><XMRef idref="p2.m1.7"/><XMRef idref="p2.m1.8"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.7"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok fontsize="70%" meaning="4" role="NUMBER">4</XMTok></XMApp><XMTok role="MIDDLE" stretchy="false">|</XMTok><XMApp xml:id="p2.m1.8"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">s</XMTok><XMTok fontsize="70%" meaning="4" role="NUMBER">4</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">A</XMTok><XMTok fontsize="70%" meaning="4" role="NUMBER">4</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
    ] {
      let id = math
        .split("xml:id=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap();
      latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
    }
  }
}
