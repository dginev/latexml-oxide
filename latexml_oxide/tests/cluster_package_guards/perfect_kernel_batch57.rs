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

/// A chain of OPFUNCTIONs taking bare arguments reads separate factors since 57cb (user ruling
/// 2026-09-29): `\log x\,\log y\,…\,\log t` is log@(x)·log@(y)·…, one derivation, where Perl's greedy
/// `barearg` (MathGrammar:321-337) nested log@(x·log@(y·…)) among ~4ⁿ trees, past the tree iterator's
/// caps. Eleven bar pairs beside three logs, and sixteen logs beside bar pairs, still reach the ASF
/// second chance through their bars (one `ambiguous_math` warning each) and parse, since bar fences nest
/// no deeper than Perl's `MAX_ABS_DEPTH` (before, the first passed 8 GB uncapped); on a smaller budget
/// they spend it and stay flagged unparsed, keeping none of the readings of a traversal cut short inside
/// the root.
#[test]
fn opfunction_chain_parses_past_the_tree_sampler() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/opfunction_chain_parses_past_the_tree_sampler.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 2, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:ambiguous_math:").count(),
    2,
    "{stderr}"
  );
  for (id, math) in [
    (
      "S0.Ex1.m1",
      r##"<Math mode="display" tex="\log x\,\log y\,\log z\,\log w\,\log v\,\log u\,\log t" text="logarithm@(x) * logarithm@(y) * logarithm@(z) * logarithm@(w) * logarithm@(v) * logarithm@(u) * logarithm@(t)" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">x</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">y</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">z</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">w</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">v</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">u</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">t</XMTok></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex2.m1",
      r##"<Math mode="display" tex="\operatorname{tr}A\operatorname{tr}B\operatorname{tr}C\operatorname{tr}D\operatorname{tr}E\operatorname{tr}F\operatorname{tr}G" text="tr@(A) * tr@(B) * tr@(C) * tr@(D) * tr@(E) * tr@(F) * tr@(G)" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">A</XMTok></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">B</XMTok></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">C</XMTok></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">D</XMTok></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">E</XMTok></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">F</XMTok></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">G</XMTok></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex3.m1",
      r##"<Math mode="display" tex="|a||b||c||d||e||f||g||h||i||j||k|+\log a\,\log b\,\log c" text="absolute-value@(a) * absolute-value@(b) * absolute-value@(c) * absolute-value@(d) * absolute-value@(e) * absolute-value@(f) * absolute-value@(g) * absolute-value@(h) * absolute-value@(i) * absolute-value@(j) * absolute-value@(k) + logarithm@(a) * logarithm@(b) * logarithm@(c)" xml:id="S0.Ex3.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.1">a</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.2">b</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.3">c</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.4">d</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.5"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.5">e</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.6"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.6">f</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.7"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.7">g</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.8"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.8">h</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.9"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.9">i</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.10"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.10">j</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.11"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.11">k</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
    ),
    (
      "S0.Ex4.m1",
      r##"<Math mode="display" tex="\log a\,\log b\,\log c\,\log d\,\log e\,\log f\,\log g\,\log h\,\log i\,\log j\,\log k\,\log l\,\log m\,\log n\,\log o\,\log p+|a|_{L}|b|_{L}+|c|_{L}|d|_{L}+|e|_{L}|f|_{L}+|g|_{L}+|z|^{2}" text="logarithm@(a) * logarithm@(b) * logarithm@(c) * logarithm@(d) * logarithm@(e) * logarithm@(f) * logarithm@(g) * logarithm@(h) * logarithm@(i) * logarithm@(j) * logarithm@(k) * logarithm@(l) * logarithm@(m) * logarithm@(n) * logarithm@(o) * logarithm@(p) + (absolute-value@(a)) _ L * (absolute-value@(b)) _ L + (absolute-value@(c)) _ L * (absolute-value@(d)) _ L + (absolute-value@(e)) _ L * (absolute-value@(f)) _ L + (absolute-value@(g)) _ L + (absolute-value@(z)) ^ 2" xml:id="S0.Ex4.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">c</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">d</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">e</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">f</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">g</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">h</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">i</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">j</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">k</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">l</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">m</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">n</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">o</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.1">a</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.2">b</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.3">c</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.4">d</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.5"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.5">e</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.6"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.6">f</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.7"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.7">g</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.8"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.8">z</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
  // The second chance's work is bounded: below what the bar formula needs, it spends the budget,
  // keeps none of the readings — cut short, they are an arbitrary part of the whole — and the formula
  // stays flagged unparsed; the sixteen logs beside bars, flat since 57cb, need no second chance but
  // their bars' ambiguity (one `ambiguous_math` warning), read from a sample the small budget cuts.
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
  assert_eq!(warning_count(&stderr), 2, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:unparsed_math:").count(),
    1,
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex3.m1""#],
    r##"<Math class="ltx_math_unparsed" mode="display" tex="|a||b||c||d||e||f||g||h||i||j||k|+\log a\,\log b\,\log c" xml:id="S0.Ex3.m1"><XMath><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">g</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">h</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">i</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">j</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN">k</XMTok><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math>"##,
  );
  // … and the sixteen logs keep the reading the cut sample happens to rank first — pinned so a change
  // to that path is seen, not endorsed: its bars read an evaluation bar and a `limit-from` that the
  // full budget (above) does not (57cb review).
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex4.m1""#],
    r##"<Math mode="display" tex="\log a\,\log b\,\log c\,\log d\,\log e\,\log f\,\log g\,\log h\,\log i\,\log j\,\log k\,\log l\,\log m\,\log n\,\log o\,\log p+|a|_{L}|b|_{L}+|c|_{L}|d|_{L}+|e|_{L}|f|_{L}+|g|_{L}+|z|^{2}" text="logarithm@(a) * logarithm@(b) * logarithm@(c) * logarithm@(d) * logarithm@(e) * logarithm@(f) * logarithm@(g) * logarithm@(h) * logarithm@(i) * logarithm@(j) * logarithm@(k) * logarithm@(l) * logarithm@(m) * logarithm@(n) * logarithm@(o) * logarithm@(p) + (absolute-value@(a)) _ L * (absolute-value@(b)) _ L + (absolute-value@(c)) _ L * (absolute-value@(d)) _ L + evaluated-at@((absolute-value@(e)) _ L * absolute-value@(limit-from@(evaluated-at@(f, L), +)) * g, L) + (absolute-value@(z)) ^ 2" xml:id="S0.Ex4.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">b</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">c</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">d</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">e</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">f</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">g</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">h</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">i</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">j</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">k</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">l</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">m</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">n</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">o</XMTok></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.1">a</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.2">b</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.3">c</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.4">d</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp></XMApp><XMDual><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="S0.Ex4.m1.5"/><XMRef idref="S0.Ex4.m1.10"/></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMApp xml:id="S0.Ex4.m1.5"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.6"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.6">e</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" fontsize="70%" role="UNKNOWN">L</XMTok></XMApp><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.7"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMApp xml:id="S0.Ex4.m1.7"><XMTok meaning="limit-from"/><XMDual><XMApp><XMTok meaning="evaluated-at"/><XMRef idref="S0.Ex4.m1.8"/><XMRef idref="S0.Ex4.m1.9"/></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.8">f</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.Ex4.m1.9">L</XMTok></XMApp></XMDual><XMTok meaning="plus" role="ADDOP">+</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">g</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap><XMTok font="italic" fontsize="70%" role="UNKNOWN" xml:id="S0.Ex4.m1.10">L</XMTok></XMApp></XMDual><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex4.m1.11"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.11">z</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
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
      r##"<Math mode="display" tex="\log|a|\,\log|b|\,\log|c|\,\log|d|" text="logarithm@(absolute-value@(a)) * logarithm@(absolute-value@(b)) * logarithm@(absolute-value@(c)) * logarithm@(absolute-value@(d))" xml:id="S0.Ex3.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.1">a</XMTok><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.2">b</XMTok><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.3"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.3">c</XMTok><XMTok role="CLOSE" rpadding="1.7pt" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp><XMApp><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex3.m1.4"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.4">d</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap></XMDual></XMApp></XMApp></XMath></Math>"##,
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

/// A bare conditional binds its adjacent factors, as Perl's `moreFactors : evalAtOp ExpressionsNoBars`
/// (MathGrammar:261-268) does: a relation that is no event after it relates the whole conditional, so `\sim` ends
/// the condition before the statements that follow (2605.05396, 2605.13128, 2605.19519, 2605.03152; were unparsed).
/// Two `ambiguous_math` warnings: the statement-level bar enumerates its refused condition spans (14 and 11 raw
/// trees; one reading, and two for the #18 `Cat` choice the pragmas settle; SYNC_STATUS "Enumeration residuals").
#[test]
fn conditional_ends_before_the_next_statement() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/conditional_ends_before_the_next_statement.tex"
    ),
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
      r##"<Math mode="inline" tex="y_{i}|\theta_{i}\sim P,\quad i=1,\ldots,n" text="formulae@(conditional@(y _ i, theta _ i) similar-to P, i = list@(1, ldots, n))" xml:id="p1.m1"><XMath><XMDual><XMApp><XMTok meaning="formulae"/><XMRef idref="p1.m1.1"/><XMRef idref="p1.m1.2"/></XMApp><XMWrap><XMApp xml:id="p1.m1.1"><XMTok meaning="similar-to" name="sim" role="RELOP">∼</XMTok><XMApp><XMTok meaning="conditional" role="MODIFIEROP" stretchy="false">|</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="theta" role="UNKNOWN">θ</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">P</XMTok></XMApp><XMTok role="PUNCT" rpadding="10.0pt">,</XMTok><XMApp xml:id="p1.m1.2"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">i</XMTok><XMDual><XMApp><XMTok meaning="list"/><XMRef idref="p1.m1.3"/><XMRef idref="p1.m1.4"/><XMRef idref="p1.m1.5"/></XMApp><XMWrap><XMTok meaning="1" role="NUMBER" xml:id="p1.m1.3">1</XMTok><XMTok role="PUNCT">,</XMTok><XMTok name="ldots" role="ID" xml:id="p1.m1.4">…</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.5">n</XMTok></XMWrap></XMDual></XMApp></XMWrap></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m2",
      r##"<Math mode="inline" tex="\pi_{i}\mid\alpha\sim\mathrm{Cat}(\alpha),\quad i=1,2" text="list@(conditional@(pi _ i, alpha) similar-to Cat@(alpha), i = 1, 2)" xml:id="p1.m2"><XMath><XMDual><XMApp><XMTok meaning="list"/><XMRef idref="p1.m2.1"/><XMRef idref="p1.m2.3"/><XMRef idref="p1.m2.4"/></XMApp><XMWrap><XMApp xml:id="p1.m2.1"><XMTok meaning="similar-to" name="sim" role="RELOP">∼</XMTok><XMApp><XMTok meaning="conditional" name="mid" role="MODIFIEROP">∣</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="pi" role="UNKNOWN">π</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMTok font="italic" name="alpha" role="UNKNOWN">α</XMTok></XMApp><XMApp><XMTok role="UNKNOWN">Cat</XMTok><XMDual><XMRef idref="p1.m2.2"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" name="alpha" role="UNKNOWN" xml:id="p1.m2.2">α</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMApp><XMTok role="PUNCT" rpadding="10.0pt">,</XMTok><XMApp xml:id="p1.m2.3"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">i</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="PUNCT">,</XMTok><XMTok meaning="2" role="NUMBER" xml:id="p1.m2.4">2</XMTok></XMWrap></XMDual></XMath></Math>"##,
    ),
    (
      "p1.m3",
      r##"<Math mode="inline" tex="y_{i}|\theta_{i}\sim P,\ i=1,\ldots,n" text="formulae@(conditional@(y _ i, theta _ i) similar-to P, i = list@(1, ldots, n))" xml:id="p1.m3"><XMath><XMDual><XMApp><XMTok meaning="formulae"/><XMRef idref="p1.m3.1"/><XMRef idref="p1.m3.2"/></XMApp><XMWrap><XMApp xml:id="p1.m3.1"><XMTok meaning="similar-to" name="sim" role="RELOP">∼</XMTok><XMApp><XMTok meaning="conditional" role="MODIFIEROP" stretchy="false">|</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="theta" role="UNKNOWN">θ</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">P</XMTok></XMApp><XMTok role="PUNCT" rpadding="5.0pt">,</XMTok><XMApp xml:id="p1.m3.2"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">i</XMTok><XMDual><XMApp><XMTok meaning="list"/><XMRef idref="p1.m3.3"/><XMRef idref="p1.m3.4"/><XMRef idref="p1.m3.5"/></XMApp><XMWrap><XMTok meaning="1" role="NUMBER" xml:id="p1.m3.3">1</XMTok><XMTok role="PUNCT">,</XMTok><XMTok name="ldots" role="ID" xml:id="p1.m3.4">…</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m3.5">n</XMTok></XMWrap></XMDual></XMApp></XMWrap></XMDual></XMath></Math>"##,
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

/// 57by: a float that collapses into its one inner float takes that float's attributes (Perl collapseFloat,
/// latex_constructs.pool.ltxml:3447-3449, KNOWN_PERL_ERRORS #388) — but an inner float beside other
/// content is one panel of the outer, and its box geometry and box/panel classes describe the panel: a side
/// caption's `0.3\linewidth` minipage gave its figure `width="103.5pt"` around a `0.65\linewidth` panel
/// (2605.03502), and a `0.4\linewidth` subfigure of a `0.1\linewidth` caption minipage and a panel claimed
/// `width="34.5pt"`, so three of them shared one row (2605.15932); now the third starts a row, as pdflatex.
/// A lone inner float still gives its float its geometry, and classes merge in both cases: the algorithm
/// keeps `ltx_float_algorithm` (2605.17037). Each keeps the id its tags carry (`S0.F1`, `S0.F2.sf1`,
/// `algorithm1`; 57cp, #372). OXIDIZED_DESIGN_DIVERGENCES #375; repro captions-floats/collapsed_panel_keeps_the_float_geometry.
#[test]
fn collapsed_panel_keeps_the_float_geometry() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/collapsed_panel_keeps_the_float_geometry.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" labels="LABEL:f" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p class="ltx_figure_panel ltx_minipage" vattach="middle" width="224.3pt">Wide panel.</p><toccaption><tag close=" ">1</tag>Side caption.</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Side caption.</text></caption></figure>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    r#"<figure inlist="lof" xml:id="S0.F2"><tags><tag><text fontsize="90%">Figure 2</text></tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><figure class="ltx_figure_panel" inlist="lof" labels="LABEL:a" xml:id="S0.F2.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">2a</tag></tags><toccaption><tag close=" ">a</tag></toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag></caption><p class="ltx_figure_panel ltx_minipage" vattach="middle" width="276.0pt">One.</p></figure><figure class="ltx_figure_panel" inlist="lof" labels="LABEL:b" xml:id="S0.F2.sf2"><tags><tag><text fontsize="90%">(b)</text></tag><tag role="refnum">2b</tag></tags><toccaption><tag close=" ">b</tag></toccaption><caption><tag close=" "><text fontsize="90%">(b)</text></tag></caption><p class="ltx_figure_panel ltx_minipage" vattach="middle" width="276.0pt">Two.</p></figure><break class="ltx_break"/><figure class="ltx_figure_panel" inlist="lof" labels="LABEL:c" xml:id="S0.F2.sf3"><tags><tag><text fontsize="90%">(c)</text></tag><tag role="refnum">2c</tag></tags><toccaption><tag close=" ">c</tag></toccaption><caption><tag close=" "><text fontsize="90%">(c)</text></tag></caption><p class="ltx_figure_panel ltx_minipage" vattach="middle" width="276.0pt">Three.</p></figure><toccaption><tag close=" ">2</tag>Rows.</toccaption><caption><tag close=": "><text fontsize="90%">Figure 2</text></tag><text fontsize="90%">Rows.</text></caption></figure>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "float",
    &[r#"xml:id="algorithm1""#],
    r#"<float class="ltx_float_algorithm ltx_minipage" framed="top" inlist="loa" labels="LABEL:alg" vattach="middle" width="345.0pt" xml:id="algorithm1"><tags><tag><text font="bold">Algorithm 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Algorithm 1</tag></tags><toccaption><tag close=" ">1</tag>Alg</toccaption><caption><tag close=" "><text font="bold">Algorithm 1</text></tag> Alg</caption><p framed="topbottom">Step.</p></float>"#,
  );
}

