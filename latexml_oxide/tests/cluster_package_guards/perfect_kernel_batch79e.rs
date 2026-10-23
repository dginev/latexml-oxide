//! Red/green guards for batch 79e (user ruling 2026-10-23, Option A: capital Greek follows the class): a capital Greek
//! letter `\DeclareMathSymbol`'d from the math italic symbol font (`letters`) takes the shape of the fontmath.ltx
//! glyphs of its class — a `\mathalpha` one italic unless a math alphabet is in force, as a Latin letter; a `\mathord`
//! one italic whatever the alphabet, as a lowercase Greek letter (`Font::specialize_letters`, KNOWN_PERL_ERRORS
//! #1147). The classes that declare them (llncs.cls:379-389, svmult.cls:971-981, svjour3.cls:147-157, svproc.cls:369-379),
//! the packages (fixmath raw; the `slantedGreek` option of mathptmx, mathpple, ccfonts, cmbright, newtxmath and
//! newpxmath), amsmath's and apjfonts' `\var…` letters, and a raw `\DeclareMathSymbol`. In MathML Core an italic
//! single-character `<mi>` carries no `mathvariant`, an upright one `normal`, a bold italic one its Plane 1 letter (𝜴).
//! Each guard converts its pdflatex-checked repro under `tools/perfect_kernel/repros/` (its header holds the `\showbox`
//! oracle) or a probe whose pdflatex fonts it states, and asserts every identifier element and exact diagnostic counts.
use latexml::util::test::{assert_element, convert_with};
use regex::Regex;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `tex` converted to HTML (MathML) by the binary in a directory of its own: its HTML, the conversion free of errors and
/// warnings.
fn html_of(tex: &str) -> String {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html")
}

/// Every `<mi>` element of `html`, whole, in document order.
fn identifiers(html: &str) -> Vec<String> {
  let mi = Regex::new(r"<mi(?: [^>]*)?>[^<]*</mi>").unwrap();
  mi.find_iter(html).map(|m| m.as_str().to_string()).collect()
}

const ITALIC: &str = "<mi>Ω</mi>";
const UPRIGHT: &str = "<mi mathvariant=\"normal\">Ω</mi>";
const BOLD: &str = "<mi>𝛀</mi>";
const BOLD_ITALIC: &str = "<mi>𝜴</mi>";

/// llncs.cls:379-389: `$\Omega$` CMMI10, `\mathrm` CMR10, `\mathbf` CMBX10, `\boldsymbol` and `\bm` CMMIB10, `\mathit`
/// CMTI10, amsmath's `\varOmega` CMMI10 also under `\mathrm`, the class's `\vec` (`\mbox{\boldmath$\Omega$}`, the 79b
/// residual) CMMIB10. Red (int75): every plain, bold-math and `\vec` Omega upright, `\varOmega` upright. Repro
/// fonts-nfss/llncs_capital_greek_is_the_math_italic.
#[test]
fn llncs_capital_greek_is_the_math_italic() {
  let html = html_of(repro!(
    "fonts-nfss/llncs_capital_greek_is_the_math_italic.tex"
  ));
  assert_eq!(
    identifiers(&html),
    [
      ITALIC,
      UPRIGHT,
      BOLD,
      BOLD_ITALIC,
      BOLD_ITALIC,
      ITALIC,
      ITALIC,
      ITALIC,
      BOLD_ITALIC
    ],
    "{html}"
  );
}

/// The core XML of the same: an italic capital Greek token states its shape as a Latin letter's does (`\boldmath` bold
/// italic, CMMIB10), the name and the role of the kernel's `\Omega` kept (no ID role from `\DeclareMathSymbol`).
#[test]
fn llncs_capital_greek_token_is_italic_in_the_xml() {
  let (log, xml) = convert_with(
    "\\documentclass{llncs}\n\\begin{document}\n$\\Omega$ {\\boldmath$\\Omega$}\n\\end{document}\n",
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m1\""],
    r#"<Math mode="inline" tex="\Omega" text="Omega" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok font="italic" name="Omega" role="UNKNOWN">Ω</XMTok></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m2\""],
    r#"<Math mode="inline" tex="\Omega" text="Omega" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMTok font="bold italic" name="Omega" role="UNKNOWN">Ω</XMTok></XMath></Math>"#,
  );
}

