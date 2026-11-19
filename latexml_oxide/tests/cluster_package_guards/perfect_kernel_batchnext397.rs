//! Red/green guards for batch NEXT397, three Error roots of run 397 where TeX (or the class TeX would read) is clean:
//! astron.sty's `\astroncite` labels under the locked `\cite`, the 1998 aipproc copy's `\affiliationmark`, and
//! elsart.sty's option switches. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// astron.sty's `\cite` (0812.2799, 0812.4839; its copies EslabStyle.cls astro-ph0001290, ISOStyle.cls astro-ph0008362)
/// is the only definer of the `\astroncite` its `\bibitem` labels hold, and the lock refuses it: the first kernel
/// `\cite` runs the copy's own `\citefull` definition. Distinct from
/// `perfect_kernel_batchclustfix::starred_cite_under_the_lock` (the star; this is the label the citation prints). Repro
/// index-bib/astroncite.
#[test]
fn astroncite_label_under_the_lock() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/astroncite/astroncite_label_under_the_lock.tex"
    ),
    &[(
      "astron.sty",
      include_str!("../../../tools/perfect_kernel/repros/index-bib/astroncite/astron.sty"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "bibitem",
    &[],
    "<bibitem key=\"c_89\" xml:id=\"bib.bibx1\"><tags><tag role=\"refnum\">Cardelli et al., 1989</tag></tags>\
     <bibblock> Cardelli J., 1989\n</bibblock></bibitem>",
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">As shown by <cite class="ltx_citemacro_cite">[<bibref bibrefs="c_89" separator="," yyseparator=","/>]</cite>.</p>"#,
  );
}

/// astro-ph0011502's myaipproc.cls, a renamed copy of the 1998 aipproc.cls (the binding answers it as a versioned
/// fallback, so the copy itself is not read): its `\affiliationmark` (myaipproc.cls:86) beside `\affiliation`. Distinct
/// from `perfect_kernel_batchclustfix::aipproc_old_author_affiliation`, where the copy declares itself aipproc and is
/// read. Repro stand-in-copies/aipproc_v1/aipproc_affiliationmark.
#[test]
fn aipproc_old_affiliationmark() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/aipproc_v1/aipproc_affiliationmark.tex"
    ),
    &[(
      "myaipproc.cls",
      include_str!("../../../tools/perfect_kernel/repros/stand-in-copies/aipproc_v1/myaipproc.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>A.\u{a0}Author</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"><sup xml:id=\"id1\">a</sup>Dept of Physics</contact></creator>",
  );
}

/// elsart.sty (the LaTeX 2.09 style, v2.23, 0805.0525's copy) defines `\if@seceqn`/`\if@secthm` set by its options
/// (elsart.sty:66-71), as elsart.cls does (`perfect_kernel_batchclustfix::elsart_amsthm_option_is_the_class_switch`);
/// no `\if@amsthm`. Repro class-bindings/elsart_sty_option_switches.
#[test]
fn elsart_sty_options_are_its_switches() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/elsart_sty_option_switches.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">seceqn none</p>"#);
}