/// 57cd (user ruling 2026-09-30): ℙ is an operator only where it is applied, a group after it —
/// physics' `\qty(A)` too, a group built whole (`parser::opens_a_group`): ℙ@(A), where a bare letter
/// after it (`\mathbb{P}X`) multiplies.
#[test]
fn probability_applies_to_a_physics_group() {
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\\usepackage{amssymb,physics}\\begin{document}\n\
     $\\mathbb{P}\\qty(A)$ $\\mathbb{P}X$\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, math) in [
    (
      "p1.m1",
      r##"<Math mode="inline" tex="\mathbb{P}\quantity(A)" text="P@(A)" xml:id="p1.m1"><XMath><XMApp><XMTok font="blackboard" role="UNKNOWN">P</XMTok><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">A</XMTok><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"##,
    ),
    (
      "p1.m2",
      r##"<Math mode="inline" tex="\mathbb{P}X" text="P * X" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="blackboard" role="UNKNOWN">P</XMTok><XMTok font="italic" role="UNKNOWN">X</XMTok></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57cd review: an expectation or probability named by `\lxDefMath` (a `meaning` or `name`) is typed by
/// its content, so it applies — expectation@(X), prob@(A) — where 57cd's lexeme took the name and
/// matched no terminal (every formula unparsed).
#[test]
fn named_expectation_tokens_apply() {
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\\usepackage{amssymb,latexml}\n\
     \\lxDefMath{\\EE}{𝔼}[meaning=expectation]\\lxDefMath{\\PP}{ℙ}[name=prob]\n\
     \\begin{document}\n$\\nabla\\EE[X]$ $a+\\PP(A)$\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, math) in [
    (
      "p1.m1",
      r##"<Math mode="inline" tex="\nabla\EE[X]" text="nabla@(expectation@(X))" xml:id="p1.m1"><XMath><XMApp><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMDual><XMApp><XMRef idref="p1.m1.1"/><XMRef idref="p1.m1.2"/></XMApp><XMApp><XMTok meaning="expectation" name="EE" role="UNKNOWN" xml:id="p1.m1.1">𝔼</XMTok><XMWrap><XMTok role="OPEN" stretchy="false">[</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.2">X</XMTok><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMApp></XMDual></XMApp></XMath></Math>"##,
    ),
    (
      "p1.m2",
      r##"<Math mode="inline" tex="a+\PP(A)" text="a + prob@(A)" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMDual><XMApp><XMRef idref="p1.m2.1"/><XMRef idref="p1.m2.2"/></XMApp><XMApp><XMTok name="prob" role="UNKNOWN" xml:id="p1.m2.1">ℙ</XMTok><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.2">A</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMApp></XMDual></XMApp></XMath></Math>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57ce: a register command on a non-register is one error and returns, reading no `by` and no
/// operand (tex.web §1236-1237), as pdflatex: three errors, "A by 2 B by 2 C by 2 D" (KNOWN_PERL_ERRORS
/// #390: Perl defines `\relax` a register and reads on).
#[test]
fn register_command_on_a_non_register_is_one_error() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/register_command_on_a_non_register.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 3, "{stderr}");
  assert_eq!(
    stderr.matches("Error:expected:<variable>").count(),
    3,
    "{stderr}"
  );
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1"><p>A by 2 B by 2 C by 2 D</p></para>"##,
  );
  // the `\afterassignment` token fires where the command returns, before `by` (§1269; 57ce review)
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\\begin{document}\n\
     \\def\\x{[X]}\\afterassignment\\x\\advance\\relax by 2\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1"><p>[X]by 2</p></para>"##,
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
    enumerations,
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
    postfix_operands,
    rust_parse_additions,
    scripted_operator,
    trailing_punctuation,
  );
}

/// 57cl: a picture TeX copies (`\usebox`, expl3's `\box_use:N` = `\copy`, tex.web:20952) repeats the svg
/// ids the pgf driver fixed at digestion; each outermost svg root now makes its ids unique in the document
/// and rebinds its `url(#…)` references (`Document::record_svg_ids`). suanpan-l3 had 9,480 jing lines,
/// thuaslogos-doc-english/-dutch 4 each; Perl writes the duplicates (KNOWN_PERL_ERRORS #391). Two
/// standalone copies and two copies nested in one picture. Repro
/// graphics-tikz/copied_picture_keeps_unique_svg_ids.
#[test]
fn copied_pictures_keep_unique_svg_ids() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/copied_picture_keeps_unique_svg_ids.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let mut ids: Vec<&str> = xml
    .split(" id=\"")
    .skip(1)
    .map(|rest| &rest[..rest.find('"').unwrap()])
    .collect();
  ids.sort_unstable();
  assert_eq!(
    ids,
    [
      "pgfcp1", "pgfcp1a", "pgfcp1b", "pgfcp1c", "pgfsh2", "pgfsh2a", "pgfsh2b", "pgfsh2c"
    ],
    "{xml}"
  );
  for id in ids {
    assert_eq!(
      xml.matches(&format!("url(#{id})")).count(),
      1,
      "each id referenced once, by its own copy: {id}\n{xml}"
    );
  }
  // The first copy keeps its ids, and each copy's references follow its own definitions.
  let first = xml.find("id=\"pgfcp1\"").unwrap();
  let second = xml.find("id=\"pgfcp1a\"").unwrap();
  assert!(first < xml.find("url(#pgfcp1)").unwrap() && xml.find("url(#pgfcp1)").unwrap() < second);
  assert!(second < xml.find("url(#pgfcp1a)").unwrap());
}

