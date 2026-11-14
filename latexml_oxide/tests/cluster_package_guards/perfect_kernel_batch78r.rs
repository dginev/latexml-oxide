//! Red/green guards for batch 78r (quality batch 15): AASTeX 6+'s own journal abbreviations, an eqnarray `\label` with
//! no numbered row after it, `\nonumber` in `{equation}` without amsmath, algorithm2e's `\Indp`/`\Indm`. Each guard
//! converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and whole elements,
//! against pdflatex.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// Every `<equation>` in document order as `labels@xml:id:<tags>…</tags>`, `-` for one without tags.
pub(super) fn equation_heads(xml: &str) -> String {
  let re = regex::Regex::new(r#"(?s)<equation ([^>]*)>\s*(<tags>.*?</tags>)?"#).unwrap();
  let labels = regex::Regex::new(r#"labels="([^"]*)""#).unwrap();
  let id = regex::Regex::new(r#"xml:id="([^"]*)""#).unwrap();
  let space = regex::Regex::new(r"\s*\n\s*").unwrap();
  re.captures_iter(xml)
    .map(|c| {
      let label = labels
        .captures(&c[1])
        .map_or("", |l| l.get(1).unwrap().as_str());
      let id = id
        .captures(&c[1])
        .map_or("", |i| i.get(1).unwrap().as_str());
      let tags = c.get(2).map_or("-".to_string(), |t| {
        space.replace_all(t.as_str(), "").to_string()
      });
      format!("{label}@{id}:{tags}")
    })
    .collect::<Vec<_>>()
    .join("|")
}

/// A numbered equation's tags: the printed `(n)` and the refnum.
pub(super) fn tags(n: u32) -> String {
  format!("<tags><tag>({n})</tag><tag role=\"refnum\">{n}</tag></tags>")
}

/// The reference forms alone of the number `n`, which no row prints.
pub(super) fn reference_tags(n: u32) -> String {
  format!("<tags><tag role=\"refnum\">{n}</tag></tags>")
}

/// aastex6.cls:1303, aastex631.cls:1838-1900, aastex701.cls:8310-8373: AASTeX 6+'s journal abbreviations, which differ
/// from aas_macros.sty's ADS forms in 28 names (pdflatex "J: ApJL, PhRvD, SSRv, A&A Rv, JCAP, PhR, ApJ, ApJL, ZA.").
/// Red (int54a): "J: ApJ, Phys. Rev. D, Space Sci. Rev., …" (KNOWN_PERL_ERRORS #1105). Witness 1701.04641. Repro
/// class-bindings/aastex6_journal_abbreviations_are_the_class_s.
#[test]
fn aastex6_journal_abbreviations_are_the_class_s() {
  let xml = convert_clean(repro!(
    "class-bindings/aastex6_journal_abbreviations_are_the_class_s.tex"
  ));
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">J: ApJL, PhRvD, SSRv, A&amp;A\u{a0}Rv, JCAP, PhR, ApJ, ApJL, ZA.</p>",
  );
}

/// No AASTeX 6+ class loads aas_macros.sty: a document's own `\usepackage{aas_macros}` after the class loads the file,
/// whose `\def`s put the ADS names back (pdflatex "J: ApJ, Phys. Rev. D, A&A Rev., ApJ." with 2002.05892's copy).
/// Witnesses 1803.01278, 2002.05892, 2208.01612. Repro stand-in-copies/aas_macros_after_aastex.
#[test]
fn aas_macros_after_aastex6_restores_the_ads_names() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/aas_macros_after_aastex/aas_macros_restores_the_ads_names.tex"),
    &[(
      "aas_macros.sty",
      repro!("stand-in-copies/aas_macros_after_aastex/aas_macros.sty"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">J: ApJ, Phys.\u{a0}Rev.\u{a0}D, A&amp;A\u{a0}Rev., ApJ.</p>",
  );
}

/// latex.ltx:15750-15752, :15768-15771, :15795: eqnarray steps the counter at its start and at each printing `\@@eqncr`,
/// and back at its end, so a `\label` with no numbered row after it names the counter's next value — the next
/// equation's number (pdflatex "See 2, 2, 4 and 4."). Its row keeps that number's reference forms, no printed tag, and
/// stands as its own equation (a continuation row too). Red (int54a): x, y and w on rows with no number (their `\ref`
/// a raw id), z joined into the equation numbered (3). Witnesses 1602.03553, 2301.00760, 1303.3431. Repro cross-references/eqnarray_label_with_no_numbered_row_after_it.
#[test]
fn eqnarray_label_with_no_numbered_row_after_it_names_the_next_number() {
  let xml = convert_clean(repro!(
    "cross-references/eqnarray_label_with_no_numbered_row_after_it.tex"
  ));
  assert_eq!(
    equation_heads(&xml),
    format!(
      "@S0.E1:{}|LABEL:x@S0.Ex1:{}|LABEL:y@S0.Ex2:{}|@S0.E2:{}|@S0.E3:{}|LABEL:z@S0.Ex3:{}|LABEL:w@S0.Ex4:{}",
      tags(1),
      reference_tags(2),
      reference_tags(2),
      tags(2),
      tags(3),
      reference_tags(4),
      reference_tags(4)
    ),
    "{xml}"
  );
}

/// latex.ltx:15737-15739: `{equation}` prints its number whatever a `\nonumber` in it said (pdflatex (1) (2) (3), "See
/// 1, 2, 3."). Red (int54a): the second equation unnumbered, the third (2) (OXIDIZED_DESIGN_DIVERGENCES #865).
/// Witnesses hep-ph9710521, nucl-th0404042. Repro cross-references/equation_nonumber_without_amsmath_keeps_the_number.
#[test]
fn equation_nonumber_without_amsmath_keeps_the_number() {
  let xml = convert_clean(repro!(
    "cross-references/equation_nonumber_without_amsmath_keeps_the_number.tex"
  ));
  assert_eq!(
    equation_heads(&xml),
    format!(
      "LABEL:e1@S0.E1:{}|LABEL:e2@S0.E2:{}|LABEL:e3@S0.E3:{}",
      tags(1),
      tags(2),
      tags(3)
    ),
    "{xml}"
  );
}

/// amsmath.sty:1268-1269: amsmath's `\nonumber` (and `\notag`) removes an `{equation}`'s number (pdflatex (1), two
/// unnumbered, (2)). Repro cross-references/equation_nonumber_with_amsmath_removes_the_number.
#[test]
fn equation_nonumber_with_amsmath_removes_the_number() {
  let xml = convert_clean(repro!(
    "cross-references/equation_nonumber_with_amsmath_removes_the_number.tex"
  ));
  assert_eq!(
    equation_heads(&xml),
    format!(
      "LABEL:e1@S0.E1:{}|@S0.Ex1:-|@S0.Ex2:-|LABEL:e3@S0.E2:{}",
      tags(1),
      tags(2)
    ),
    "{xml}"
  );
}

/// algorithm2e.sty:1626-1629: `\Indp` and `\Indm` move the lines after them a level in and back (pdflatex "indented;"
/// one level in, "deeper;" two, "back;" at the margin). Red (int54a): every line at the margin (KNOWN_PERL_ERRORS
/// #1107). Repro listings-algorithms/algorithm2e_indp_indm_indent_the_lines_between.
#[test]
fn algorithm2e_indp_indm_indent_the_lines_between() {
  let xml = convert_clean(repro!(
    "listings-algorithms/algorithm2e_indp_indm_indent_the_lines_between.tex"
  ));
  assert_element(
    &xml,
    "listing",
    &["xml:id=\"algorithm1.3\""],
    "<listing class=\"ltx_lst_numbers_left\" framed=\"topbottom\" xml:id=\"algorithm1.3\">\
     <listingline xml:id=\"algorithm1.3.1\">first;</listingline>\
     <listingline xml:id=\"algorithm1.3.2\">\u{2002}\u{2003}indented;</listingline>\
     <listingline xml:id=\"algorithm1.3.3\">\u{2002}\u{2003}\u{2002}\u{2003}deeper;</listingline>\
     <listingline xml:id=\"algorithm1.3.4\">back;</listingline></listing>",
  );
}

/// algorithm2e.sty:1626-1629: `\Indm` is `\advance\leftskip by -\algoskipindent`, a signed offset, not a block's level:
/// in the flush-left Input idiom `\Indm\Indmm \KwIn{…} \KwOut{…} \Indp\Indpp` the body stays at the margin (pdflatex
/// Input/Output at x=128.4, the body at 148.7, the loop's body a level in). Red (78r r1): every body line a level in —
/// the `\Indm` popped nothing, the `\Indp` pushed a level (2102.06989's 13 lines; KNOWN_PERL_ERRORS #1107). Witnesses
/// 2102.06989, 1412.1462, 2106.03753, 2508.14614. Repro
/// listings-algorithms/algorithm2e_indm_before_the_input_keeps_the_body_at_the_margin.
#[test]
fn algorithm2e_indm_before_the_input_keeps_the_body_at_the_margin() {
  let xml = convert_clean(repro!(
    "listings-algorithms/algorithm2e_indm_before_the_input_keeps_the_body_at_the_margin.tex"
  ));
  assert_eq!(xml.matches("<listingline ").count(), 6, "{xml}");
  for (id, line) in [
    (
      "tab1.1.1",
      "<listingline xml:id=\"tab1.1.1\"><text xml:id=\"tab1.1.1.1\"><text font=\"bold\" \
       xml:id=\"tab1.1.1.1.1\">Input:</text> </text>Xin</listingline>",
    ),
    (
      "tab1.1.2",
      "<listingline xml:id=\"tab1.1.2\"><text xml:id=\"tab1.1.2.1\"><text font=\"bold\" \
       xml:id=\"tab1.1.2.1.1\">Output:</text> </text>Yout</listingline>",
    ),
    (
      "tab1.1.3",
      "<listingline xml:id=\"tab1.1.3\">Zbody;</listingline>",
    ),
    (
      "tab1.1.5",
      "<listingline xml:id=\"tab1.1.5\">\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}Wloop;</listingline>",
    ),
    (
      "tab1.1.6",
      "<listingline xml:id=\"tab1.1.6\">Vend;</listingline>",
    ),
  ] {
    assert_element(&xml, "listingline", &[&format!("xml:id=\"{id}\"")], line);
  }
}

/// algorithm2e.sty:1826-1860: a block's body is set in `\hbox{\vtop{…}}`, whose group ends what an `\Indp`/`\Indm` in
/// it did; at the algorithm's level they last to its end (pdflatex levels 0 0 1 1 1 1 0 1 2 0: "Cc;" one level in after
/// the inner `\Indm`, "Dd;" one level in after the inner block, "Ff;" two after `\Indp` and a block). Red (int54a): "Cc;"
/// two levels in, "Ff;" one; (78r r1): "Dd;" at the margin, the `\Indm` popped the inner block's level and the block's
/// end the outer's (KNOWN_PERL_ERRORS #1107). Witness 1903.06418. Repro
/// listings-algorithms/algorithm2e_indm_in_a_block_ends_with_the_block.
#[test]
fn algorithm2e_indm_in_a_block_ends_with_the_block() {
  let xml = convert_clean(repro!(
    "listings-algorithms/algorithm2e_indm_in_a_block_ends_with_the_block.tex"
  ));
  assert_eq!(xml.matches("<listingline ").count(), 10, "{xml}");
  let level = "\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}";
  for (id, line) in [
    ("tab1.1.1", "Aa;".to_string()),
    ("tab1.1.3", format!("{level}Bb;")),
    ("tab1.1.5", format!("{level}Cc;")),
    ("tab1.1.6", format!("{level}Dd;")),
    ("tab1.1.7", "Ee;".to_string()),
    ("tab1.1.9", format!("{level}\u{2002}\u{2003}Ff;")),
    ("tab1.1.10", "Gg;".to_string()),
  ] {
    assert_element(
      &xml,
      "listingline",
      &[&format!("xml:id=\"{id}\"")],
      &format!("<listingline xml:id=\"{id}\">{line}</listingline>"),
    );
  }
}

/// mnras.cls's `{equation}` is LaTeX's (latex.ltx:15737-15739), set by the binding of mn2e/mnras (mn2e_support_sty.rs,
/// as aa_support_sty.rs for the old aa.cls) only to let a `$` in it: a `\nonumber` there leaves the number (pdflatex (1)
/// (2), "See 1; 2."). Red (int54a, 78r r1): the first equation unnumbered (OXIDIZED_DESIGN_DIVERGENCES #865). Repro
/// class-bindings/mnras_equation_nonumber_keeps_the_number.
#[test]
fn mnras_equation_nonumber_keeps_the_number() {
  let xml = convert_clean(repro!(
    "class-bindings/mnras_equation_nonumber_keeps_the_number.tex"
  ));
  let tags = |n: u32| {
    format!(
      "<tags><tag>({n})</tag><tag role=\"autoref\">Equation\u{a0}{n}<text xml:id=\"S0.E{n}.1\"/></tag>\
       <tag role=\"refnum\">{n}</tag></tags>"
    )
  };
  assert_eq!(
    equation_heads(&xml),
    format!("LABEL:m1@S0.E1:{}|LABEL:m2@S0.E2:{}", tags(1), tags(2)),
    "{xml}"
  );
}
