//! Red/green guards for batch NEXT397, four Error roots of run 397, three where TeX (or the class TeX would read) is clean:
//! astron.sty's `\astroncite` labels under the locked `\cite`, the 1998 aipproc copy's `\affiliationmark`,
//! elsart.sty's option switches, and `\verb` in an argument (TeX errs there too). Repros under
//! `tools/perfect_kernel/repros/`.
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

/// `\verb` inside a pre-tokenized argument (cond-mat0203143's `\parbox[t]{…}{\verb+ +\\ \includegraphics…}` in a
/// tabular cell; astro-ph0108286's `\multicolumn`): its frozen delimiter never matches, so TeX ends it at the `}` that
/// closes the argument, "\verb illegal in argument" (latex.ltx:15485-15499), and typesets the rest. The scan ran past
/// that `}` to the next line's end and the rest of the table was lost. The `}` of the verb's own group inside the
/// argument (`{\verb+ + }`) ends it the same way. Repro boxes-groups/verb_in_an_argument_ends_at_its_brace.
#[test]
fn verb_in_an_argument_ends_at_its_brace() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/verb_in_an_argument_ends_at_its_brace.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (2, 0), "{log}");
  assert_eq!(
    log
      .matches("Error:unexpected:\\verb \\verb illegal in argument")
      .count(),
    2,
    "{log}"
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="center" xml:id="p1.1.1.1">1)<inline-block class="ltx_parbox" vattach="top" width="85.4pt" xml:id="p1.1.1.1.1"><p xml:id="p1.1.1.1.1.1"><verbatim font="typewriter" xml:id="p1.1.1.1.1.1.1">+<break/><text font="bold" xml:id="p1.1.1.1.1.1.1.1">second</text> line</verbatim></p></inline-block></td></tr><tr xml:id="p1.1.2"><td align="center" xml:id="p1.1.2.1">2)<inline-block class="ltx_parbox" vattach="top" width="85.4pt" xml:id="p1.1.2.1.1"><p xml:id="p1.1.2.1.1.1"><verbatim font="typewriter" xml:id="p1.1.2.1.1.1.1">+</verbatim></p><p xml:id="p1.1.2.1.1.2"><text font="bold" xml:id="p1.1.2.1.1.2.1">third</text> line</p></inline-block></td></tr></tbody></tabular>"#,
  );
  assert!(xml.contains(r#"<p xml:id="p1.2">Then more.</p>"#), "{xml}");
}