/// 57cl.1: a picture the math parser or an alignment re-creates (`append_tree`/`append_clone` copy
/// its attributes, `_svgid` mark included, into new nodes, each closing a new `svg:svg`) is the same
/// picture and keeps its ids; only a TeX copy is renamed, and a `use` element's `xlink:href` — set as
/// a literal prefixed name — follows its copy's path. 57cl renamed the inline-math picture to
/// `pgfcp1b`/`pgfpath1b` and left `xlink:href="#pgfpath1"` dangling. Repro
/// graphics-tikz/rebuilt_math_picture_keeps_its_svg_ids.
#[test]
fn rebuilt_math_pictures_keep_their_svg_ids() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/rebuilt_math_picture_keeps_its_svg_ids.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  assert!(
    !xml.contains("_svgid"),
    "the bookkeeping mark leaked:\n{xml}"
  );
  let mut ids: Vec<&str> = xml
    .split(" id=\"")
    .skip(1)
    .map(|rest| &rest[..rest.find('"').unwrap()])
    .collect();
  ids.sort_unstable();
  assert_eq!(
    ids,
    [
      "pgfcp1",
      "pgfcp2",
      "pgfcp2a",
      "pgfpath1",
      "pgfpath2",
      "pgfpath2a"
    ],
    "{xml}"
  );
  let uses: Vec<&str> = xml
    .match_indices("<svg:use ")
    .map(|(at, _)| &xml[at..at + xml[at..].find("/>").unwrap() + 2])
    .collect();
  assert_eq!(
    uses,
    [
      r##"<svg:use style="fill:none" xlink:href="#pgfpath1"/>"##,
      r##"<svg:use style="fill:none" xlink:href="#pgfpath2"/>"##,
      r##"<svg:use style="fill:none" xlink:href="#pgfpath2a"/>"##,
    ],
    "{xml}"
  );
}

/// Whole-element pins for the 57cn repros: each converts error-free and warning-free, the XML
/// is schema-valid where jing is installed, and the named element is exactly as pinned.
fn assert_repro(tex: &str, preload: &str, tag: &str, attrs: &[&str], expected: &str) {
  let xml = assert_elements(tex, preload, (0, 0), &[]);
  latexml::util::test::assert_element(&xml, tag, attrs, expected);
}

/// 57cn: glossaries-extra's `\glshyperlink` points at the anchor `\glstarget` makes — the binding's
/// `\glsdisablehyper` disables the links but no longer the targets (glossaries.sty:4302-4306,
/// glossaries-extra.sty:5204-5210, :6558; KNOWN_PERL_ERRORS #392). glossariesbegin 44 → 0 and
/// mfirstuc-manual 32 → 0 jing lines. Repro index/glshyperlink_target_resolves.
#[test]
fn glshyperlink_target_resolves() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index/glshyperlink_target_resolves.tex");
  assert_repro(
    tex,
    "[rawstyles,rawclasses]latexml.sty",
    "para",
    &["xml:id=\"p1\""],
    r##"<para xml:id="p1"><p>Definition: <text yoffset="6.8pt"><anchor xml:id="glo..foo"/></text>Foo.</p></para>"##,
  );
  assert_repro(
    tex,
    "[rawstyles,rawclasses]latexml.sty",
    "para",
    &["xml:id=\"p2\""],
    r##"<para xml:id="p2"><p>Later: <ref idref="glo..foo">foo</ref> and <glossaryref inlist="main" key="foo">foo</glossaryref>.</p></para>"##,
  );
}

/// 57cn: `\Hy@raisedlink` keeps its argument (hyperref.sty:2100-2118, hdvips.def:40) and
/// `\hyper@@anchor` is `\hypertarget` (hyperref.sty:5121) — cms-noteref-demo 18 → 0 and arXiv
/// 2308.06254 43 → 0 dangling idrefs. Repro singletons/hy_raisedlink_keeps_its_anchor.
#[test]
fn hy_raisedlink_keeps_its_anchor() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/hy_raisedlink_keeps_its_anchor.tex"
  );
  assert_repro(
    tex,
    "[rawstyles,rawclasses]latexml.sty",
    "para",
    &["xml:id=\"p1\""],
    r##"<para xml:id="p1"><p>Text<anchor xml:id="Hendnotepage.1"/><sup>1</sup>.</p></para>"##,
  );
}

/// 57cn: a `\hypertarget` name has no size (the anchor is a zero-size whatsit); pdflatex's widths.
/// Repro boxes-groups/hypertarget_has_no_size.
#[test]
fn hypertarget_has_no_size() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/hypertarget_has_no_size.tex");
  assert_repro(
    tex,
    "[rawstyles,rawclasses]latexml.sty",
    "para",
    &["xml:id=\"p1\""],
    r##"<para xml:id="p1"><p>W=0.0pt, H=0.0pt, SW=0.0pt, SH=0.0pt.</p></para>"##,
  );
}

/// 57cn: `\floatsetup{font=…}` defines `\floatfont` (caption3.sty:850-852 `\caption@setfont`), so a
/// floatrow float holds no ERROR (kaytannollista-latexia, floatrow-rus, makecell-rus). The float's
/// number is RED captions-floats/floatrow_floatbox_steps_its_counter_once, so the pin is the body, and
/// the ERROR check is the whole document's.
/// Repro captions-floats/floatrow_font_option_defines_floatfont.
#[test]
fn floatrow_font_option_defines_floatfont() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/floatrow_font_option_defines_floatfont.tex"
  );
  assert_repro(
    tex,
    "[rawstyles,rawclasses]latexml.sty",
    "p",
    &["align=\"center\""],
    r##"<p align="center" vattach="bottom">Body</p>"##,
  );
  let (_, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert!(!xml.contains("<ERROR"), "{xml}");
}

/// 57cn: unicode-math's Greek names resolve at `\begin{document}` (unicode-math-luatex.sty:3719-3734)
/// — kaytannollista-latexia's 12 undefined `\Alpha`…`\Chi` — unconditionally, as unicode-math sets
/// them (57cp: a preamble `\renewcommand{\epsilon}` gives way, as under lualatex). Repro
/// luatex-profile/unicode_math_greek_names.
#[test]
fn unicode_math_greek_names() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/luatex-profile/unicode_math_greek_names.tex"
  );
  assert_elements(tex, "[luatex,rawstyles,rawclasses]latexml.sty", (0, 0), &[
    (
      "Math",
      "p1.m1",
      r##"<Math mode="inline" tex="\Alpha\Beta\Gamma\omicron\Omicron" text="Alpha * Beta * Gamma * omicron * Omicron" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok name="Alpha" role="UNKNOWN">Α</XMTok><XMTok name="Beta" role="UNKNOWN">Β</XMTok><XMTok name="Gamma" role="UNKNOWN">Γ</XMTok><XMTok font="italic" name="omicron" role="UNKNOWN">ο</XMTok><XMTok name="Omicron" role="UNKNOWN">Ο</XMTok></XMApp></XMath></Math>"##,
    ),
    // `\up<name>`/`\it<name>` (:3732-3733): the letter upright and italic (57cn.1).
    (
      "Math",
      "p2.m1",
      r##"<Math mode="inline" tex="\upmu\itbeta" text="upmu * itbeta" xml:id="p2.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok name="upmu" role="UNKNOWN">μ</XMTok><XMTok font="italic" name="itbeta" role="UNKNOWN">β</XMTok></XMApp></XMath></Math>"##,
    ),
    // set unconditionally (57cp): a preamble redefinition gives way, as under lualatex
    (
      "Math",
      "p3.m1",
      r##"<Math mode="inline" tex="\epsilon\Rho" text="epsilon * Rho" xml:id="p3.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" name="epsilon" role="UNKNOWN">ϵ</XMTok><XMTok name="Rho" role="UNKNOWN">Ρ</XMTok></XMApp></XMath></Math>"##,
    ),
  ]);
}

/// 57cn.1: footnotehyper's saved notes anchor `\Hy@footnote@currentHref` (hyperref.sty:6114, `\@empty`)
/// through `\Hy@raisedlink`, whose argument runs since 57cn; an empty name anchors nothing
/// (hyperref.sty:5122-5124). The footnotehyper manual had gained
/// `Error:undefined:\Hy@footnote@currentHref` and anchors named after it. Both notes are pinned, the
/// savenotes one too (57cp). Repro singletons/footnote_anchor_name_is_defined.
#[test]
fn footnote_anchor_name_is_defined() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/footnote_anchor_name_is_defined.tex"
  );
  // `mark="0"` is the footnote counter where LaTeX prints no mark: the blank-footnote idiom's
  // residual, shared with Perl (KNOWN_PERL_ERRORS #394).
  let xml = assert_elements(tex, RAW, (0, 0), &[
    (
      "note",
      "footnotex1",
      r##"<note mark="0" role="footnotetext" xml:id="footnotex1"><inline-block vattach="bottom"><rule height="0.0pt" width="0.0pt"/><p><text fontsize="80%">Raw footnotetext in table.</text></p></inline-block></note>"##,
    ),
    (
      "note",
      "footnotex2",
      r##"<note mark="0" role="footnotetext" xml:id="footnotex2"><inline-block vattach="bottom"><rule height="0.0pt" width="0.0pt"/><p><text fontsize="80%">Saved raw.</text></p></inline-block></note>"##,
    ),
  ]);
  assert!(
    !xml.contains("currentHref"),
    "an anchor named after the control sequence:\n{xml}"
  );
}

