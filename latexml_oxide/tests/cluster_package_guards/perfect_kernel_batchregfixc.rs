//! Red/green guards for batch REGFIXC: natbib's repeat-load "Option clash" (72a r6) raised where TeX has none, from
//! loads our bindings make. Repros `tools/perfect_kernel/repros/natbib_repeat_options/`; each pdflatex-checked.
use latexml::util::test::{assert_element, convert_files_with, kpse_text};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/natbib_repeat_options/",
      $path
    ))
  };
}

/// elsarticle.cls:1240-1243 reads the `\jobname.spl` a previous run wrote before `\RequirePackage[\@biboptions]
/// {natbib}`; a paper that ships it (2406.18569, 2503.23289) loads natbib `[numbers,sort&compress]`, so its own
/// `\usepackage[sort&compress]{natbib}` repeats no new option. pdflatex: 0 errors, "Text [1, 2]."
#[test]
fn elsarticle_reads_its_spl() {
  let (log, xml) = convert_files_with(
    repro!("elsarticle_spl.tex"),
    &[("t.spl", repro!("t.spl"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a,b\" class=\"ltx_cite_sort \
     ltx_cite_compress\" separator=\",\" show=\"Number\" yyseparator=\",\"/>]</cite>.</p>",
  );
}

/// A compiled binding's own repeat load is no load TeX sees: the cvpr binding's `[numbers,sort&compress]` after the
/// document's `\usepackage[numbers]{natbib}`, where the paper's cvpr.sty loads no natbib (2109.08809, 2408.02054),
/// raises no clash and is dropped. pdflatex: 0 errors, "Text [1, 2]."
#[test]
fn a_binding_repeat_load_does_not_clash() {
  let (log, xml) = convert_files_with(
    repro!("binding_repeat_load.tex"),
    &[("cvpr.sty", repro!("cvpr.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("Options 'sort&compress' of a binding's repeat load of natbib.sty dropped"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a,b\" separator=\",\" \
     show=\"Number\" yyseparator=\",\"/>]</cite>.</p>",
  );
}

/// `\PassOptionsToPackage` after the load counts in a later repeat's clash test, as latex.ltx:18514-18517 extends
/// `\opt@<file>`: no clash, and the passed option is never processed (2403.07005's neurips_2022.sty copy).
/// pdflatex: 0 errors, "Text (A, 2001)."
#[test]
fn a_pass_after_the_load_extends_the_option_list() {
  let (log, xml) = convert_files_with(repro!("pass_after_load.tex"), &[], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">(<bibref bibrefs=\"a\" separator=\";\" \
     show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase>, </bibrefphrase></bibref>)</cite>.</p>",
  );
}

/// A class binding's natbib load is the class's own only where the paper ships no copy of the class or one that makes
/// the class's load (`an_unmodified_class_copy_keeps_the_clash`): a copy may load natbib otherwise (2105.06643's elsarticle.cls `[authoryear]`, 2402.13480's) or not at all (2206.02837; 2209.10785's
/// cidr.cls, an acmart.cls copy), so the document's options apply. pdflatex: 0 errors, "Text (A, 2001)."
#[test]
fn a_shipped_class_copy_keeps_the_binding_load_early() {
  let (log, xml) = convert_files_with(
    repro!("class_copy_load.tex"),
    &[("elsarticle.cls", repro!("elsarticle.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">(<bibref bibrefs=\"a\" separator=\";\" \
     show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase>, </bibrefphrase></bibref>)</cite>.</p>",
  );
}

/// A shipped copy that loads natbib as the distributed class does keeps the binding's load the class's own: TeX Live's
/// elsarticle.cls copied beside the document, then `\usepackage[authoryear]{natbib}`, is LaTeX's "Option clash", the
/// citations numeric (review revREGFIXC B1: p_eA; p_eC's `[sort&compress]` the same path). pdflatex: 1 Option clash,
/// "Text [1]."
#[test]
fn an_unmodified_class_copy_keeps_the_clash() {
  let Some(cls) = kpse_text("elsarticle.cls") else {
    eprintln!("SKIP: elsarticle.cls is not in this TeX Live");
    return;
  };
  let (log, xml) = convert_files_with(
    repro!("unmodified_class_copy.tex"),
    &[("elsarticle.cls", &cls)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains("Option clash for package natbib: options 'authoryear' of a repeat load dropped"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a\" separator=\",\" \
     show=\"Number\" yyseparator=\",\"/>]</cite>.</p>",
  );
}

/// A key-value package takes a repeat load's options with no clash test (latex.ltx:18752-18762), so an option passed
/// after the load does not keep the repeat from applying it: xcolor's `table` loads colortbl (review revREGFIXC B2:
/// p_xc). pdflatex: 0 errors, the cell red.
#[test]
fn a_pass_after_the_load_leaves_a_key_value_repeat() {
  let (log, xml) = convert_files_with(repro!("xcolor_pass_after_load.tex"), &[], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "td",
    &["xml:id=\"p1.1.1.1\""],
    "<td align=\"center\" backgroundcolor=\"#FF0000\" xml:id=\"p1.1.1.1\"><text backgroundcolor=\"#FF0000\" \
     xml:id=\"p1.1.1.1.1\">A</text></td>",
  );
}