/// svmult.cls:971-981 (svmono.cls, SNmult.cls the same) declares the capital Greek letters from `letters`, its `\var…`
/// from `operators` (:982-993: upright, over amsmath's, which amsmath.sty:384 `\@ifundefined` leaves); svjour3.cls:147-157
/// and svproc.cls:369-379 the capitals, amsmath's `\varOmega` italic. pdflatex (copies, probes scratch_79e/g/{svm,svj}):
/// CMMI10, CMR10, `\varOmega` CMR10 (svmult) or CMMI10, `\boldsymbol` CMMIB10.
#[test]
fn springer_classes_capital_greek_is_the_math_italic() {
  for (class, var) in [
    ("svmult", UPRIGHT),
    ("svmono", UPRIGHT),
    ("svjour3", ITALIC),
    ("svproc", ITALIC),
  ] {
    let html = html_of(&format!(
      "\\documentclass{{{class}}}\n\\usepackage{{amsmath}}\n\\begin{{document}}\n\
       $\\Omega\\ \\mathrm{{\\Omega}}\\ \\varOmega\\ \\boldsymbol{{\\Omega}}$\n\\end{{document}}\n"
    ));
    assert_eq!(
      identifiers(&html),
      [ITALIC, UPRIGHT, var, BOLD_ITALIC],
      "{class}: {html}"
    );
  }
}

/// A raw `\DeclareMathSymbol` (a shipped class or package copy, fixmath.sty): a `\mathalpha` capital from `letters`
/// follows `\mathrm` (CMMI10, CMR10), a `\mathord` one does not (CMMI10 both), one from `operators` is upright. Red
/// (int75): all upright. Repro fonts-nfss/declare_math_symbol_from_letters_takes_the_class_shape.
#[test]
fn declare_math_symbol_from_letters_takes_the_class_shape() {
  let html = html_of(repro!(
    "fonts-nfss/declare_math_symbol_from_letters_takes_the_class_shape.tex"
  ));
  assert_eq!(
    identifiers(&html),
    [
      "<mi>Ξ</mi>",
      "<mi mathvariant=\"normal\">Ξ</mi>",
      "<mi>Ψ</mi>",
      "<mi>Ψ</mi>",
      "<mi mathvariant=\"normal\">Φ</mi>"
    ],
    "{html}"
  );
}

/// amsmath.sty:384-395: `\varOmega` is the math italic's (`\mathord`), CMMI10 also under `\mathrm`; `\Omega` upright
/// (fontmath.ltx). Red (int75): `\varOmega` upright, its italic shape lost to `Font::specialize`. Repro
/// fonts-nfss/amsmath_var_capital_greek_is_italic.
#[test]
fn amsmath_var_capital_greek_is_italic() {
  let html = html_of(repro!("fonts-nfss/amsmath_var_capital_greek_is_italic.tex"));
  assert_eq!(identifiers(&html), [ITALIC, ITALIC, UPRIGHT], "{html}");
}

/// The packages, each under its own options (pdflatex probes scratch_79e/g and /h): fixmath (raw) always italic;
/// mathptmx, mathpple, ccfonts, cmbright, newtxmath, newpxmath and mathpazo italic under `slantedGreek` only,
/// newtxmath's `frenchmath` upright; apjfonts' `\varOmega` the math italic. Under llncs, the packages whose letters are
/// upright re-declare them over the class's italic ones — newtxmath from `lettersA` (U/ntxmia), mathpazo from `upright`
/// (OT1/zplm/m/n), eulervm's Euler (U/zeur/m/n), fourier's `futm` — and mathptmx leaves them (OML/ztmcm/m/it). Repro
/// fonts-nfss/newtxmath_greek_options_follow_the_package.
#[test]
fn packages_capital_greek_follow_their_options() {
  let cases: [(&str, &str, &str, [&str; 3]); 17] = [
    ("article", "", "fixmath", [ITALIC, UPRIGHT, BOLD_ITALIC]),
    ("article", "slantedGreek", "mathptmx", [
      ITALIC,
      UPRIGHT,
      BOLD_ITALIC,
    ]),
    ("article", "", "mathptmx", [UPRIGHT, UPRIGHT, BOLD]),
    ("article", "slantedGreek", "mathpple", [
      ITALIC,
      UPRIGHT,
      BOLD_ITALIC,
    ]),
    ("article", "slantedGreek", "ccfonts", [
      ITALIC,
      UPRIGHT,
      BOLD_ITALIC,
    ]),
    ("article", "slantedGreek", "cmbright", [
      ITALIC,
      UPRIGHT,
      BOLD_ITALIC,
    ]),
    ("article", "", "cmbright", [UPRIGHT, UPRIGHT, BOLD]),
    ("article", "slantedGreek", "newtxmath", [
      ITALIC,
      UPRIGHT,
      BOLD_ITALIC,
    ]),
    ("article", "slantedGreek,frenchmath", "newtxmath", [
      UPRIGHT, UPRIGHT, BOLD,
    ]),
    ("article", "", "newtxmath", [UPRIGHT, UPRIGHT, BOLD]),
    ("article", "slantedGreek", "newpxmath", [
      ITALIC,
      UPRIGHT,
      BOLD_ITALIC,
    ]),
    ("llncs", "", "newtxmath", [UPRIGHT, UPRIGHT, BOLD]),
    ("llncs", "", "mathptmx", [ITALIC, UPRIGHT, BOLD_ITALIC]),
    ("article", "slantedGreek", "mathpazo", [
      ITALIC,
      UPRIGHT,
      BOLD_ITALIC,
    ]),
    ("llncs", "", "mathpazo", [UPRIGHT, UPRIGHT, BOLD]),
    ("llncs", "", "eulervm", [UPRIGHT, UPRIGHT, BOLD]),
    ("llncs", "", "fourier", [UPRIGHT, UPRIGHT, BOLD]),
  ];
  for (class, options, package, expected) in cases {
    let html = html_of(&format!(
      "\\documentclass{{{class}}}\n\\usepackage{{amsmath}}\n\\usepackage[{options}]{{{package}}}\n\
       \\begin{{document}}\n$\\Omega\\ \\mathrm{{\\Omega}}\\ \\boldsymbol{{\\Omega}}$\n\\end{{document}}\n"
    ));
    assert_eq!(
      identifiers(&html),
      expected,
      "{class} {package}[{options}]: {html}"
    );
  }
  let html = html_of(repro!(
    "fonts-nfss/newtxmath_greek_options_follow_the_package.tex"
  ));
  assert_eq!(identifiers(&html), [UPRIGHT], "{html}");
  let html = html_of(
    "\\documentclass{article}\n\\usepackage{apjfonts}\n\\begin{document}\n$\\varOmega\\ \\mathrm{\\varOmega}\\ \\Omega$\n\
     \\end{document}\n",
  );
  assert_eq!(identifiers(&html), [ITALIC, ITALIC, UPRIGHT], "{html}");
}