/// 57co: tabularray's libraries run (`\UseTblrLibrary`, tabularray.sty:8036-8050; `\NewTblrLibrary`
/// registers packages' own), its public variables, `\TblrNote` and ninecolors exist, and a
/// `\NewTblrEnviron` name is expanded. The tabularray manual 26 → 7 errors, tikzfill 6 → 0, dlrg 7 → 2.
/// 57cp (57co review): a math table has no margin columns, `\morecmidrules` keeps booktabs' meaning,
/// `+array` keeps its colspec, a text-mode `+matrix` is a text table, a table in a table's cell keeps
/// `\hline`, a tikz overlay is dropped with a warning. Every table is pinned whole.
/// Repro alignment-bindings/tabularray_libraries_and_public_variables (15 errors before 57co).
#[test]
fn tabularray_libraries_and_public_variables() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_libraries_and_public_variables.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (0, 1),
    &["The tblrtikzabove drawing is not rendered"],
    &[
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><p><Math mode="inline" tex="\begin{pmatrix}1&amp;2\\&#10;3&amp;4\\&#10;\end{pmatrix}" text="matrix@(Array[[1, 2], [3, 4]])" xml:id="p1.m1"><XMath><XMDual><XMApp><XMTok meaning="matrix"/><XMRef idref="p1.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMArray xml:id="p1.m1.1"><XMRow><XMCell align="center"><XMTok meaning="1" role="NUMBER">1</XMTok></XMCell><XMCell align="center"><XMTok meaning="2" role="NUMBER">2</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok meaning="3" role="NUMBER">3</XMTok></XMCell><XMCell align="center"><XMTok meaning="4" role="NUMBER">4</XMTok></XMCell></XMRow></XMArray><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMath></Math></p></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para xml:id="p2"><tabular class="ltx_guessed_headers" vattach="middle"><thead><tr><td align="left" border="tt" thead="column">A</td><td align="left" border="tt" thead="column">B</td><td align="left" border="tt" thead="column">C</td></tr></thead><tbody><tr><td align="left" border="bb t">D</td><td align="left" border="bb tt">E</td><td align="left" border="bb t">F<sup>a</sup></td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p3",
        r##"<para xml:id="p3"><p><Math mode="inline" tex="\begin{array}[]{|c|cc|}\diagbox[]{{\shortstack[l]{$X_{1}$}}}{{\shortstack[r]{$X_{2}$}}}&amp;0&amp;1\\&#10;0&amp;0.1&amp;0.2\\&#10;\end{array}" text="Array[[[X1X2], 0, 1], [0, 0.1, 0.2]]" xml:id="p3.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center" border="l r"><XMText class="ltx_nopad"><picture height="23.08" width="37.52" xml:id="p3.m1.1"><line points="0,23.08 37.52,0" stroke="#000000" stroke-width="0.4"/><g class="ltx_svg_fog" innerheight="11.54" innerwidth="18.76" transform="translate(0,0)"><inline-block><inline-block align="left"><p><Math mode="inline" tex="X_{1}" text="X _ 1" xml:id="p3.m1.pic1.m1"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">X</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math></p></inline-block></inline-block></g><g class="ltx_svg_fog" innerheight="11.54" innerwidth="18.76" transform="translate(18.76,11.54)"><inline-block><inline-block align="right"><p><Math mode="inline" tex="X_{2}" text="X _ 2" xml:id="p3.m1.pic1.m2"><XMath><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">X</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></p></inline-block></inline-block></g></picture></XMText></XMCell><XMCell align="center"><XMTok meaning="0" role="NUMBER">0</XMTok></XMCell><XMCell align="center" border="r"><XMTok meaning="1" role="NUMBER">1</XMTok></XMCell></XMRow><XMRow><XMCell align="center" border="l r"><XMTok meaning="0" role="NUMBER">0</XMTok></XMCell><XMCell align="center"><XMTok meaning="0.1" role="NUMBER">0.1</XMTok></XMCell><XMCell align="center" border="r"><XMTok meaning="0.2" role="NUMBER">0.2</XMTok></XMCell></XMRow></XMArray></XMath></Math></p></para>"##,
      ),
      (
        "para",
        "p4",
        r##"<para xml:id="p4"><tabular vattach="middle"><tbody><tr><td align="center">Head</td><td align="center">Head</td></tr><tr><td align="center">111</td><td align="center">2.1</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p5",
        r##"<para xml:id="p5"><tabular vattach="middle"><tbody><tr><td align="left">A</td><td align="left">B</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p6",
        r##"<para xml:id="p6"><tabular class="ltx_guessed_headers" vattach="middle"><thead><tr><td align="left" border="tt" thead="column">G</td><td align="left" border="tt" thead="column">H</td></tr></thead><tbody><tr><td align="left" border="tt">I</td><td align="left" border="tt">J</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p7",
        r##"<para xml:id="p7"><p><Math mode="inline" tex="\begin{array}[]{cc}5&amp;6\\&#10;7&amp;8\end{array}" text="Array[[5, 6], [7, 8]]" xml:id="p7.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok meaning="5" role="NUMBER">5</XMTok></XMCell><XMCell align="center"><XMTok meaning="6" role="NUMBER">6</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok meaning="7" role="NUMBER">7</XMTok></XMCell><XMCell align="center"><XMTok meaning="8" role="NUMBER">8</XMTok></XMCell></XMRow></XMArray></XMath></Math></p></para>"##,
      ),
      (
        "para",
        "p8",
        r##"<para xml:id="p8"><p>Text: <tabular vattach="middle"><tbody><tr><td align="left">p</td><td align="left">q</td></tr></tbody></tabular> end.</p></para>"##,
      ),
      (
        "para",
        "p9",
        r##"<para xml:id="p9"><tabular vattach="middle"><tbody><tr><td align="left" border="b t">a</td><td align="left" border="b t"><tabular vattach="middle"><tr><td align="left" border="b t">x</td></tr></tabular></td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p10",
        r##"<para xml:id="p10"><p><text color="#6666DE">Blue</text> text.</p></para>"##,
      ),
    ],
  );
}

/// The raw-styles preload of the perfect-kernel protocol.
pub(super) const RAW: &str = "[rawstyles,rawclasses]latexml.sty";

/// Convert `tex` under `preload`: `errors` errors and `warnings` warnings, schema-valid, and each `(tag, xml:id,
/// element)` exactly as given. Returns the XML.
pub(super) fn assert_elements(
  tex: &str,
  preload: &str,
  (errors, warnings): (usize, usize),
  expected: &[(&str, &str, &str)],
) -> String {
  assert_elements_with(tex, preload, (errors, warnings), &[], expected)
}

/// [`assert_elements`], and each of `messages` is the text of one of the `Error:`/`Warning:` lines
/// counted (so a count cannot be met by a different diagnostic).
pub(super) fn assert_elements_with(
  tex: &str,
  preload: &str,
  (errors, warnings): (usize, usize),
  messages: &[&str],
  expected: &[(&str, &str, &str)],
) -> String {
  let (stderr, xml) = convert_with(tex, Some(preload));
  assert_eq!(error_count(&stderr), errors, "{stderr}");
  assert_eq!(warning_count(&stderr), warnings, "{stderr}");
  let diagnostics: Vec<&str> = stderr
    .lines()
    .filter(|l| l.starts_with("Error:") || l.starts_with("Warning:"))
    .collect();
  // a message listed n times is n diagnostics
  for message in messages {
    let wanted = messages.iter().filter(|m| *m == message).count();
    let found = diagnostics.iter().filter(|d| d.contains(message)).count();
    assert!(
      found >= wanted,
      "{wanted} diagnostics {message:?} expected, {found} found:\n{stderr}"
    );
  }
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  for (tag, id, element) in expected {
    latexml::util::test::assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
  xml
}

/// 57cp: a long or tall tabularray table prints its outer spec's caption (a numbered, listed,
/// labelled `table` caption; `entry=` the list form), notes and remarks (tabularray.sty:6384-6474,
/// :5895-6240); `label=none` drops the number and the tag, a theme that empties the head the caption
/// (the counter still steps), a talltblr in a `table` float captions that float, and a plain `tblr`
/// has no caption. 16 texts were lost in the manuals. Repro
/// alignment-bindings/tabularray_long_table_caption_notes.
#[test]
fn tabularray_long_table_caption_notes() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_long_table_caption_notes.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>See Table <ref labelref="LABEL:tab:a"/>.</p></para>"##,
    ),
    (
      "table",
      "S0.T1",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:tab:a" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Short A</toccaption><caption><tag close=": ">Table 1</tag>Cap A</caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">Alpha</td><td align="left">Beta<sup>a</sup></td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">a</text></sup> First note.</p><break class="ltx_break"/><p class="ltx_figure_panel"><text font="italic">Source</text>: Somewhere</p></table>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><tabular vattach="middle"><tbody><tr><td align="left">Plain</td><td align="left">One</td></tr></tbody></tabular></para>"##,
    ),
    (
      "table",
      "tab1",
      r##"<table class="ltx_tblr_box" inlist="lot" xml:id="tab1"><toccaption><tag close=" ">1</tag>Untagged</toccaption><caption>Untagged</caption><tabular vattach="middle"><tbody><tr><td align="left">Plain</td><td align="left">Two</td></tr></tbody></tabular></table>"##,
    ),
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><tabular vattach="middle"><tbody><tr><td align="left">Naked</td><td align="left">Three</td></tr></tbody></tabular></para>"##,
    ),
    (
      "table",
      "S0.T3",
      r##"<table align="center" class="ltx_tblr_box" inlist="lot" labels="LABEL:tab:t" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>Tall in float</toccaption><caption><tag close=": ">Table 3</tag>Tall in float</caption><tabular vattach="middle"><tbody><tr><td align="left">Tall</td><td align="left">Four</td></tr></tbody></tabular></table>"##,
    ),
    (
      "para",
      "p6",
      r##"<para xml:id="p6"><tabular vattach="middle"><tbody><tr><td align="left">Short</td><td align="left">Five</td></tr></tbody></tabular><p>Table <ref labelref="LABEL:tab:t"/>.</p></para>"##,
    ),
    (
      "table",
      "S0.T4",
      r##"<table class="ltx_tblr_box" framed="rectangle" inlist="lot" xml:id="S0.T4"><tags><tag>Table 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Table 4</tag></tags><toccaption><tag close=" ">4</tag>Boxed</toccaption><caption><tag close=": ">Table 4</tag>Boxed</caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">Box</td><td align="left">Six</td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">b</text></sup> Boxed note.</p></table>"##,
    ),
    (
      "table",
      "S0.T5",
      r##"<table class="ltx_tblr_box" inlist="lot" xml:id="S0.T5"><tags><tag>Table 5</tag><tag role="refnum">5</tag><tag role="typerefnum">Table 5</tag></tags><toccaption><tag close=" ">5</tag>Untagged too</toccaption><caption>Untagged too</caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">Hash</td><td align="left">Seven</td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">c</text></sup> At <ref class="ltx_nolink ltx_url" font="typewriter" href="http://x.org/#1x">http://x.org/#1x</ref></p></table>"##,
    ),
  ]);
}

/// 57cp (review, review 3): a colspec naming a column type tabularray does not know is its error, "Unknown
/// Column type S!" (tabularray.sty:3389-3410), for the first such type of the colspec, where tabularray
/// stops reading columns (`{lzr}` is `l`, the rest it typesets as stray text dropped); the table is
/// typeset — never with the key list around the colspec as the template, nor with the colspec as the
/// kernel's template (its "Unrecognized tabular template" warnings).
/// Repro alignment-bindings/tabularray_unknown_column_type.
#[test]
fn tabularray_unknown_column_type() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_unknown_column_type.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (5, 0),
    &[
      "Unknown Column type S!",
      "Unknown Column type z!",
      "Unknown Column type z!",
      "Unknown Column type z!",
      "Unknown Column type z!",
    ],
    &[
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="left">Head</td><td align="left">Head</td></tr><tr><td align="left">111</td><td align="left">2.1</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p3",
        r##"<para xml:id="p3"><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td><td align="left">c</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p4",
        r##"<para xml:id="p4"><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td><td align="left">c</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p5",
        r##"<para xml:id="p5"><p><Math mode="inline" tex="\begin{array}[]{l*{2}{l}}1&amp;2&amp;3\\&#10;\end{array}" text="Array[[1, 2, 3]]" xml:id="p5.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="left"><XMTok meaning="1" role="NUMBER">1</XMTok></XMCell><XMCell align="left"><XMTok meaning="2" role="NUMBER">2</XMTok></XMCell><XMCell align="left"><XMTok meaning="3" role="NUMBER">3</XMTok></XMCell></XMRow></XMArray></XMath></Math></p></para>"##,
      ),
    ],
  );
}

