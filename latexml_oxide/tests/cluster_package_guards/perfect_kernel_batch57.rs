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
    r##"<Math mode="inline" tex="\textdollar 10\text{\,}{\mathrm{kg}}^{-1}" text="$ * (10 * power@(kilogram, - 1))" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok role="UNKNOWN">$</XMTok><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m1.1"> </XMText><XMTok meaning="10" role="NUMBER">10</XMTok><XMApp xml:id="p1.m1.3"><XMTok meaning="power" role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok class="ltx_unit" meaning="kilogram" role="ID">kg</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
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
    // Neither route reports them: bar fences nest no deeper than Perl's `MAX_ABS_DEPTH` (57am), so
    // the `\|…\|` formula's fallback forest keeps under ten trees.
    assert_eq!(warning_count(&stderr), 0, "limit {limit:?}: {stderr}");
    for math in [
      r##"<Math mode="inline" tex="|\log x-\log y|=a|u|\leq c" text="absolute-value@(logarithm@(x) - logarithm@(y)) = a * absolute-value@(u) &lt;= c" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="multirelation"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="p1.m1.1"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.2">u</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math>"##,
      r##"<Math mode="inline" tex="\|\nabla_{x}\log\pi(x)-\nabla_{x}\log\rho(x)\|=a\|\mu-u\|\leq c" text="norm@(((nabla _ x)@(logarithm))@(pi@(x)) - ((nabla _ x)@(logarithm))@(rho@(x))) = a * norm@(mu - u) &lt;= c" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="multirelation"/><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m2.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMApp xml:id="p1.m2.1"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMDual><XMRef idref="p1.m2.2"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.2">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp><XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok></XMApp><XMApp><XMTok font="italic" name="rho" role="UNKNOWN">ρ</XMTok><XMDual><XMRef idref="p1.m2.3"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.3">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMApp><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m2.4"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMApp xml:id="p1.m2.4"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok font="italic" name="mu" role="UNKNOWN">μ</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math>"##,
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
    // Each term is one derivation on both routes (57am: the bare function no chain item).
    assert_eq!(warning_count(&stderr), 0, "limit {limit:?}: {stderr}");
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

/// 57al: Perl flags a formula `ltx_math_unparsed` when any of its parses failed and the failure
/// is genuine (MathParser.pm:297-303, :997-1027: `is_genuinely_unparsed` descends an XMDual's
/// content branch only); Rust flagged only a failed XMath, so a gathered/split row's content
/// branch that did not parse left the formula looking parsed (2605.06394, 2605.09779,
/// 2605.09802, 2605.13374). Marked inside `parse`, as Perl does, so a formula nested in
/// `\mbox{$…$}` keeps its class when the outer parse copies it (57al review); the outer formula
/// parsed and is not flagged. Repro math-parse/unparsed_content_branch_is_flagged.
#[test]
fn unparsed_content_branch_is_flagged() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/unparsed_content_branch_is_flagged.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // One `unparsed_math` per distinct content parse that failed (Perl warns per parse, 4).
  assert_eq!(warning_count(&stderr), 3, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:unparsed_math:").count(),
    3,
    "{stderr}"
  );
  for (id, math) in [
    (
      "S0.Ex1.m1",
      r##"<Math class="ltx_math_unparsed" mode="display" tex="\begin{gathered}]\,x\,[\;\,\sum\end{gathered}" text="]@x@[@sum" xml:id="S0.Ex1.m1"><XMath><XMDual><XMWrap rule="Anything,"><XMRef idref="S0.Ex1.m1.1" rpadding="1.7pt"/><XMRef idref="S0.Ex1.m1.2" rpadding="1.7pt"/><XMRef idref="S0.Ex1.m1.3"/><XMRef idref="S0.Ex1.m1.4" lpadding="4.5pt"/></XMWrap><XMArray name="gathered"><XMRow><XMCell align="center"><XMArg rule="Anything,"><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false" xml:id="S0.Ex1.m1.1">]</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt" xml:id="S0.Ex1.m1.2">x</XMTok><XMTok role="OPEN" stretchy="false" xml:id="S0.Ex1.m1.3">[</XMTok><XMTok lpadding="4.5pt" mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid" xml:id="S0.Ex1.m1.4">∑</XMTok></XMArg></XMCell></XMRow></XMArray></XMDual></XMath></Math>"##,
    ),
    (
      "S0.E1.m1",
      r##"<Math class="ltx_math_unparsed" mode="display" tex="\begin{split}&amp;J_{ij}&gt;0\;:\qquad{\rm ferromagnetic}\;,\\&#10;&amp;J_{ij}&lt;0\;:\qquad{\rm anti-ferromagnetic}\;.\end{split}" text="J@(i * j)@()@&gt;@0@colon@ferromagnetic@PUNCT@J@(i * j)@()@&lt;@0@colon@anti@-@ferromagnetic@PERIOD" xml:id="S0.E1.m1"><XMath><XMDual><XMWrap rule="Anything,"><XMRef idref="S0.E1.m1.1"/><XMApp role="POSTSUBSCRIPT" scriptpos="6"><XMRef idref="S0.E1.m1.2"/></XMApp><XMRef idref="S0.E1.m1.3"/><XMRef idref="S0.E1.m1.4" rpadding="2.8pt"/><XMRef idref="S0.E1.m1.5"/><XMRef idref="S0.E1.m1.6" rpadding="2.8pt"/><XMTok role="PUNCT"/><XMRef idref="S0.E1.m1.7"/><XMApp role="POSTSUBSCRIPT" scriptpos="6"><XMRef idref="S0.E1.m1.8"/></XMApp><XMRef idref="S0.E1.m1.9"/><XMRef idref="S0.E1.m1.10" rpadding="2.8pt"/><XMRef idref="S0.E1.m1.11"/><XMRef idref="S0.E1.m1.12"/><XMRef idref="S0.E1.m1.13"/><XMRef idref="S0.E1.m1.14" rpadding="2.8pt"/><XMTok role="PERIOD"/></XMWrap><XMArray colsep="0pt" name="aligned"><XMRow><XMCell/><XMCell align="left"><XMArg rule="Anything,"><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.1">J</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="6"><XMApp xml:id="S0.E1.m1.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">j</XMTok></XMApp></XMApp><XMTok meaning="greater-than" role="RELOP" xml:id="S0.E1.m1.3">&gt;</XMTok><XMTok meaning="0" role="NUMBER" rpadding="2.8pt" xml:id="S0.E1.m1.4">0</XMTok><XMTok name="colon" role="METARELOP" xml:id="S0.E1.m1.5">:</XMTok><XMTok role="UNKNOWN" rpadding="2.8pt" xml:id="S0.E1.m1.6">ferromagnetic</XMTok><XMTok role="PUNCT">,</XMTok></XMArg></XMCell></XMRow><XMRow><XMCell/><XMCell align="left"><XMArg rule="Anything,"><XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.7">J</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="6"><XMApp xml:id="S0.E1.m1.8"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">j</XMTok></XMApp></XMApp><XMTok meaning="less-than" role="RELOP" xml:id="S0.E1.m1.9">&lt;</XMTok><XMTok meaning="0" role="NUMBER" rpadding="2.8pt" xml:id="S0.E1.m1.10">0</XMTok><XMTok name="colon" role="METARELOP" xml:id="S0.E1.m1.11">:</XMTok><XMTok role="UNKNOWN" xml:id="S0.E1.m1.12">anti</XMTok><XMTok meaning="minus" role="ADDOP" xml:id="S0.E1.m1.13">-</XMTok><XMTok role="UNKNOWN" rpadding="2.8pt" xml:id="S0.E1.m1.14">ferromagnetic</XMTok><XMTok role="PERIOD">.</XMTok></XMArg></XMCell></XMRow></XMArray></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m1",
      r##"<Math mode="inline" tex="a=b+\mbox{ where $\begin{gathered}]\,x\,[\;\,\sum\end{gathered}$ holds}" text="a = b + [ where ]x[∑ holds]" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMText> where <Math class="ltx_math_unparsed" mode="inline" tex="\begin{gathered}]\,x\,[\;\,\sum\end{gathered}" text="]@x@[@sum" xml:id="p1.m1.m1"><XMath><XMDual><XMWrap rule="Anything,"><XMRef idref="p1.m1.m1.1" rpadding="1.7pt"/><XMRef idref="p1.m1.m1.2" rpadding="1.7pt"/><XMRef idref="p1.m1.m1.3"/><XMRef idref="p1.m1.m1.4" lpadding="4.5pt"/></XMWrap><XMArray name="gathered"><XMRow><XMCell align="center"><XMArg rule="Anything,"><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false" xml:id="p1.m1.m1.1">]</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt" xml:id="p1.m1.m1.2">x</XMTok><XMTok role="OPEN" stretchy="false" xml:id="p1.m1.m1.3">[</XMTok><XMTok lpadding="4.5pt" mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid" xml:id="p1.m1.m1.4">∑</XMTok></XMArg></XMCell></XMRow></XMArray></XMDual></XMath></Math> holds</XMText></XMApp></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57am: Perl's `absExpression` sets `$forbidEvalAt` (MathGrammar:410; tested at :259-262), so no
/// evaluation bar is read inside a `|…|` pair: `|\nabla a|_L|\nabla b|_L` is |∇a|_L·|∇b|_L, not an
/// evaluation bar closing over both (RUST-ONLY; 2605.12082, 2605.04766). Divergence #350: only
/// single-bar pairs, and not in a nested group — `\|u|_{\Gamma}\|`, `\left|M|_{S}\right|`,
/// `|g(f|_{x=0})|` keep their evaluation bar (2605.01526, 2605.07463; Perl fails) — and a
/// preference, not a prune: `|f(x)|_{0}^{1}|`, whose only reading nests it, keeps it (Perl fails),
/// and counted per pair — `|\nabla a|_L|\nabla b|_L+|f(x)|_0^1|` keeps Perl's |∇a|_L·|∇b|_L (one
/// `ambiguous_math` warning there).
#[test]
fn evaluated_at_stays_outside_absolute_bars() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/evaluated_at_stays_outside_absolute_bars.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  for (id, math) in [
    (
      "p1.m1",
      r##"<Math mode="inline" tex="|\nabla a|_{L}|\nabla b|_{L}" text="(absolute-value@(nabla@(a))) _ L * (absolute-value@(nabla@(b))) _ L" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="p1.m1.1"><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="p1.m1.2"><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "p1.m2",
      r##"<Math mode="inline" tex="|a|_{L}|b|_{L}" text="(absolute-value@(a)) _ L * (absolute-value@(b)) _ L" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m2.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.1">a</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m2.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.2">b</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "p1.m3",
      r##"<Math mode="inline" tex="(|u|_{H}+|\nabla\times u|_{H})" text="(absolute-value@(u)) _ H + (absolute-value@(nabla * u)) _ H" xml:id="p1.m3"><XMath><XMDual><XMRef idref="p1.m3.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m3.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m3.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m3.2">u</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">H</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m3.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="p1.m3.3"><XMTok meaning="times" role="MULOP">×</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">H</XMTok></XMApp></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m4",
      r##"<Math mode="inline" tex="f(x)|_{0}^{1}" text="evaluated-at@(f@(x), 0, 1)" xml:id="p1.m4"><XMath><XMDual><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="p1.m4.1"/><XMRef idref="p1.m4.3"/><XMRef idref="p1.m4.4"/></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMApp xml:id="p1.m4.1"><XMTok font="italic" role="UNKNOWN">f</XMTok><XMDual><XMRef idref="p1.m4.2"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m4.2">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMTok fontsize="70%" meaning="0" role="NUMBER" xml:id="p1.m4.3">0</XMTok></XMApp><XMTok fontsize="70%" meaning="1" role="NUMBER" xml:id="p1.m4.4">1</XMTok></XMApp></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m5",
      r##"<Math mode="inline" tex="\|a\|_{2}\|b\|_{2}" text="(norm@(a)) _ 2 * (norm@(b)) _ 2" xml:id="p1.m5"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m5.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m5.1">a</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m5.2"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m5.2">b</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "p1.m6",
      r##"<Math mode="inline" tex="\|u|_{\Gamma}\|" text="norm@(evaluated-at@(u, Gamma))" xml:id="p1.m6"><XMath><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m6.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMDual xml:id="p1.m6.1"><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="p1.m6.2"/><XMRef idref="p1.m6.3"/></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="p1.m6.2">u</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMTok fontsize="70%" name="Gamma" role="UNKNOWN" xml:id="p1.m6.3">Γ</XMTok></XMApp></XMDual><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m7",
      r##"<Math mode="inline" tex="\left|M|_{S}\right|" text="absolute-value@(evaluated-at@(M, S))" xml:id="p1.m7"><XMath><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m7.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="true">|</XMTok><XMDual xml:id="p1.m7.1"><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="p1.m7.2"/><XMRef idref="p1.m7.3"/></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="p1.m7.2">M</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="p1.m7.3">S</XMTok></XMApp></XMDual><XMTok role="CLOSE" stretchy="true">|</XMTok></XMWrap></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m8",
      r##"<Math mode="inline" tex="|g(f|_{x=0})|" text="absolute-value@(g@(evaluated-at@(f, x = 0)))" xml:id="p1.m8"><XMath><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m8.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="p1.m8.1"><XMTok font="italic" role="UNKNOWN">g</XMTok><XMDual><XMRef idref="p1.m8.2"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMDual xml:id="p1.m8.2"><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="p1.m8.3"/><XMRef idref="p1.m8.4"/></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="p1.m8.3">f</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMApp xml:id="p1.m8.4"><XMTok fontsize="70%" meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="0" role="NUMBER">0</XMTok></XMApp></XMApp></XMDual><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m9",
      r##"<Math mode="inline" tex="|f(x)|_{0}^{1}|" text="absolute-value@(evaluated-at@(f@(x), 0, 1))" xml:id="p1.m9"><XMath><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m9.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMDual xml:id="p1.m9.1"><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="p1.m9.2"/><XMRef idref="p1.m9.4"/><XMRef idref="p1.m9.5"/></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMApp xml:id="p1.m9.2"><XMTok font="italic" role="UNKNOWN">f</XMTok><XMDual><XMRef idref="p1.m9.3"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m9.3">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMTok fontsize="70%" meaning="0" role="NUMBER" xml:id="p1.m9.4">0</XMTok></XMApp><XMTok fontsize="70%" meaning="1" role="NUMBER" xml:id="p1.m9.5">1</XMTok></XMApp></XMDual><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m10",
      r##"<Math mode="inline" tex="|\nabla a|_{L}|\nabla b|_{L}+|f(x)|_{0}^{1}|" text="(absolute-value@(nabla@(a))) _ L * (absolute-value@(nabla@(b))) _ L + absolute-value@(evaluated-at@(f@(x), 0, 1))" xml:id="p1.m10"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m10.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="p1.m10.1"><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m10.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="p1.m10.2"><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m10.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMDual xml:id="p1.m10.3"><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="p1.m10.4"/><XMRef idref="p1.m10.6"/><XMRef idref="p1.m10.7"/></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMApp xml:id="p1.m10.4"><XMTok font="italic" role="UNKNOWN">f</XMTok><XMDual><XMRef idref="p1.m10.5"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m10.5">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMTok fontsize="70%" meaning="0" role="NUMBER" xml:id="p1.m10.6">0</XMTok></XMApp><XMTok fontsize="70%" meaning="1" role="NUMBER" xml:id="p1.m10.7">1</XMTok></XMApp></XMDual><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// A conditional bar takes no relation missing its operand beside it (Perl's conditional is
/// `Term | ExpressionsNoBars`, MathGrammar:261-268): `\bigg|g\big|_{t=1}-h\bigg|\le C` is
/// |eval(g, t=1) − h| ≤ C, not (|g|)_{t=1}−h | (absent ≤ C) (2605.26054, 2605.02499, 2605.12082);
/// genuine conditionals stay. One `ambiguous_math` warning: the three bar pairs of
/// `\leq|a|\,|b|_{L}|c|_{L}` have 12 trees.
#[test]
fn conditional_bar_takes_no_bare_relation() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/conditional_bar_takes_no_bare_relation.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  for (id, math) in [
    (
      "S0.Ex1.m1",
      r##"<Math mode="display" tex="\bigg|g\big|_{t=1}-h\bigg|\leq C" text="absolute-value@(evaluated-at@(g, t = 1) - h) &lt;= C" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex1.m1.1"/></XMApp><XMWrap><XMTok fontsize="210%" role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="S0.Ex1.m1.1"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMDual><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="S0.Ex1.m1.2"/><XMRef idref="S0.Ex1.m1.3"/></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.2">g</XMTok><XMTok fontsize="120%" role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMApp xml:id="S0.Ex1.m1.3"><XMTok fontsize="70%" meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">t</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp></XMDual><XMTok font="italic" role="UNKNOWN">h</XMTok></XMApp><XMTok fontsize="210%" role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">C</XMTok></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex2.m1",
      r##"<Math mode="display" tex="\lim_{N\to\infty}|Nf\big|_{N}(n)-g(n)|=0" text="(limit _ (N to infinity))@(absolute-value@(evaluated-at@(N * f, N) * n - g@(n))) = 0" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok meaning="limit" role="LIMITOP" scriptpos="mid">lim</XMTok><XMApp><XMTok fontsize="70%" name="to" role="ARROW">→</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">N</XMTok><XMTok fontsize="70%" meaning="infinity" name="infty" role="ID">∞</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex2.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="S0.Ex2.m1.1"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="S0.Ex2.m1.2"/><XMRef idref="S0.Ex2.m1.3"/></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMApp xml:id="S0.Ex2.m1.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">N</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp><XMTok fontsize="120%" role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.Ex2.m1.3">N</XMTok></XMApp></XMDual><XMDual><XMRef idref="S0.Ex2.m1.4"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.4">n</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok font="italic" role="UNKNOWN">g</XMTok><XMDual><XMRef idref="S0.Ex2.m1.5"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.5">n</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="0" role="NUMBER">0</XMTok></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex3.m1",
      r##"<Math mode="display" tex="\delta(\xi)\asymp|\xi-\phi(x)|\asymp r" text="delta@(xi) asymptotically-equals absolute-value@(xi - phi@(x)) asymptotically-equals r" xml:id="S0.Ex3.m1"><XMath><XMApp><XMTok meaning="multirelation"/><XMApp><XMTok font="italic" name="delta" role="UNKNOWN">δ</XMTok><XMDual><XMRef idref="S0.Ex3.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" name="xi" role="UNKNOWN" xml:id="S0.Ex3.m1.1">ξ</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="asymptotically-equals" name="asymp" role="RELOP">≍</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="S0.Ex3.m1.2"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok font="italic" name="xi" role="UNKNOWN">ξ</XMTok><XMApp><XMTok font="italic" name="phi" role="UNKNOWN">ϕ</XMTok><XMDual><XMRef idref="S0.Ex3.m1.3"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.3">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok meaning="asymptotically-equals" name="asymp" role="RELOP">≍</XMTok><XMTok font="italic" role="UNKNOWN">r</XMTok></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex4.m1",
      r##"<Math mode="display" tex="\leq|a|\,|b|_{L}|c|_{L}" text="absent &lt;= absolute-value@(a) * (absolute-value@(b)) _ L * (absolute-value@(c)) _ L" xml:id="S0.Ex4.m1"><XMath><XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok meaning="absent"/><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.1">a</XMTok><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false">|</XMTok></XMWrap></XMDual><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.2">b</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.3">c</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex5.m1",
      r##"<Math mode="display" tex="p(x|y)+P(A|B=b)" text="p@(conditional@(x, y)) + P@(conditional@(A, B = b))" xml:id="S0.Ex5.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok font="italic" role="UNKNOWN">p</XMTok><XMDual><XMApp><XMTok meaning="conditional"/><XMRef idref="S0.Ex5.m1.1"/><XMRef idref="S0.Ex5.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.1">x</XMTok><XMTok role="MIDDLE" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.2">y</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok font="italic" role="UNKNOWN">P</XMTok><XMDual><XMApp><XMTok meaning="conditional"/><XMRef idref="S0.Ex5.m1.3"/><XMRef idref="S0.Ex5.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.3">A</XMTok><XMTok role="MIDDLE" stretchy="false">|</XMTok><XMApp xml:id="S0.Ex5.m1.4"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">B</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// A chain of OPFUNCTIONs taking bare arguments nests greedily, as Perl's `barearg`
/// (MathGrammar:321-337): `\log x\,\log y\,…\,\log t` is log@(x·log@(y·…)). Its one reading sits among
/// ~4ⁿ trees, past the tree iterator's caps; ASF, a second chance for a sampled bocage with no
/// parse, reads it exactly. One `ambiguous_math` warning per formula (the sample's enumeration).
/// Eleven bar pairs beside three logs, and sixteen logs beside bar pairs, parse too since bar fences
/// nest no deeper than Perl's `MAX_ABS_DEPTH` (before, the first passed 8 GB uncapped); on a smaller
/// budget they spend it and stay flagged unparsed, keeping none of the readings of a traversal cut
/// short inside the root.
#[test]
fn opfunction_chain_parses_past_the_tree_sampler() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/opfunction_chain_parses_past_the_tree_sampler.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 4, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:ambiguous_math:").count(),
    4,
    "{stderr}"
  );
  for (id, math) in [
    (
      "S0.Ex1.m1",
      r##"<Math mode="display" tex="\log x\,\log y\,\log z\,\log w\,\log v\,\log u\,\log t" text="logarithm@(x * logarithm@(y * logarithm@(z * logarithm@(w * logarithm@(v * logarithm@(u * logarithm@(t)))))))" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">x</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">y</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">z</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">w</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">v</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">u</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">t</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex2.m1",
      r##"<Math mode="display" tex="\operatorname{tr}A\operatorname{tr}B\operatorname{tr}C\operatorname{tr}D\operatorname{tr}E\operatorname{tr}F\operatorname{tr}G" text="tr@(A * tr@(B * tr@(C * tr@(D * tr@(E * tr@(F * tr@(G)))))))" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">A</XMTok><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">B</XMTok><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">C</XMTok><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">D</XMTok><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">E</XMTok><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">F</XMTok><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">G</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex3.m1",
      r##"<Math mode="display" tex="|a||b||c||d||e||f||g||h||i||j||k|+\log a\,\log b\,\log c" text="absolute-value@(a) * absolute-value@(b) * absolute-value@(c) * absolute-value@(d) * absolute-value@(e) * absolute-value@(f) * absolute-value@(g) * absolute-value@(h) * absolute-value@(i) * absolute-value@(j) * absolute-value@(k) + logarithm@(a * logarithm@(b * logarithm@(c)))" xml:id="S0.Ex3.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.1">a</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.2">b</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.3">c</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.4">d</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.5"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.5">e</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.6"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.6">f</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.7"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.7">g</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.8"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.8">h</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.9"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.9">i</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.10"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.10">j</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.11"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.11">k</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex4.m1",
      r##"<Math mode="display" tex="\log a\,\log b\,\log c\,\log d\,\log e\,\log f\,\log g\,\log h\,\log i\,\log j\,\log k\,\log l\,\log m\,\log n\,\log o\,\log p+|a|_{L}|b|_{L}+|c|_{L}|d|_{L}+|e|_{L}|f|_{L}+|g|_{L}+|z|^{2}" text="logarithm@(a * logarithm@(b * logarithm@(c * logarithm@(d * logarithm@(e * logarithm@(f * logarithm@(g * logarithm@(h * logarithm@(i * logarithm@(j * logarithm@(k * logarithm@(l * logarithm@(m * logarithm@(n * logarithm@(o * logarithm@(p)))))))))))))))) + (absolute-value@(a)) _ L * (absolute-value@(b)) _ L + (absolute-value@(c)) _ L * (absolute-value@(d)) _ L + (absolute-value@(e)) _ L * (absolute-value@(f)) _ L + (absolute-value@(g)) _ L + (absolute-value@(z)) ^ 2" xml:id="S0.Ex4.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">c</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">d</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">e</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">f</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">g</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">h</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">i</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">j</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">k</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">l</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">m</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">n</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">o</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.1">a</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.2">b</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.3">c</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.4">d</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.5"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.5">e</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.6"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.6">f</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.7"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.7">g</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.8"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.8">z</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
  // The second chance's work is bounded: below what the last two need (674 and 1,759 alternatives),
  // it spends the budget, keeps none of the readings — cut short, they are an arbitrary part of the
  // whole — and the formulas stay flagged unparsed; the chains, 172 each, parse as before.
  let (stderr, xml, ()) = super::perfect_kernel_batch46::convert_with_setup_then(
    tex,
    None,
    || latexml_math_parser::set_asf_second_chance_alternatives_override(Some(500)),
    |_| (),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    stderr.matches("ASF second chance: budget spent").count(),
    2,
    "{stderr}"
  );
  assert_eq!(warning_count(&stderr), 4, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:unparsed_math:").count(),
    2,
    "{stderr}"
  );
  for (id, math) in [
    (
      "S0.Ex3.m1",
      r##"<Math class="ltx_math_unparsed" mode="display" tex="|a||b||c||d||e||f||g||h||i||j||k|+\log a\,\log b\,\log c" xml:id="S0.Ex3.m1"><XMath><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">g</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">h</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">i</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">j</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">k</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math>"##,
    ),
    (
      "S0.Ex4.m1",
      r##"<Math class="ltx_math_unparsed" mode="display" tex="\log a\,\log b\,\log c\,\log d\,\log e\,\log f\,\log g\,\log h\,\log i\,\log j\,\log k\,\log l\,\log m\,\log n\,\log o\,\log p+|a|_{L}|b|_{L}+|c|_{L}|d|_{L}+|e|_{L}|f|_{L}+|g|_{L}+|z|^{2}" xml:id="S0.Ex4.m1"><XMath><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">c</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">d</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">e</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">f</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">g</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">h</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">i</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">j</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">k</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">l</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">m</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">n</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">o</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">p</XMTok><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">g</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMApp role="POSTSUBSCRIPT" scriptpos="1"><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">z</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMApp role="POSTSUPERSCRIPT" scriptpos="1"><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// Perl parses with `MAX_ABS_DEPTH` 1 — no bar fence inside another (`absExpression`, MathGrammar:410-412) —
/// and retries at 2, then 3, only when that failed having tried deeper (MathParser.pm:813-836): the bars of
/// `\log|a|+…+\log|d|` are four absolute values, never one around `c·|+\log|·d`; `||x|+|y||` nests on the
/// retry. Four `ambiguous_math` warnings: the readings nested too deep are counted pruned.
#[test]
fn bar_pairs_nest_as_shallow_as_they_can() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/bar_pairs_nest_as_shallow_as_they_can.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 4, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:ambiguous_math:").count(),
    4,
    "{stderr}"
  );
  for (id, math) in [
    (
      "S0.Ex1.m1",
      r##"<Math mode="display" tex="\log|a|+\log|b|+\log|c|+\log|d|" text="logarithm@(absolute-value@(a)) + logarithm@(absolute-value@(b)) + logarithm@(absolute-value@(c)) + logarithm@(absolute-value@(d))" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex1.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.1">a</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex1.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.2">b</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex1.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.3">c</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex1.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.4">d</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex2.m1",
      r##"<Math mode="display" tex="\log\|a\|+\log\|b\|+\log\|c\|+\log\|d\|+\log\|e\|+\log\|f\|" text="logarithm@(norm@(a)) + logarithm@(norm@(b)) + logarithm@(norm@(c)) + logarithm@(norm@(d)) + logarithm@(norm@(e)) + logarithm@(norm@(f))" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex2.m1.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.1">a</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex2.m1.2"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.2">b</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex2.m1.3"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.3">c</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex2.m1.4"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.4">d</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex2.m1.5"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.5">e</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex2.m1.6"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.6">f</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex3.m1",
      r##"<Math mode="display" tex="\log|a|\,\log|b|\,\log|c|\,\log|d|" text="logarithm@(absolute-value@(a) * logarithm@(absolute-value@(b) * logarithm@(absolute-value@(c) * logarithm@(absolute-value@(d)))))" xml:id="S0.Ex3.m1"><XMath><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.1">a</XMTok><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false">|</XMTok></XMWrap></XMDual><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.2">b</XMTok><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false">|</XMTok></XMWrap></XMDual><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.3">c</XMTok><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false">|</XMTok></XMWrap></XMDual><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.4">d</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex4.m1",
      r##"<Math mode="display" tex="\max_{i}||a_{i}||+\max_{i}||b_{i}||+\max_{i}||c_{i}||+\max_{i}||d_{i}||" text="(maximum _ i)@(norm@(a _ i)) + (maximum _ i)@(norm@(b _ i)) + (maximum _ i)@(norm@(c _ i)) + (maximum _ i)@(norm@(d _ i))" xml:id="S0.Ex4.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok meaning="maximum" role="OPFUNCTION" scriptpos="mid">max</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex4.m1.1"/></XMApp><XMWrap><XMTok role="OPEN">‖</XMTok><XMApp xml:id="S0.Ex4.m1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMTok role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok meaning="maximum" role="OPFUNCTION" scriptpos="mid">max</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex4.m1.2"/></XMApp><XMWrap><XMTok role="OPEN">‖</XMTok><XMApp xml:id="S0.Ex4.m1.2"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMTok role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok meaning="maximum" role="OPFUNCTION" scriptpos="mid">max</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex4.m1.3"/></XMApp><XMWrap><XMTok role="OPEN">‖</XMTok><XMApp xml:id="S0.Ex4.m1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMTok role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok meaning="maximum" role="OPFUNCTION" scriptpos="mid">max</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex4.m1.4"/></XMApp><XMWrap><XMTok role="OPEN">‖</XMTok><XMApp xml:id="S0.Ex4.m1.4"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMTok role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex5.m1",
      r##"<Math mode="display" tex="||x|+|y||" text="absolute-value@(absolute-value@(x) + absolute-value@(y))" xml:id="S0.Ex5.m1"><XMath><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex5.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="S0.Ex5.m1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex5.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.2">x</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex5.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.3">y</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMath></Math>"##,
    ),
    (
      "S0.Ex6.m1",
      r##"<Math mode="display" tex="\log||x|+|y||\,z" text="logarithm@(absolute-value@(absolute-value@(x) + absolute-value@(y)) * z)" xml:id="S0.Ex6.m1"><XMath><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex6.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="S0.Ex6.m1.1"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex6.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex6.m1.2">x</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex6.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex6.m1.3">y</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">z</XMTok></XMApp></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// A long formula pairs its bars as Perl: `|\Gamma|` in 2605.19037 S3.E24 is an absolute value, not a
/// `conditional` across the second sum (Perl's conditional takes `ExpressionsNoBars`, MathGrammar:261-268),
/// and 2605.26654 A3.Ex160's four `\|…\|` are sibling norms, none nested in another (`MAX_ABS_DEPTH`,
/// MathParser.pm:813-836). Two `ambiguous_math` warnings: the sampled readings are counted.
#[test]
fn bars_pair_in_a_long_formula() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/math-parse/bars_pair_in_a_long_formula.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 2, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:ambiguous_math:").count(),
    2,
    "{stderr}"
  );
  for (id, math) in [
    (
      "p1.m1",
      r##"<Math mode="inline" tex="\sum_{K\in\mathcal{T}_{h}}\int_{K}\nabla u_{h}\cdot\nabla v_{h}\,{\rm d}x+\frac{D}{2}\sum_{\Gamma\in\mathbb{E}_{h}}\sum_{a\in e(\Gamma)}|\Gamma|[u_{h}]_{a}[v_{h}]_{a}=\int_{\Omega}fv_{h}\,{\rm d}x" text="(sum _ (K element-of T _ h))@((integral _ K)@((nabla@(u _ h) cdot nabla@(v _ h)) * differential-d@(x))) + (D / 2) * (sum _ (Gamma element-of E _ h))@((sum _ (a element-of e@(Gamma)))@(absolute-value@(Gamma) * (delimited-[]@(u _ h)) _ a * (delimited-[]@(v _ h)) _ a)) = (integral _ Omega)@(f * v _ h * differential-d@(x))" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok mathstyle="text" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok><XMApp><XMTok fontsize="70%" meaning="element-of" name="in" role="RELOP">∈</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">K</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post2"/><XMTok font="caligraphic" fontsize="70%" role="UNKNOWN">T</XMTok><XMTok font="italic" fontsize="50%" role="UNKNOWN">h</XMTok></XMApp></XMApp></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok mathstyle="text" meaning="integral" name="int" role="INTOP">∫</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">K</XMTok></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">u</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">h</XMTok></XMApp></XMApp><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp rpadding="1.7pt"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">h</XMTok></XMApp></XMApp></XMApp><XMApp><XMTok meaning="differential-d" role="DIFFOP">d</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMApp></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok mathstyle="text" meaning="divide" role="FRACOP"/><XMTok font="italic" fontsize="70%" role="UNKNOWN">D</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok mathstyle="text" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok><XMApp><XMTok fontsize="70%" meaning="element-of" name="in" role="RELOP">∈</XMTok><XMTok fontsize="70%" name="Gamma" role="UNKNOWN">Γ</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post2"/><XMTok font="blackboard" fontsize="70%" role="UNKNOWN">E</XMTok><XMTok font="italic" fontsize="50%" role="UNKNOWN">h</XMTok></XMApp></XMApp></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok mathstyle="text" meaning="sum" role="SUMOP" scriptpos="post">∑</XMTok><XMApp><XMTok fontsize="70%" meaning="element-of" name="in" role="RELOP">∈</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok><XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">e</XMTok><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok fontsize="70%" role="OPEN" stretchy="false">(</XMTok><XMTok fontsize="70%" name="Gamma" role="UNKNOWN" xml:id="p1.m1.1">Γ</XMTok><XMTok fontsize="70%" role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="p1.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok name="Gamma" role="UNKNOWN" xml:id="p1.m1.2">Γ</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="delimited-[]"/><XMRef idref="p1.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">[</XMTok><XMApp xml:id="p1.m1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">u</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">h</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="delimited-[]"/><XMRef idref="p1.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">[</XMTok><XMApp xml:id="p1.m1.4"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">h</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok mathstyle="text" meaning="integral" name="int" role="INTOP">∫</XMTok><XMTok fontsize="70%" name="Omega" role="UNKNOWN">Ω</XMTok></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok><XMApp rpadding="1.7pt"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">h</XMTok></XMApp><XMApp><XMTok meaning="differential-d" role="DIFFOP">d</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "p2.m1",
      r##"<Math mode="inline" tex="\leq\tau^{-1}\|\operatorname{diag}(\pi^{*}(x_{1}))\|\|\nabla r(x_{1})-\nabla r(x_{2})\|+\tau^{-1}\|\operatorname{diag}(\pi^{*}(x_{1}))-\operatorname{diag}(\pi^{*}(x_{2}))\|\|\nabla r(x_{2})\|" text="absent &lt;= tau ^ (- 1) * norm@(diag@(pi ^ * * x _ 1)) * norm@(nabla@(r@(x _ 1)) - nabla@(r@(x _ 2))) + tau ^ (- 1) * norm@(diag@(pi ^ * * x _ 1) - diag@(pi ^ * * x _ 2)) * norm@(nabla@(r@(x _ 2)))" xml:id="p2.m1"><XMath><XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok meaning="absent"/><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="tau" role="UNKNOWN">τ</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p2.m1.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMDual xml:id="p2.m1.1"><XMApp><XMRef idref="p2.m1.2"/><XMRef idref="p2.m1.3"/></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post" xml:id="p2.m1.2">diag</XMTok><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMTok fontsize="70%" meaning="times" role="MULOP">∗</XMTok></XMApp><XMDual><XMRef idref="p2.m1.4"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.4"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMApp></XMDual><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p2.m1.5"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMApp xml:id="p2.m1.5"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok font="italic" role="UNKNOWN">r</XMTok><XMDual><XMRef idref="p2.m1.6"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.6"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok font="italic" role="UNKNOWN">r</XMTok><XMDual><XMRef idref="p2.m1.7"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.7"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp></XMApp><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="tau" role="UNKNOWN">τ</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p2.m1.8"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMApp xml:id="p2.m1.8"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMDual><XMApp><XMRef idref="p2.m1.9"/><XMRef idref="p2.m1.10"/></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post" xml:id="p2.m1.9">diag</XMTok><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.10"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMTok fontsize="70%" meaning="times" role="MULOP">∗</XMTok></XMApp><XMDual><XMRef idref="p2.m1.11"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.11"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMApp></XMDual><XMDual><XMApp><XMRef idref="p2.m1.12"/><XMRef idref="p2.m1.13"/></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post" xml:id="p2.m1.12">diag</XMTok><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.13"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMTok fontsize="70%" meaning="times" role="MULOP">∗</XMTok></XMApp><XMDual><XMRef idref="p2.m1.14"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.14"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMApp></XMDual></XMApp><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p2.m1.15"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMApp xml:id="p2.m1.15"><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok font="italic" role="UNKNOWN">r</XMTok><XMDual><XMRef idref="p2.m1.16"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p2.m1.16"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMApp></XMApp></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57ao: a sample the tree iterator cut short at `max_unique` is a part of the readings — the
/// iterator varies the leftmost choice fastest, so the one where every integral takes its whole
/// operand (Perl's greedy `addOpArgs`, MathGrammar:603-617) came after the first ten, eleventh of
/// 16 in `\int dt\,a/t=\int dt\,b/t` and none of the ten in 2605.19037 S4.E47's split rows. It
/// goes to the bounded ASF second chance, whose complete readings complete it: the ranking finds
/// the wide reading, and each of the first section's formulas reads as through ASF alone
/// (witnesses 2605.16034, 2605.22940, 2605.19037). 57at: the readings the sample missed join it
/// after its own, so where the ranking cannot choose the sample's first reading is taken, as
/// before 57ao — the second section's `(f(X)-f(Y))(h(X)-h(Y))` keeps f@(X) (#18; 2605.18798),
/// which ASF alone listed after f·X. 57av: the student pragmas rank the readings by violation count
/// (K19), so both routes read the second section alike too (golden tests/parse/function_application.tex).
/// Golden
/// tests/parse/sampled_readings.tex; its rows enumerate more than ten readings, so both routes warn
/// `ambiguous_math` alike.
#[test]
fn sampled_readings_match_asf() {
  let maths = |xml: &str| -> Vec<String> {
    xml
      .split("<Math ")
      .skip(1)
      .map(|m| format!("<Math {}", m.split("</Math>").next().unwrap_or_default()))
      .collect()
  };
  // The tree iterator past 500 AND-nodes (the default, pinned so the environment cannot make both
  // routes ASF), then pure ASF. The nine parses (three inline formulas, the split and its two rows;
  // the second section's three formulas; 574-5,033 AND-nodes) each stop at ten readings and each take
  // the second chance, whose readings complete the sample (≤ 70 ms in release, under the 5 s
  // deadline); pure ASF has none to take.
  let mut routes = Vec::new();
  for (limit, second_chances) in [(Some(Some(500)), 9), (Some(None), 0)] {
    let (stderr, xml, ()) = super::perfect_kernel_batch46::convert_with_setup_then(
      include_str!("../parse/sampled_readings.tex"),
      None,
      move || latexml_math_parser::set_hybrid_and_node_limit_override(limit),
      |_| (),
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    let lines = |needle: &str| stderr.lines().filter(|line| line.contains(needle)).count();
    // Each parse enumerates more than ten readings, through either route.
    assert_eq!(
      lines("Warning:ambiguous_math:"),
      9,
      "limit {limit:?}: {stderr}"
    );
    assert_eq!(warning_count(&stderr), 9, "limit {limit:?}: {stderr}");
    assert_eq!(
      lines("ASF second chance: parsed,"),
      second_chances,
      "limit {limit:?}: {stderr}"
    );
    routes.push(maths(&xml));
  }
  assert_eq!(routes[0].len(), 7);
  // 57av (K19 step 1): the second section no longer reads by route order — pure ASF listed f·X
  // first, and the pragmas now rank the readings instead of giving up.
  assert_eq!(routes[0], routes[1]);
}

/// 57aq: declaration scopes Perl's rewrite resolves specially, as same-host Perl reads them
/// (Rewrite.pm:49-55, :298-311): a scope no element carries applies nowhere — `scope=id:NOPE`
/// silently, `scope=label:nope` with `getLabelID`'s error — an unrecognized scope is ignored
/// with an error and the rule applies unscoped (`scope=bogus`; `scope=global` too, which is no
/// Perl scope pattern either, 57as), an empty `scope=` is the whole
/// document, and the latest of two declarations wins (`UnshiftValue`, latexml.sty.ltxml:564).
/// Rust applied the unresolved ones everywhere, fell back to the current section for `scope=`,
/// let the earliest fast-path declaration win, and reported no error. Perl also warns that each
/// non-counter scope `\c@<scope>` is no register and about an undefined value in its own code
/// (KNOWN_PERL_ERRORS #378). The unit scopes: golden tests/parse/declaration_scope.tex.
#[test]
fn declaration_scopes_resolve_as_perl() {
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{latexml}\n\\begin{document}\n\\section{A}\n\
     \\lxDeclare[scope=label:nope,role=FUNCTION]{$q$}%\n\\lxDeclare[scope=id:NOPE,role=ID]{$r$}%\n\
     \\lxDeclare[scope=bogus,role=ID]{$s$}%\n\\lxDeclare[scope=global,role=ID]{$g$}%\n\
     $q$ $r$ $s$ $w$ $g$\n\\section{B}\n\
     \\lxDeclare[scope=,role=ID]{$w$}%\n\\lxDeclare[role=ADDOP]{$*$}%\n\\lxDeclare[role=MULOP]{$*$}%\n\
     $q$ $r$ $s$ $w$ $a*b$ $g$\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 3, "{stderr}");
  for message in [
    "Error:misdefined:<rewrite> No id for label nope in Rewrite",
    "Error:misdefined:<rewrite> Unrecognized scope pattern in Rewrite clause: \"bogus\"; Ignoring it.",
    "Error:misdefined:<rewrite> Unrecognized scope pattern in Rewrite clause: \"global\"; Ignoring it.",
  ] {
    assert_eq!(
      stderr.lines().filter(|line| line.contains(message)).count(),
      1,
      "{stderr}"
    );
  }
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (section, g) in [("S1", 5), ("S2", 6)] {
    for (m, letter, role) in [
      (1, "q", "UNKNOWN"),
      (2, "r", "UNKNOWN"),
      (3, "s", "ID"),
      (4, "w", "ID"),
      (g, "g", "ID"),
    ] {
      latexml::util::test::assert_element(
        &xml,
        "Math",
        &[&format!(r#"xml:id="{section}.p1.m{m}""#)],
        &format!(
          r#"<Math mode="inline" tex="{letter}" text="{letter}" xml:id="{section}.p1.m{m}"><XMath><XMTok font="italic" role="{role}">{letter}</XMTok></XMath></Math>"#
        ),
      );
    }
  }
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S2.p1.m5""#],
    r#"<Math mode="inline" tex="a*b" text="a * b" xml:id="S2.p1.m5"><XMath><XMApp><XMTok meaning="times" role="MULOP">∗</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"#,
  );
}

/// 57an: the Rust-authored parse goldens (`tests/parse/<phenomenon>.tex`, which absorbed the green
/// `math-parse/` repros and their whole-`<Math>` guards, and the older `count_parses`, `norm`,
/// `scripted_operator`) parse every formula. The `70_parse` goldens pin each formula's XML; this pins
/// that no formula warns, so a regression that leaves one unparsed fails here instead of being
/// blessed into its golden. The Perl mirrors (`LaTeXML/t/parse` copies) are not listed: five of them
/// (`compose`, `functions`, `kludge`, `operators`, `qm`) warn today, with unparsed or ambiguous
/// formulas their goldens record. Nor is `sampled_readings`, whose rows must enumerate more than
/// ten readings: `sampled_readings_match_asf` pins its warnings by count.
mod parse_groups_are_warning_free {
  use super::{convert_with, error_count, warning_count};

  macro_rules! warning_free {
    ($($group:ident),* $(,)?) => {$(
      #[test]
      fn $group() {
        let (stderr, _) =
          convert_with(include_str!(concat!("../parse/", stringify!($group), ".tex")), None);
        assert_eq!(error_count(&stderr), 0, "{stderr}");
        assert_eq!(warning_count(&stderr), 0, "{stderr}");
      }
    )*};
  }

  warning_free!(
    aligned_content_branch,
    bar_pairs,
    bigop_operands,
    count_parses,
    ellipsis_products,
    declaration_scope,
    declared_operators,
    decorated_relations,
    fenced_lists,
    function_application,
    integrals_and_differentials,
    math_lexemes,
    norm,
    operator_application,
    opfunction_arguments,
    paren_pairs,
    rust_parse_additions,
    scripted_operator,
    trailing_punctuation,
  );
}