/// The packages that re-declare the capital Greek letters upright, over a class's italic ones (review rev79e finding
/// 1): stix.sty:117-127 and stix2.sty:116-126 `\stix@MathSymbol{\Gamma}{\mathalpha}{operators}{"00}`… (not under
/// `nomath`, :57-572), libertinust1math.sty:484-497 `\let\Gamma\upGamma` (`\mathalpha` from `operators`, :312) unless
/// `slantedGreek` (:48-49; `frenchmath` upright, `ISO` slanted, :62-65), then `\let\Gamma\itGamma` (`letters`, :400).
/// pdflatex (probes scratch_79e/r2/pdf): under llncs and article alike `$\Omega$`, `\mathrm`, `\boldsymbol`
/// LS1/stix/m/n, m/n, b/n (stix2, libertinust1math the same); libertinust1math `slantedGreek` or `ISO` m/it, m/n, b/it;
/// stix `nomath` leaves llncs's OML/cmm/m/it, OT1/cmr/m/n, OML/cmm/b/it. Red (79e r1): every llncs case italic. Repro
/// fonts-nfss/upright_capital_greek_packages_over_the_class (witnesses 2606.09074, 2604.00509: llncs).
#[test]
fn upright_capital_greek_packages_over_the_class() {
  let html = html_of(repro!(
    "fonts-nfss/upright_capital_greek_packages_over_the_class.tex"
  ));
  assert_eq!(
    identifiers(&html),
    [UPRIGHT, "<mi mathvariant=\"normal\">Γ</mi>"],
    "{html}"
  );
  let upright = [UPRIGHT, UPRIGHT, BOLD];
  let italic = [ITALIC, UPRIGHT, BOLD_ITALIC];
  let cases: [(&str, &str, &str, [&str; 3]); 14] = [
    ("llncs", "", "stix", upright),
    ("article", "", "stix", upright),
    ("llncs", "nomath", "stix", italic),
    ("llncs", "", "stix2", upright),
    ("article", "", "stix2", upright),
    ("llncs", "", "libertinust1math", upright),
    ("article", "", "libertinust1math", upright),
    ("llncs", "slantedGreek", "libertinust1math", italic),
    ("article", "slantedGreek", "libertinust1math", italic),
    (
      "llncs",
      "slantedGreek,uprightGreek",
      "libertinust1math",
      upright,
    ),
    (
      "llncs",
      "slantedGreek,frenchmath",
      "libertinust1math",
      upright,
    ),
    ("article", "ISO", "libertinust1math", italic),
    ("llncs", "ISO", "libertinust1math", italic),
    ("article", "uprightGreek", "libertinust1math", upright),
  ];
  for (class, options, package, expected) in cases {
    let html = html_of(&format!(
      "\\documentclass{{{class}}}\n\\usepackage{{amsmath}}\n\\usepackage[{options}]{{{package}}}\n\
       \\begin{{document}}\n$\\Omega\\ \\mathrm{{\\Omega}}\\ \\boldsymbol{{\\Omega}}$\n\\end{{document}}\n"
    ));
    assert_eq!(
      identifiers(&html),
      expected,
      "{class} {package}[{options}]: {html}"
    );
  }
}