#[test]
fn tabularray_table_commands_and_key_lists() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_table_commands_and_key_lists.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="left" border="t">Alpha</td><td align="left" border="t">Beta</td></tr><tr><td align="left" border="t">Gamma</td><td align="left" border="t">Delta</td></tr></tbody></tabular></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left">One</td><td align="left">Two</td><td align="left">Three</td></tr></tbody></tabular></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><tabular vattach="middle"><tbody><tr><td align="left">Four</td><td align="left">Five</td></tr></tbody></tabular></para>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><tabular vattach="middle"><tbody><tr><td align="center">Six</td><td align="left">Seven</td></tr></tbody></tabular></para>"##,
    ),
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><tabular vattach="middle"><tbody><tr><td align="center">Head</td><td align="center">Head</td></tr><tr><td align="center">111</td><td align="center">2.1</td></tr></tbody></tabular></para>"##,
    ),
  ]);
}

/// 57cp (review 2): a long or tall tabularray table stays where it is written with its caption and
/// notes — in a quote, a footnote, `\fbox`, `\resizebox`, a tabular cell, a threeparttable, a
/// longtable cell; a long table after text ends the paragraph and stands as the table; a table in a
/// table's cell has its own caption and the outer table keeps its number, list line and label
/// (review 3: they went to the inner table, KNOWN_PERL_ERRORS #395); a box in `\resizebox` is as wide
/// as its table; a hidden caption keeps its number for `\ref`. Repro
/// alignment-bindings/tabularray_table_placement.
#[test]
fn tabularray_table_placement() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_table_placement.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><quote><p>Quote <inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>In quote</toccaption><caption><tag close=": ">Table 1</tag>In quote</caption><tabular vattach="middle"><tbody><tr><td align="left">A</td><td align="left">B</td></tr></tbody></tabular></table></inline-logical-block> end.</p></quote><p>Text<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Note <inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>In note</toccaption><caption><tag close=": ">Table 2</tag>In note</caption><tabular vattach="middle"><tbody><tr><td align="left">C</td><td align="left">D</td></tr></tbody></tabular></table></inline-logical-block> end.</note><inline-logical-block class="ltx_tblr_box" framed="rectangle"><table inlist="lot" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>In fbox</toccaption><caption><tag close=": ">Table 3</tag>In fbox</caption><tabular vattach="middle"><tbody><tr><td align="left">E</td><td align="left">F</td></tr></tbody></tabular></table></inline-logical-block><inline-block depth="210.4pt" height="30.4pt" width="172.5pt" xscale="4.38404272605046" xtranslate="66.6pt" yscale="4.38404272605046" ytranslate="-93.0pt"><inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T4"><tags><tag>Table 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Table 4</tag></tags><toccaption><tag close=" ">4</tag>Scaled</toccaption><caption><tag close=": ">Table 4</tag>Scaled</caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">G</td><td align="left">H</td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">a</text></sup> Scaled note.</p></table></inline-logical-block></inline-block><tabular vattach="middle"><tbody><tr><td align="left"><inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T5"><tags><tag>Table 5</tag><tag role="refnum">5</tag><tag role="typerefnum">Table 5</tag></tags><toccaption><tag close=" ">5</tag>In cell</toccaption><caption><tag close=": ">Table 5</tag>In cell</caption><tabular vattach="middle"><tr><td align="left">I</td><td align="left">J</td></tr></tabular></table></inline-logical-block></td></tr></tbody></tabular><inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T6"><tags><tag>Table 6</tag><tag role="refnum">6</tag><tag role="typerefnum">Table 6</tag></tags><toccaption><tag close=" ">6</tag>In threeparttable</toccaption><caption><tag close=": ">Table 6</tag>In threeparttable</caption><tabular vattach="middle"><tbody><tr><td align="left">K</td><td align="left">L</td></tr></tbody></tabular></table></inline-logical-block></p></para>"##,
    ),
    (
      "table",
      "S0.T7",
      r##"<table inlist="lot" xml:id="S0.T7"><tags><tag>Table 7</tag><tag role="refnum">7</tag><tag role="typerefnum">Table 7</tag></tags><tabular><tr><td align="left"><inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T8"><tags><tag>Table 8</tag><tag role="refnum">8</tag><tag role="typerefnum">Table 8</tag></tags><toccaption><tag close=" ">8</tag>In longtable</toccaption><caption><tag close=": ">Table 8</tag>In longtable</caption><tabular vattach="middle"><tr><td align="left">M</td><td align="left">N</td></tr></tabular></table></inline-logical-block></td></tr></tabular></table>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>Before the long table.</p></para>"##,
    ),
    (
      "table",
      "S0.T9",
      r##"<table class="ltx_tblr_box" inlist="lot" xml:id="S0.T9"><tags><tag>Table 9</tag><tag role="refnum">9</tag><tag role="typerefnum">Table 9</tag></tags><toccaption><tag close=" ">9</tag>Long after text</toccaption><caption><tag close=": ">Table 9</tag>Long after text</caption><tabular vattach="middle"><tbody><tr><td align="left">O</td><td align="left">P</td></tr></tbody></tabular></table>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><p>After the long table.<inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T10"><tags><tag>Table 10</tag><tag role="refnum">10</tag><tag role="typerefnum">Table 10</tag></tags><toccaption><tag close=" ">10</tag>Outer</toccaption><caption><tag close=": ">Table 10</tag>Outer</caption><tabular vattach="middle"><tbody><tr><td align="left">Q</td><td align="left"><inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T11"><tags><tag>Table 11</tag><tag role="refnum">11</tag><tag role="typerefnum">Table 11</tag></tags><toccaption><tag close=" ">11</tag>Inner</toccaption><caption><tag close=": ">Table 11</tag>Inner</caption><tabular vattach="middle"><tr><td align="left">R</td></tr></tabular></table></inline-logical-block></td></tr></tbody></tabular></table></inline-logical-block></p></para>"##,
    ),
    (
      "table",
      "S0.T12",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:hidden" xml:id="S0.T12"><tags><tag>Table 12</tag><tag role="refnum">12</tag><tag role="typerefnum">Table 12</tag></tags><toccaption><tag close=" ">12</tag></toccaption><tabular vattach="middle"><tbody><tr><td align="left">S</td><td align="left">T</td></tr></tbody></tabular></table>"##,
    ),
    (
      "para",
      "p6",
      r##"<para xml:id="p6"><p>See Table <ref labelref="LABEL:t:hidden"/>.</p></para>"##,
    ),
  ]);
}

/// 57cp (review 2, review 3): a math tabularray table has as many columns as its widest row (a row
/// wider than the colspec; a colspec-less table), from its body read ahead; a tall table in math keeps
/// its caption around the math cells; the read ahead ends at the end of an environment wrapping the
/// table (`\newenvironment{mymat}{\begin{tblr}{cc}}{\end{tblr}}`) and leaves a later verbatim and
/// `\verb` untouched (review 3: it ran to the end of the file), and counts a row's cells, not those of an
/// environment in a cell (review 4). The tall table is pinned, not its
/// `Math`, whose `text=` is Perl's `[textContent]` of the table (MathParser.pm:964-965). Repro
/// alignment-bindings/tabularray_math_tables.
#[test]
fn tabularray_math_tables() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_math_tables.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p><Math mode="inline" tex="\begin{array}[]{cc*{1}{c}}1&amp;2&amp;3\\&#10;4&amp;5\\&#10;\end{array}" text="Array[[1, 2, 3], [4, 5, ]]" xml:id="p1.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok meaning="1" role="NUMBER">1</XMTok></XMCell><XMCell align="center"><XMTok meaning="2" role="NUMBER">2</XMTok></XMCell><XMCell align="center"><XMTok meaning="3" role="NUMBER">3</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok meaning="4" role="NUMBER">4</XMTok></XMCell><XMCell align="center"><XMTok meaning="5" role="NUMBER">5</XMTok></XMCell><XMCell/></XMRow></XMArray></XMath></Math></p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p><Math mode="inline" tex="\begin{array}[]{*{2}{l}}6&amp;7\\&#10;8&amp;9\\&#10;\end{array}" text="Array[[6, 7], [8, 9]]" xml:id="p2.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="left"><XMTok meaning="6" role="NUMBER">6</XMTok></XMCell><XMCell align="left"><XMTok meaning="7" role="NUMBER">7</XMTok></XMCell></XMRow><XMRow><XMCell align="left"><XMTok meaning="8" role="NUMBER">8</XMTok></XMCell><XMCell align="left"><XMTok meaning="9" role="NUMBER">9</XMTok></XMCell></XMRow></XMArray></XMath></Math></p></para>"##,
    ),
    (
      "table",
      "p3.m1.1",
      r##"<table inlist="lot" xml:id="p3.m1.1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Math tall</toccaption><caption><tag close=": ">Table 1</tag>Math tall</caption><p><Math mode="inline" tex="\begin{array}[]{cc}1&amp;2\\&#10;\end{array}" text="Array[[1, 2]]" xml:id="S0.T1.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok meaning="1" role="NUMBER">1</XMTok></XMCell><XMCell align="center"><XMTok meaning="2" role="NUMBER">2</XMTok></XMCell></XMRow></XMArray></XMath></Math></p></table>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><p><Math mode="inline" tex="M=\begin{array}[]{cc}a&amp;b\\&#10;c&amp;d\\&#10;\end{array}" text="M = Array[[a, b], [c, d]]" xml:id="p4.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">M</XMTok><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMCell></XMRow></XMArray></XMApp></XMath></Math> and <Math mode="inline" tex="\begin{array}[]{*{3}{l}}x&amp;y&amp;z\\&#10;\begin{array}[]{*{1}{l}}p\\&#10;\end{array}&amp;q\\&#10;\end{array}" text="Array[[x, y, z], [Array[[p]], q, ]]" xml:id="p4.m2"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="left"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMCell><XMCell align="left"><XMTok font="italic" role="UNKNOWN">y</XMTok></XMCell><XMCell align="left"><XMTok font="italic" role="UNKNOWN">z</XMTok></XMCell></XMRow><XMRow><XMCell align="left"><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="left"><XMTok font="italic" role="UNKNOWN">p</XMTok></XMCell></XMRow></XMArray></XMCell><XMCell align="left"><XMTok font="italic" role="UNKNOWN">q</XMTok></XMCell><XMCell/></XMRow></XMArray></XMath></Math> here.</p></para>"##,
    ),
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><p><Math mode="inline" tex="\begin{array}[]{cc}\begin{array}[]{ccc}a&amp;b&amp;c\end{array}&amp;d\\&#10;e&amp;f\\&#10;\end{array}" text="Array[[Array[[a, b, c]], d], [e, f]]" xml:id="p5.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell></XMRow></XMArray></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">e</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">f</XMTok></XMCell></XMRow></XMArray></XMath></Math></p></para>"##,
    ),
    (
      "para",
      "p6",
      r##"<para xml:id="p6"><verbatim font="typewriter">x_1 &amp; y ~ % not a comment \end{tblr}</verbatim><p>Inline <verbatim font="typewriter">a%b#c</verbatim> done.</p></para>"##,
    ),
  ]);
}