/// mathptmx.sty:93-103 `\DeclareMathSymbol{\upGamma}{\mathord}{operators}{0}`…: the `\up…` letters are glyphs of their
/// own, upright under any class, and their `tex=` reads the source (main-session decision 1, review rev79e finding 2);
/// `\Omega` is llncs.cls's italic letter. pdflatex: OT1/ztmcm/m/n, OT1/ztmcm/m/n, OML/ztmcm/m/it. mathpazo.sty:134-144
/// (`upright`) and eulervm.sty:97-98 (`letters`, the upright Euler font) declare theirs the same way. Red (79e r1):
/// `\let\upOmega\Omega` took the class's italic letter, `tex="\Omega\Omega\Gamma"`. Repro
/// fonts-nfss/mathptmx_up_capital_greek_keeps_its_name (witness 2606.09074: llncs).
#[test]
fn mathptmx_up_capital_greek_keeps_its_name() {
  let tex = repro!("fonts-nfss/mathptmx_up_capital_greek_keeps_its_name.tex");
  let html = html_of(tex);
  assert_eq!(
    identifiers(&html),
    [UPRIGHT, "<mi mathvariant=\"normal\">Γ</mi>", ITALIC],
    "{html}"
  );
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m1\""],
    r#"<Math mode="inline" tex="\upOmega\upGamma\Omega" text="Omega * Gamma * Omega" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok name="Omega" role="UNKNOWN">Ω</XMTok><XMTok name="Gamma" role="UNKNOWN">Γ</XMTok><XMTok font="italic" name="Omega" role="UNKNOWN">Ω</XMTok></XMApp></XMath></Math>"#,
  );
  for (package, letters) in [
    ("mathpazo", "\\upOmega\\upGamma"),
    ("eulervm", "\\upOmega\\upDelta"),
  ] {
    let (log, xml) = convert_with(
      &format!(
        "\\documentclass{{llncs}}\n\\usepackage{{{package}}}\n\\begin{{document}}\n${letters}$\n\\end{{document}}\n"
      ),
      Some(PRELOAD),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{package}: {log}"
    );
    assert!(
      xml.contains(&format!("tex=\"{letters}\"")),
      "{package}: {xml}"
    );
    assert!(!xml.contains("font=\"italic\""), "{package}: {xml}");
  }
}

/// The Greek options in the order each package processes them (review rev79e finding 4): newtxmath.sty:277
/// `\ProcessOptions` runs them in declaration order (:246 `uprightGreek` before :247 `slantedGreek`), so `slantedGreek`
/// wins whatever order the user writes; newpxmath.sty:188 `\ProcessOptions*` and libertinust1math.sty:59
/// `\ProcessOptionsX` run them in the order given, the last winning (libertinust1math's `ISO` slanted after
/// `frenchmath`, :62-65). pdflatex (probes scratch_79e/r2/pdf): newtxmath both orders OML/ntxmi/m/it; newpxmath
/// `[slantedGreek,uprightGreek]` U/npxmia (upright `lettersA`), `[uprightGreek,slantedGreek]` OML/npxmi/m/it;
/// libertinust1math `[uprightGreek,slantedGreek]` and `[ISO,frenchmath]` LS1/libertinust1math/m/it. Red (79e r1):
/// newtxmath `[slantedGreek,uprightGreek]` upright. Witness 2508.05134 (newpxmath `slantedGreek`).
#[test]
fn greek_options_follow_the_package_order() {
  let upright = [UPRIGHT, UPRIGHT, BOLD];
  let italic = [ITALIC, UPRIGHT, BOLD_ITALIC];
  let cases: [(&str, &str, [&str; 3]); 6] = [
    ("slantedGreek,uprightGreek", "newtxmath", italic),
    ("uprightGreek,slantedGreek", "newtxmath", italic),
    ("slantedGreek,uprightGreek", "newpxmath", upright),
    ("uprightGreek,slantedGreek", "newpxmath", italic),
    ("uprightGreek,slantedGreek", "libertinust1math", italic),
    ("ISO,frenchmath", "libertinust1math", italic),
  ];
  for (options, package, expected) in cases {
    let html = html_of(&format!(
      "\\documentclass{{article}}\n\\usepackage{{amsmath}}\n\\usepackage[{options}]{{{package}}}\n\
       \\begin{{document}}\n$\\Omega\\ \\mathrm{{\\Omega}}\\ \\boldsymbol{{\\Omega}}$\n\\end{{document}}\n"
    ));
    assert_eq!(identifiers(&html), expected, "{package}[{options}]: {html}");
  }
}