#[test]
fn tabularray_undefined_theme() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_undefined_theme.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (2, 0),
    &[
      "Erroneous variable \\g__tblr_theme_nothere_code_tl used",
      "Erroneous variable \\g__tblr_theme_nothere_code_tl used",
    ],
    &[
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="left">A</td><td align="left">B</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left">C</td><td align="left">D</td></tr></tbody></tabular></para>"##,
      ),
    ],
  );
}

/// 57cp (review 2, review 3): a minipage holding a block `ltx:block` cannot take (a theorem, a float, a
/// sectioning unit) in a scaled box (`\resizebox`, `\scalebox`, `\rotatebox`: an `inline-block`) is
/// the first inline container the box holds that holds it all — Perl renames it in place to the
/// invalid `inline-block > logical-block` (TeX_Box.pool.ltxml:502-513; divergence #385); a minipage of
/// text or of demotable blocks keeps its shape. Repro
/// boxes-groups/minipage_blocks_in_a_scaled_box_are_inline.
#[test]
fn minipage_blocks_in_a_scaled_box_are_inline() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/minipage_blocks_in_a_scaled_box_are_inline.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>A: <inline-block depth="0.0pt" height="13.9pt" width="345.0pt" xscale="2" xtranslate="86.3pt" yscale="2" ytranslate="-3.5pt"><inline-logical-block class="ltx_minipage" vattach="middle" width="172.5pt"><theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" xml:id="Thmtheorem1"><tags><tag>Theorem 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Theorem 1</tag></tags><title class="ltx_runin"><tag><text font="bold">Theorem 1</text></tag><text font="bold">.</text></title><para xml:id="Thmtheorem1.p1"><p><text font="italic">Scaled theorem.</text></p></para></theorem></inline-logical-block></inline-block></p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>B: <inline-block depth="10.8pt" height="7.7pt" width="155.3pt" xscale="0.9" xtranslate="-8.6pt" yscale="0.9" ytranslate="1.0pt"><inline-logical-block class="ltx_minipage" vattach="middle" width="172.5pt"><table inlist="lot" placement="H" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><tabular class="ltx_centering" vattach="middle"><tbody><tr><td align="left">k</td></tr></tbody></tabular><toccaption class="ltx_centering"><tag close=" ">1</tag>H table</toccaption><caption class="ltx_centering"><tag close=": ">Table 1</tag>H table</caption></table></inline-logical-block></inline-block></p></para>"##,
    ),
    (
      "para",
      "p3",
      r##"<para xml:id="p3"><p>C: <inline-block angle="90" depth="0.0pt" height="85.4pt" innerdepth="1.9pt" innerheight="6.9pt" innerwidth="85.4pt" width="8.9pt" xtranslate="-38.2pt" ytranslate="-38.2pt"><inline-sectional-block class="ltx_minipage"><section xml:id="Sx1"><title>Rotated section</title><para xml:id="Sx1.p1"><p>Body.</p></para></section></inline-sectional-block></inline-block></p></para>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><p>D: <inline-block depth="1.9pt" height="6.9pt" width="345.0pt" xscale="1" xtranslate="0.0pt" yscale="1" ytranslate="0.0pt"><p class="ltx_minipage" vattach="middle" width="345.0pt">plain text</p></inline-block></p></para>"##,
    ),
    (
      "para",
      "p5",
      r##"<para xml:id="p5"><p>E: <inline-block depth="7.8pt" height="12.8pt" width="345.0pt" xscale="1" xtranslate="0.0pt" yscale="1" ytranslate="0.0pt"><block class="ltx_minipage" vattach="middle" width="345.0pt"><tabular vattach="middle"><tbody><tr><td align="left">k</td></tr></tbody></tabular><p>more</p></block></inline-block></p></para>"##,
    ),
  ]);
}

#[test]
fn hyperref_empty_anchor_and_mpfootnote() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/hyperref_empty_anchor_and_mpfootnote.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>Start Kept text. Also kept. End.
<inline-block class="ltx_minipage" vattach="middle" width="142.3pt"><p>Box text.<note mark="0" role="footnotetext" xml:id="footnotex1">Minipage note.</note></p></inline-block></p></para>"##,
  )]);
  assert!(!xml.contains("<anchor"), "an anchor without an id:\n{xml}");
}

/// 57cp (review 3): a float nested in a captioned float keeps the outer float's number, list line and
/// label: each float sets the enclosing float's pending caption state aside while it is open (a `[H]`
/// table in a minipage in a table, a figure in a figure, a `\captionof` in a table, a figure in a table,
/// an inner table captioned first). Perl gave the outer caption's state to the inner float and left the
/// outer untagged (KNOWN_PERL_ERRORS #395). Repro captions-floats/nested_float_keeps_the_outer_caption.
#[test]
fn nested_float_keeps_the_outer_caption() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/nested_float_keeps_the_outer_caption.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "figure",
      "S0.F1",
      r##"<figure inlist="lof" labels="LABEL:f:o" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>Outer figure</toccaption><caption><tag close=": ">Figure 1</tag>Outer figure</caption><figure class="ltx_minipage" inlist="lof" labels="LABEL:f:i" placement="H" vattach="middle" width="155.3pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><toccaption><tag close=" ">2</tag>Inner figure</toccaption><caption><tag close=": ">Figure 2</tag>Inner figure</caption><p>x</p></figure></figure>"##,
    ),
    (
      "table",
      "S0.T1",
      r##"<table inlist="lot" labels="LABEL:t:o" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Outer table</toccaption><caption><tag close=": ">Table 1</tag>Outer table</caption><table class="ltx_minipage" inlist="lot" labels="LABEL:t:c" vattach="middle" width="155.3pt" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>Captionof inner</toccaption><caption><tag close=": ">Table 2</tag>Captionof inner</caption></table></table>"##,
    ),
    (
      "table",
      "S0.T3",
      r##"<table inlist="lot" labels="LABEL:t:o2" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>Outer two</toccaption><caption><tag close=": ">Table 3</tag>Outer two</caption><figure class="ltx_minipage" inlist="lof" labels="LABEL:f:t" placement="H" vattach="middle" width="155.3pt" xml:id="S0.F3"><tags><tag>Figure 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><toccaption><tag close=" ">3</tag>Figure in table</toccaption><caption><tag close=": ">Figure 3</tag>Figure in table</caption><p>y</p></figure></table>"##,
    ),
    (
      "table",
      "S0.T5",
      r##"<table inlist="lot" labels="LABEL:t:o3" xml:id="S0.T5"><tags><tag>Table 5</tag><tag role="refnum">5</tag><tag role="typerefnum">Table 5</tag></tags><table class="ltx_minipage" inlist="lot" labels="LABEL:t:i3" placement="H" vattach="middle" width="155.3pt" xml:id="S0.T4"><tags><tag>Table 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Table 4</tag></tags><toccaption><tag close=" ">4</tag>Inner first</toccaption><caption><tag close=": ">Table 4</tag>Inner first</caption><p>z</p></table><toccaption><tag close=" ">5</tag>Outer after</toccaption><caption><tag close=": ">Table 5</tag>Outer after</caption></table>"##,
    ),
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Refs <ref labelref="LABEL:f:o"/> <ref labelref="LABEL:f:i"/> <ref labelref="LABEL:t:o"/> <ref labelref="LABEL:t:c"/> <ref labelref="LABEL:t:o2"/> <ref labelref="LABEL:f:t"/> <ref labelref="LABEL:t:i3"/> <ref labelref="LABEL:t:o3"/>.</p></para>"##,
    ),
  ]);
}

/// 57cp (review 3): a long or tall table's List of Tables line follows `entry` (tabularray.sty:6459-6478):
/// none for `entry=none`, the short form for `entry={…}`, a line for a `label=none` table (the counter as
/// it stands) and for a caption the theme hides; `\ref`s read the numbers. A formatting-only caption
/// template with an empty `caption-lot` lists nothing, and a redeclared `empty` head prints its caption
/// (review 5, tmpl). Repro alignment-bindings/tabularray_list_of_tables_entries.
#[test]
fn tabularray_list_of_tables_entries() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_list_of_tables_entries.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "table",
      "S0.T1",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:n" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Normal</toccaption><caption><tag close=": ">Table 1</tag>Normal</caption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T2",
      r##"<table class="ltx_tblr_box" labels="LABEL:t:e" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>No entry</toccaption><caption><tag close=": ">Table 2</tag>No entry</caption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T3",
      r##"<table class="ltx_tblr_box" inlist="lot" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>Short</toccaption><caption><tag close=": ">Table 3</tag>Short entry</caption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "tab1",
      r##"<table class="ltx_tblr_box" inlist="lot" xml:id="tab1"><toccaption><tag close=" ">3</tag>No label</toccaption><caption>No label</caption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T4",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:h" xml:id="S0.T4"><tags><tag>Table 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Table 4</tag></tags><toccaption><tag close=" ">4</tag>Hidden</toccaption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T5",
      r##"<table class="ltx_tblr_box" inlist="lot" xml:id="S0.T5"><tags><tag>Table 5</tag><tag role="refnum">5</tag><tag role="typerefnum">Table 5</tag></tags><toccaption><tag close=" ">5</tag>Hidden unlabelled</toccaption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T7",
      r##"<table class="ltx_tblr_box" labels="LABEL:t:d" xml:id="S0.T7"><tags><tag>Table 7</tag><tag role="refnum">7</tag><tag role="typerefnum">Table 7</tag></tags><toccaption><tag close=" ">7</tag>In table no entry</toccaption><caption><tag close=": ">Table 7</tag>In table no entry</caption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T8",
      r##"<table class="ltx_tblr_box" labels="LABEL:t:f" xml:id="S0.T8"><tags><tag>Table 8</tag><tag role="refnum">8</tag><tag role="typerefnum">Table 8</tag></tags><toccaption><tag close=" ">8</tag>Fancy</toccaption><caption><tag close=": ">Table 8</tag>Fancy</caption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T9",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:r" xml:id="S0.T9"><tags><tag>Table 9</tag><tag role="refnum">9</tag><tag role="typerefnum">Table 9</tag></tags><toccaption><tag close=" ">9</tag>Redeclared</toccaption><caption><tag close=": ">Table 9</tag>Redeclared</caption><tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular></table>"##,
    ),
    (
      "para",
      "p10",
      r##"<para xml:id="p10"><p>Refs <ref labelref="LABEL:t:n"/> <ref labelref="LABEL:t:e"/> <ref labelref="LABEL:t:h"/> <ref labelref="LABEL:t:d"/> <ref labelref="LABEL:t:f"/> <ref labelref="LABEL:t:r"/>.</p></para>"##,
    ),
  ]);
}

/// 57cp (review 3): the box holding a long or tall table is as wide as the table, whose caption and
/// notes tabularray sets at that width (tabularray.sty:6480-6508): `\resizebox{\linewidth}` scales it by
/// about 9, as pdflatex does, not by 1 (a note) or 3 (the caption line). Repro
/// alignment-bindings/tabularray_box_is_as_wide_as_its_table.
#[test]
fn tabularray_box_is_as_wide_as_its_table() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_box_is_as_wide_as_its_table.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Before.<inline-block depth="380.0pt" height="55.0pt" width="345.0pt" xscale="7.91585981911444" xtranslate="150.7pt" yscale="7.91585981911444" ytranslate="-190.0pt"><inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Resized</toccaption><caption><tag close=": ">Table 1</tag>Resized</caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">A</td><td align="left">B<sup>a</sup></td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">a</text></sup> Resized note.</p></table></inline-logical-block></inline-block>After one.</p></para>"##,
    ),
    (
      "table",
      "tab1",
      r##"<table xml:id="tab1"><inline-block depth="244.1pt" height="61.7pt" width="345.0pt" xscale="8.87776557511515" xtranslate="153.1pt" yscale="8.87776557511515" ytranslate="-135.7pt"><inline-logical-block class="ltx_tblr_box"><table inlist="lot" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>Resized in float</toccaption><caption><tag close=": ">Table 2</tag>Resized in float</caption><tabular vattach="middle"><tbody><tr><td align="left">C</td><td align="left">D</td></tr></tbody></tabular></table></inline-logical-block></inline-block></table>"##,
    ),
  ]);
}

/// 57cp (review 2, review 3): `\SetTblrTemplate` with a name the element has no template of, or an
/// element tabularray does not have, is its error and changes nothing (tabularray.sty:5703-5718);
/// `\SetTblrInner[]{…}` with an explicitly empty list applies to no environment (`O{tblr}`). Repro
/// alignment-bindings/tabularray_template_names_and_environment_lists.
#[test]
fn tabularray_template_names_and_environment_lists() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_template_names_and_environment_lists.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (4, 0),
    &[
      "Undefined template \"nosuch\" for element \"caption\".",
      "Undefined template \"simple\" for element \"caption-tag\".",
      "Undefined template \"simple\" for element \"note-tag\".",
      "Undefined template \"normal\" for element \"nosuchelement\".",
    ],
    &[
      (
        "table",
        "S0.T1",
        r##"<table class="ltx_tblr_box" inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag close=": ">Table 1</tag>Cap</caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">a</text></sup> Note text</p></table>"##,
      ),
      (
        "para",
        "p2",
        r##"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left">c</td></tr></tbody></tabular></para>"##,
      ),
    ],
  );
}

/// 57cp (review 4): a tall table in a `table` float is a `table` of its own, beside the float's own
/// caption before or after it (both numbers kept, as pdflatex), and the float itself when the float
/// has no caption (`collapse_float`, the table's id); the document float taking the table's caption
/// put two captions in one float (KNOWN_PERL_ERRORS #396). Repro
/// alignment-bindings/tabularray_tall_table_in_a_captioned_float.
#[test]
fn tabularray_tall_table_in_a_captioned_float() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_tall_table_in_a_captioned_float.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "table",
      "S0.T2",
      r##"<table inlist="lot" labels="LABEL:t:o" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><table align="center" class="ltx_tblr_box" inlist="lot" labels="LABEL:t:i" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Inner tall</toccaption><caption><tag close=": ">Table 1</tag>Inner tall</caption><tabular vattach="middle"><tbody><tr><td align="left">A</td><td align="left">B</td></tr></tbody></tabular></table><toccaption class="ltx_centering"><tag close=" ">2</tag>Outer after</toccaption><caption class="ltx_centering"><tag close=": ">Table 2</tag>Outer after</caption></table>"##,
    ),
    (
      "table",
      "tab1",
      r##"<table xml:id="tab1"><table align="center" class="ltx_figure_panel ltx_tblr_box" inlist="lot" labels="LABEL:t:i2" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>Inner two</toccaption><caption><tag close=": ">Table 3</tag>Inner two</caption><tabular vattach="middle"><tbody><tr><td align="left">C</td><td align="left">D</td></tr></tbody></tabular></table><table align="center" class="ltx_figure_panel ltx_tblr_box" inlist="lot" labels="LABEL:t:i3" xml:id="S0.T4"><tags><tag>Table 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Table 4</tag></tags><toccaption><tag close=" ">4</tag>Inner three</toccaption><caption><tag close=": ">Table 4</tag>Inner three</caption><tabular vattach="middle"><tbody><tr><td align="left">E</td><td align="left">F</td></tr></tbody></tabular></table></table>"##,
    ),
    (
      "table",
      "S0.T6",
      r##"<table inlist="lot" labels="LABEL:t:o4" xml:id="S0.T6"><tags><tag>Table 6</tag><tag role="refnum">6</tag><tag role="typerefnum">Table 6</tag></tags><table align="center" class="ltx_tblr_box" inlist="lot" labels="LABEL:t:i4" xml:id="S0.T5"><tags><tag>Table 5</tag><tag role="refnum">5</tag><tag role="typerefnum">Table 5</tag></tags><toccaption><tag close=" ">5</tag></toccaption><caption><tag close=": ">Table 5</tag></caption><tabular vattach="middle"><tbody><tr><td align="left">G</td><td align="left">H</td></tr></tbody></tabular></table><toccaption class="ltx_centering"><tag close=" ">6</tag>Outer four</toccaption><caption class="ltx_centering"><tag close=": ">Table 6</tag>Outer four</caption></table>"##,
    ),
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Refs A <ref labelref="LABEL:t:i"/> <ref labelref="LABEL:t:o"/> <ref labelref="LABEL:t:i2"/> <ref labelref="LABEL:t:i3"/> <ref labelref="LABEL:t:i4"/> <ref labelref="LABEL:t:o4"/>.</p></para>"##,
    ),
    (
      "table",
      "S0.T7",
      r##"<table align="center" class="ltx_tblr_box" inlist="lot" labels="LABEL:t:oB" xml:id="S0.T7"><tags><tag>Table 7</tag><tag role="refnum">7</tag><tag role="typerefnum">Table 7</tag></tags><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">A<sup>a</sup></td><td align="left">B</td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">a</text></sup> First note.</p><toccaption class="ltx_centering"><tag close=" ">7</tag>Outer after</toccaption><caption class="ltx_centering"><tag close=": ">Table 7</tag>Outer after</caption></table>"##,
    ),
    (
      "table",
      "S0.T8",
      r##"<table align="center" class="ltx_tblr_box" inlist="lot" labels="LABEL:t:bB" xml:id="S0.T8"><tags><tag>Table 8</tag><tag role="refnum">8</tag><tag role="typerefnum">Table 8</tag></tags><toccaption class="ltx_centering"><tag close=" ">8</tag>Outer before</toccaption><caption class="ltx_centering"><tag close=": ">Table 8</tag>Outer before</caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">C<sup>a</sup></td><td align="left">D</td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">a</text></sup> Second note.</p></table>"##,
    ),
    (
      "table",
      "S0.T10",
      r##"<table inlist="lot" labels="LABEL:t:3B" xml:id="S0.T10"><tags><tag>Table 10</tag><tag role="refnum">10</tag><tag role="typerefnum">Table 10</tag></tags><table align="center" class="ltx_tblr_box" inlist="lot" xml:id="S0.T9"><tags><tag>Table 9</tag><tag role="refnum">9</tag><tag role="typerefnum">Table 9</tag></tags><toccaption><tag close=" ">9</tag></toccaption><caption><tag close=": ">Table 9</tag></caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="left">E<sup>a</sup></td><td align="left">F</td></tr></tbody></tabular><break class="ltx_break"/><p class="ltx_figure_panel"><sup><text font="sansserif">a</text></sup> Third note.</p></table><toccaption class="ltx_centering"><tag close=" ">10</tag>Outer three</toccaption><caption class="ltx_centering"><tag close=": ">Table 10</tag>Outer three</caption></table>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>Refs B <ref labelref="LABEL:t:oB"/> <ref labelref="LABEL:t:bB"/> <ref labelref="LABEL:t:3B"/>.</p></para>"##,
    ),
  ]);
}

/// 57cp (review 4): a numbered table whose caption the document's own template prints (tblr-extras'
/// `caption` library, which empties `caption-lot` and writes the line through `\caption[entry]`) is
/// listed; a `label=none` one is not (review 5, tex5). Repro
/// alignment-bindings/tabularray_caption_library_lists_its_tables.
#[test]
fn tabularray_caption_library_lists_its_tables() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_caption_library_lists_its_tables.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "table",
      "S0.T1",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:tab:a" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>My Long Title</toccaption><caption><tag close=": ">Table 1</tag>My Long Title</caption><tabular vattach="middle"><tbody><tr><td align="left">A</td><td align="left">B</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T2",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:tab:b" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>Short tall</toccaption><caption><tag close=": ">Table 2</tag>My Tall Title</caption><tabular vattach="middle"><tbody><tr><td align="left">C</td><td align="left">D</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "tab1",
      r##"<table class="ltx_tblr_box" xml:id="tab1"><caption>Label none</caption><tabular vattach="middle"><tbody><tr><td align="left">E</td><td align="left">F</td></tr></tbody></tabular></table>"##,
    ),
    (
      "para",
      "p4",
      r##"<para xml:id="p4"><p>Refs <ref labelref="LABEL:tab:a"/> <ref labelref="LABEL:tab:b"/>.</p></para>"##,
    ),
  ]);
}

/// 57cp (review 5): a tall table in a caption-less `table` is that table — numbered by its tags alone
/// too (a theme hiding the head: `S0.T1`, not `tab1`) — and beside a tabular it stays one panel of
/// two. Repro alignment-bindings/tabularray_tall_table_in_a_captionless_float.
#[test]
fn tabularray_tall_table_in_a_captionless_float() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_tall_table_in_a_captionless_float.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "table",
      "S0.T1",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:x" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Hidden</toccaption><tabular vattach="middle"><tbody><tr><td align="left">A</td><td align="left">B</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T2",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:s" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>Tall with tabular</toccaption><caption><tag close=": ">Table 2</tag>Tall with tabular</caption><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="center">a</td><td align="center">b</td></tr></tbody></tabular><tabular class="ltx_figure_panel" vattach="middle"><tbody><tr><td align="center">x</td><td align="center">y</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T3",
      r##"<table inlist="lot" labels="LABEL:t:y" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>Next</toccaption><caption><tag close=": ">Table 3</tag>Next</caption><tabular vattach="middle"><tbody><tr><td align="left">C</td></tr></tbody></tabular></table>"##,
    ),
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Refs <ref labelref="LABEL:t:x"/> <ref labelref="LABEL:t:s"/> <ref labelref="LABEL:t:y"/>.</p></para>"##,
    ),
  ]);
}

/// 57cp (review 5): a caption-less float collapsing into the one float in it keeps the id of the
/// element carrying its number — a caption-less longtable's own step (`S1.T1`, Perl `S1.tab1`), the
/// float's own tags from its minipage's caption (`S1.F1`, Perl `S1.F1.fig1`) — and the captioned
/// minipage's content stays a panel beside the other minipage, its caption after both (review 6: a
/// captioned first minipage leaves the caption between the panels). Repro
/// captions-floats/captionless_float_takes_the_inner_number.
#[test]
fn captionless_float_takes_the_inner_number() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/captionless_float_takes_the_inner_number.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "table",
      "S1.T1",
      r##"<table inlist="lot" placement="h" xml:id="S1.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><tabular><tr><td align="center">a</td><td align="center">b</td></tr></tabular></table>"##,
    ),
    (
      "table",
      "S1.T2",
      r##"<table inlist="lot" labels="LABEL:nx" placement="h" xml:id="S1.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>Next</toccaption><caption><tag close=": ">Table 2</tag>Next</caption><tabular vattach="middle"><tbody><tr><td align="center">c</td></tr></tbody></tabular></table>"##,
    ),
    (
      "figure",
      "S1.F1",
      r##"<figure inlist="lof" labels="LABEL:lf" placement="h" xml:id="S1.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p align="center" class="ltx_figure_panel ltx_minipage" vattach="middle" width="155.3pt"><text cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle">L</text></p><p align="center" class="ltx_figure_panel"><text cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle">R</text></p><toccaption class="ltx_centering"><tag close=" ">1</tag>Right</toccaption><caption class="ltx_centering"><tag close=": ">Figure 1</tag>Right</caption></figure>"##,
    ),
    (
      "para",
      "S1.p1",
      r##"<para xml:id="S1.p1"><p>Refs <ref labelref="LABEL:nx"/> <ref labelref="LABEL:lf"/>.</p></para>"##,
    ),
  ]);
}

/// 57cp (review 6): a document template calling `\caption` — through a macro it calls too, a `\let`
/// copy, `\csname` or `\captionof{table}` (review 7), a head's `\UseTblrTemplate{caption}` (review 8),
/// a macro met deep in a chain first, a `\captionof` whose `{table}` follows its macro (review 9) —
/// writes the List of Tables line; `\caption*` none (also after a macro ending in `\caption`), nor
/// `\captionof{figure}` or `\csname caption \endcsname`; a `caption` template under an empty head
/// writes nothing. Repro alignment-bindings/tabularray_template_calling_caption_is_listed.
#[test]
fn tabularray_template_calling_caption_is_listed() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_template_calling_caption_is_listed.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "table",
      "S0.T1",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:a" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Nested macro</toccaption><caption><tag close=": ">Table 1</tag>Nested macro</caption><tabular vattach="middle"><tbody><tr><td align="left">A</td><td align="left">B</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T2",
      r##"<table class="ltx_tblr_box" labels="LABEL:t:s" xml:id="S0.T2"><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><toccaption><tag close=" ">2</tag>Star one</toccaption><caption><tag close=": ">Table 2</tag>Star one</caption><tabular vattach="middle"><tbody><tr><td align="left">C</td><td align="left">D</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T3",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:p" xml:id="S0.T3"><tags><tag>Table 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Table 3</tag></tags><toccaption><tag close=" ">3</tag>Plain</toccaption><caption><tag close=": ">Table 3</tag>Plain</caption><tabular vattach="middle"><tbody><tr><td align="left">E</td><td align="left">F</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T4",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:l" xml:id="S0.T4"><tags><tag>Table 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Table 4</tag></tags><toccaption><tag close=" ">4</tag>Letcap</toccaption><caption><tag close=": ">Table 4</tag>Letcap</caption><tabular vattach="middle"><tbody><tr><td align="left">G</td><td align="left">H</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T5",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:c" xml:id="S0.T5"><tags><tag>Table 5</tag><tag role="refnum">5</tag><tag role="typerefnum">Table 5</tag></tags><toccaption><tag close=" ">5</tag>Csname</toccaption><caption><tag close=": ">Table 5</tag>Csname</caption><tabular vattach="middle"><tbody><tr><td align="left">I</td><td align="left">J</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T6",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:o" xml:id="S0.T6"><tags><tag>Table 6</tag><tag role="refnum">6</tag><tag role="typerefnum">Table 6</tag></tags><toccaption><tag close=" ">6</tag>Captionof</toccaption><caption><tag close=": ">Table 6</tag>Captionof</caption><tabular vattach="middle"><tbody><tr><td align="left">K</td><td align="left">L</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T7",
      r##"<table class="ltx_tblr_box" labels="LABEL:t:t" xml:id="S0.T7"><tags><tag>Table 7</tag><tag role="refnum">7</tag><tag role="typerefnum">Table 7</tag></tags><toccaption><tag close=" ">7</tag>Trailstar</toccaption><caption><tag close=": ">Table 7</tag>Trailstar</caption><tabular vattach="middle"><tbody><tr><td align="left">M</td><td align="left">N</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T8",
      r##"<table class="ltx_tblr_box" labels="LABEL:t:h" xml:id="S0.T8"><tags><tag>Table 8</tag><tag role="refnum">8</tag><tag role="typerefnum">Table 8</tag></tags><tabular vattach="middle"><tbody><tr><td align="left">O</td><td align="left">P</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T9",
      r##"<table class="ltx_tblr_box" labels="LABEL:t:f" xml:id="S0.T9"><tags><tag>Table 9</tag><tag role="refnum">9</tag><tag role="typerefnum">Table 9</tag></tags><toccaption><tag close=" ">9</tag>Capfig</toccaption><caption><tag close=": ">Table 9</tag>Capfig</caption><tabular vattach="middle"><tbody><tr><td align="left">Q</td><td align="left">R</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T10",
      r##"<table class="ltx_tblr_box" labels="LABEL:t:x" xml:id="S0.T10"><tags><tag>Table 10</tag><tag role="refnum">10</tag><tag role="typerefnum">Table 10</tag></tags><toccaption><tag close=" ">10</tag>Spacecs</toccaption><caption><tag close=": ">Table 10</tag>Spacecs</caption><tabular vattach="middle"><tbody><tr><td align="left">S</td><td align="left">T</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T11",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:u" xml:id="S0.T11"><tags><tag>Table 11</tag><tag role="refnum">11</tag><tag role="typerefnum">Table 11</tag></tags><toccaption><tag close=" ">11</tag>Usecap</toccaption><caption><tag close=": ">Table 11</tag>Usecap</caption><tabular vattach="middle"><tbody><tr><td align="left">U</td><td align="left">V</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T12",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:d" xml:id="S0.T12"><tags><tag>Table 12</tag><tag role="refnum">12</tag><tag role="typerefnum">Table 12</tag></tags><toccaption><tag close=" ">12</tag>Deepfirst</toccaption><caption><tag close=": ">Table 12</tag>Deepfirst</caption><tabular vattach="middle"><tbody><tr><td align="left">W</td><td align="left">X</td></tr></tbody></tabular></table>"##,
    ),
    (
      "table",
      "S0.T13",
      r##"<table class="ltx_tblr_box" inlist="lot" labels="LABEL:t:m" xml:id="S0.T13"><tags><tag>Table 13</tag><tag role="refnum">13</tag><tag role="typerefnum">Table 13</tag></tags><toccaption><tag close=" ">13</tag>Cofmacro</toccaption><caption><tag close=": ">Table 13</tag>Cofmacro</caption><tabular vattach="middle"><tbody><tr><td align="left">Y</td><td align="left">Z</td></tr></tbody></tabular></table>"##,
    ),
    (
      "para",
      "p14",
      r##"<para xml:id="p14"><p>Refs <ref labelref="LABEL:t:a"/> <ref labelref="LABEL:t:s"/> <ref labelref="LABEL:t:p"/> <ref labelref="LABEL:t:l"/> <ref labelref="LABEL:t:c"/> <ref labelref="LABEL:t:o"/> <ref labelref="LABEL:t:t"/> <ref labelref="LABEL:t:h"/> <ref labelref="LABEL:t:f"/> <ref labelref="LABEL:t:x"/> <ref labelref="LABEL:t:u"/> <ref labelref="LABEL:t:d"/> <ref labelref="LABEL:t:m"/>.</p></para>"##,
    ),
  ]);
}

/// 57cp (review 6): a `sidewaysfigure` (and `sidewaysfigure*`, review 7) is listed as any figure is
/// (Perl's rotating binding gives it no `inlist`, KNOWN_PERL_ERRORS #397). Repro
/// captions-floats/sidewaysfigure_is_listed.
#[test]
fn sidewaysfigure_is_listed() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/sidewaysfigure_is_listed.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "figure",
      "S0.F1",
      r##"<figure angle="90" depth="0.0pt" height="550.0pt" inlist="lof" innerdepth="1.9pt" innerheight="6.9pt" innerwidth="550.0pt" labels="LABEL:F1" width="8.9pt" xtranslate="-270.6pt" ytranslate="-270.6pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p>F-body</p><toccaption><tag close=" ">1</tag>Direct cap</toccaption><caption><tag close=": ">Figure 1</tag>Direct cap</caption></figure>"##,
    ),
    (
      "figure",
      "S0.F2",
      r##"<figure angle="90" depth="0.0pt" height="550.0pt" inlist="lof" innerdepth="1.9pt" innerheight="6.9pt" innerwidth="550.0pt" labels="LABEL:F4" width="8.9pt" xtranslate="-270.6pt" ytranslate="-270.6pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><p>F-body</p><toccaption><tag close=" ">2</tag>Star cap</toccaption><caption><tag close=": ">Figure 2</tag>Star cap</caption></figure>"##,
    ),
    (
      "figure",
      "S0.F3",
      r##"<figure angle="90" class="ltx_minipage" depth="0.0pt" height="172.5pt" inlist="lof" innerdepth="1.9pt" innerheight="6.9pt" innerwidth="172.5pt" labels="LABEL:F2" vattach="middle" width="172.5pt" xtranslate="-81.8pt" ytranslate="-81.8pt" xml:id="S0.F3"><tags><tag>Figure 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><p>F-body</p><toccaption><tag close=" ">3</tag>Minipage cap</toccaption><caption><tag close=": ">Figure 3</tag>Minipage cap</caption></figure>"##,
    ),
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Refs <ref labelref="LABEL:F1"/> <ref labelref="LABEL:F4"/> <ref labelref="LABEL:F2"/> <ref labelref="LABEL:F3"/>.</p></para>"##,
    ),
  ]);
}
