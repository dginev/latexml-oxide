//! Red/green guards for perfect-kernel batch 65d (C5 step G2): `\left`, `\middle` and `\right` are TeX's unexpandable
//! `left_right` command (tex.web §1188) with §1160 `scan_delimiter` (`tex_math::scan_delimiter`): `\left` opens a
//! `math_left_group` that `\middle` and `\right` require (§1191-1192). OXIDIZED_DESIGN #530. Repros
//! `tools/perfect_kernel/repros/expansion-primitives/{left_right_are_unexpandable,let_left_wrapped_in_mathclose,
//! left_in_xymatrix_cell}.tex`, `math-parse/extra_right_and_middle_without_left.tex` (CONTROL); the delimiter table and
//! `\middle` are the golden pair `tests/parse/left_right_delimiters`.
use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// The opening tags of every `<Math>`, in document order.
fn math_tags(xml: &str) -> Vec<&str> {
  xml
    .split("<Math ")
    .skip(1)
    .filter_map(|m| m.find('>').map(|end| &m[..end]))
    .collect()
}

/// The `Warning:` lines of a log, ANSI-free, without the parser's `unparsed_math` reports.
fn warning_lines(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|l| l.find("Warning:").map(|i| &l[i..]))
    .filter(|l| !l.starts_with("Warning:unparsed_math") && !l.starts_with("Warning:ambiguous_math"))
    .collect()
}

/// The `Error:` lines of a log, ANSI-free.
fn error_lines(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|l| l.find("Error:").map(|i| &l[i..]))
    .collect()
}

/// tex.web §1188: `\left`, `\middle` and `\right` are unexpandable, so `\edef` keeps them (pdflatex `\show\x` =
/// `macro:->\left (`). Base: `\left` was a macro, `\x` = `macro:->\lx@delim@left (\lx@hidden@bgroup `.
#[test]
fn left_right_are_unexpandable() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/left_right_are_unexpandable.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    xml.contains(r#"<text font="typewriter" xml:id="p1.1.1">macro:-&gt;\left (</text>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text font="typewriter" xml:id="p1.1.2">macro:-&gt;\right )\middle |</text>"#),
    "{xml}"
  );
}

/// The mathtools-documentation idiom `\let\@left\left \def\left{\mathopen{}\mathclose\bgroup\@left}` (and the
/// `\right` twin, 2605.21750's shape): the copies are the constructors themselves, nothing re-enters a trampoline.
/// 0 errors; the three formulas keep 65a's readings (the right delimiter digests after the `\mathclose` group's
/// `\aftergroup\egroup`, as in Perl: unparsed, one warning each).
#[test]
fn let_left_wrapped_in_mathclose() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/let_left_wrapped_in_mathclose.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 3, "{log}");
  assert_eq!(
    math_tags(&xml),
    [
      r#"class="ltx_math_unparsed" mode="inline" tex="f\mathopen{}\mathclose{{\left(x}}\right)" xml:id="p1.m1""#,
      r#"class="ltx_math_unparsed" mode="inline" tex="\mathopen{}\mathclose{{\left[a+b}}\right]^{2}" xml:id="p1.m2""#,
      r#"class="ltx_math_unparsed" mode="inline" tex="\mathopen{}\mathclose{{\left\{y\middle|y&gt;0}}\right\}" xml:id="p1.m3""#,
    ],
    "{xml}"
  );
}

/// tex.web §1192: a `\right` or `\middle` at the formula's own group is "Extra \right." / "Extra \middle." — pdflatex's
/// 2 errors, and Perl's ("Unbalanced \right"). TeX drops the delimiter; it is kept, typeset where it stands (user
/// ruling 2026-10-12: the delimiter is the author's content), so the formulas read `a)` and `b|c` (base: the same
/// output, 1 error, `\middle` unchecked).
#[test]
fn extra_right_and_middle_without_left() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/extra_right_and_middle_without_left.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    [
      "Error:unexpected:\\right Extra \\right.",
      "Error:unexpected:\\middle Extra \\middle."
    ],
    "{log}"
  );
  assert_eq!(warning_lines(&log), Vec::<&str>::new(), "{log}");
  assert_eq!(
    math_tags(&xml),
    [
      r#"mode="inline" tex="a\right)" text="delimited-)@(a)" xml:id="p1.m1""#,
      r#"class="ltx_math_unparsed" mode="inline" tex="b\middle|c" xml:id="p1.m2""#,
    ],
    "{xml}"
  );
}

/// The whole of a probe document's output: every paragraph's text with each formula as `[M]`, and every `<Math>` tag.
fn paragraphs(xml: &str) -> Vec<String> {
  xml
    .split("<p xml:id=\"")
    .skip(1)
    .filter_map(|p| {
      let body = &p[p.find('>')? + 1..p.find("</p>")?];
      let mut text = String::new();
      let mut rest = body;
      while let Some(start) = rest.find("<Math ") {
        text.push_str(&rest[..start]);
        text.push_str("[M]");
        rest = &rest[rest.find("</Math>")? + "</Math>".len()..];
      }
      text.push_str(rest);
      Some(text)
    })
    .collect()
}

/// A probe (xcolor loaded) `Before\n\n<formula> <tail>\n\nAfter para.\n\n$z$ final.` converts with exactly `warnings` (besides the
/// math parser's), no error, the formula as `math`, and the paragraphs after it whole.
fn assert_probe(formula: &str, tail: &str, warnings: &[&str], math: &str, equation: Option<&str>) {
  let tex = format!(
    "\\documentclass{{article}}\n\\usepackage{{xcolor}}\n\\begin{{document}}\nBefore\n\n{formula} {tail}\n\nAfter para.\n\n$z$ final.\n\\end{{document}}\n"
  );
  let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{formula}: {log}");
  assert_eq!(warning_lines(&log), warnings, "{formula}: {log}");
  let second = if equation.is_some() {
    tail.to_string()
  } else {
    format!("[M] {tail}")
  };
  assert_eq!(
    paragraphs(&xml),
    [
      "Before".to_string(),
      second,
      "After para.".to_string(),
      "[M] final.".to_string()
    ],
    "{formula}: {xml}"
  );
  let id = equation.map_or("p2.m1".to_string(), |eq| format!("{eq}.m1"));
  let mode = if equation.is_some() {
    "display"
  } else {
    "inline"
  };
  assert_eq!(
    math_tags(&xml),
    [
      format!(r#"mode="{mode}" {math} xml:id="{id}""#),
      r#"mode="inline" tex="z" text="z" xml:id="p4.m1""#.to_string()
    ],
    "{formula}: {xml}"
  );
  if let Some(eq) = equation {
    assert!(
      xml.contains(&format!("<equation xml:id=\"{eq}\">")),
      "{formula}: {xml}"
    );
  }
}

/// §1191 + §1064 `off_save`: a `\right` (or `\middle`) inside a `{` group of the fence inserts the missing `}` — a
/// Warning (pdflatex errs "Missing } inserted"; Perl closes the brace group silently; the inserted brace is TeX's
/// recovery: user ruling 2026-10-11) — and the author's own `}`, when it comes, is deleted as §1069 deletes it
/// ("Extra }, or forgotten $" / "\right."), so the formula ends at its `$` and the paragraphs after it stay text.
/// pdflatex gives the same two errors for each shape. Review r1 of 65d: the stray `}` reached the formula's frame,
/// inserted `$` and set "c", "After para." and the rest as math (65a: no diagnostic, `\right` closed the brace group).
#[test]
fn right_in_a_brace_group_inserts_the_brace() {
  const BRACE: [&str; 2] = [
    "Warning:expected:} Missing } inserted",
    "Warning:unexpected:} Extra }, or forgotten $",
  ];
  assert_probe(
    r"$\left( {a \right)} b$",
    "c",
    &BRACE,
    r#"tex="\left({a}\right)b" text="a * b""#,
    None,
  );
  assert_probe(
    r"$\left( {a \middle| b} \right)$",
    "c",
    &[
      "Warning:expected:} Missing } inserted",
      "Warning:unexpected:} Extra }, or forgotten \\right.",
    ],
    r#"tex="\left({a}\middle|b\right)" text="conditional@(a, b)""#,
    None,
  );
  assert_probe(
    r"$\left( x^{2 \right)} c$",
    "d",
    &BRACE,
    r#"tex="\left(x^{2}\right)c" text="x ^ 2 * c""#,
    None,
  );
  assert_probe(
    r"$\left( a {\color{red} b \right)} c$",
    "d",
    &BRACE,
    r#"tex="\left(a{\color[rgb]{1,0,0}b}\right)c" text="a * b * c""#,
    None,
  );
  assert_probe(
    r"\begin{equation}\left( a {\bf b \right)} c\end{equation}",
    "d",
    &BRACE,
    r#"tex="\left(a{\bf b}\right)c" text="a * b * c""#,
    Some("S0.E1"),
  );
}

/// The closers `off_save` inserted are owed by the group it closed, and the debt follows the stack: at every pop it
/// moves to the group then innermost (stomach.rs `pop_stack_frame`), so the author's own closer is deleted wherever it
/// meets a group it cannot close — after `\middle` and then `\right` (m1: the `}` comes after the `\left` group
/// closed), inside an enclosing `{` group the `}` closes normally and the debt passes on to the formula (b5), and an
/// `\endgroup` the same way ("Extra \endgroup.", b3; pdflatex flips text and math there, 4 errors). pdflatex: 2 errors
/// for m1 and b5. Review r3 of 65d: m1 set "d." and "After para." as math.
#[test]
fn inserted_closer_debt_follows_the_stack() {
  assert_probe(
    r"$\left( {a \middle| b \right)} c$",
    "d.",
    &[
      "Warning:expected:} Missing } inserted",
      "Warning:unexpected:} Extra }, or forgotten $",
    ],
    r#"tex="\left({a}\middle|b\right)c" text="conditional@(a, b) * c""#,
    None,
  );
  assert_probe(
    r"$x {\left( {a \right)} \mathbf{b}} c$",
    "e.",
    &[
      "Warning:expected:} Missing } inserted",
      "Warning:unexpected:} Extra }, or forgotten $",
    ],
    r#"tex="x{\left({a}\right)}\mathbf{b}c" text="x@(a) * b * c""#,
    None,
  );
  assert_probe(
    r"$\left( \begingroup a \right) \endgroup b$",
    "c e.",
    &[
      "Warning:expected:\\endgroup Missing \\endgroup inserted",
      "Warning:unexpected:\\endgroup Extra \\endgroup.",
    ],
    r#"tex="\left(a\right)b" text="a * b""#,
    None,
  );
}

/// A brace where a delimiter belongs takes TeX's path (§1160-1166): "Missing delimiter (. inserted)", the null
/// delimiter, and the brace read again — it cannot tell the typo `\left{` from a brace-bodied macro (`\newcommand\lb
/// {{[}}`, LaTeX's `\dag`), and the PDF shows no brace either way. The typo `\left{ x \right}` then reads as pdflatex
/// shows it (4 errors there): null delimiters, `x` in a group, the trailing `}` deleted. g1/g2 (pdflatex 1 error each;
/// pdftotext "x † c y e." / "[x] c y e."): the macro's content follows the null delimiter.
#[test]
fn brace_where_a_delimiter_belongs_takes_texs_path() {
  assert_probe(
    r"$\left{ x \right}$",
    "c",
    &[
      "Warning:unexpected:{ Missing delimiter (. inserted)",
      "Warning:expected:} Missing } inserted",
      "Warning:unexpected:} Missing delimiter (. inserted)",
      "Warning:unexpected:} Extra }, or forgotten $",
    ],
    r#"tex="\left.{x}\right." text="x""#,
    None,
  );
  for (formula, math) in [
    (
      r"$\left. x \right\dag$",
      r#"class="ltx_math_unparsed" mode="inline" tex="\left.x\right.{\dagger}""#,
    ),
    (
      r"$\left\lb x \right]$",
      r#"mode="inline" tex="\left.{[}x\right]" text="delimited-[]@(x)""#,
    ),
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n\\newcommand\\lb{{{{[}}}}\n\\begin{{document}}\nBefore\n\n{formula} c $y$ e.\n\nAfter para.\n\\end{{document}}\n"
    );
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(error_count(&log), 0, "{formula}: {log}");
    assert_eq!(
      warning_lines(&log),
      ["Warning:unexpected:{ Missing delimiter (. inserted)"],
      "{formula}: {log}"
    );
    assert_eq!(
      paragraphs(&xml),
      ["Before", "[M] c [M] e.", "After para."],
      "{formula}: {xml}"
    );
    assert_eq!(
      math_tags(&xml),
      [
        format!(r#"{math} xml:id="p2.m1""#),
        r#"mode="inline" tex="y" text="y" xml:id="p2.m2""#.to_string()
      ],
      "{formula}: {xml}"
    );
  }
}

/// tex.web §1065 at the primitive that meets an open `\left` group: "Missing \right. inserted" and `\right.` read
/// before it — an `\endgroup` (pdflatex 1 error) — while a LaTeX environment whose end code holds the `\right`
/// (`\newenvironment{paren}{\left(}{\right)}`) is clean, its `\endparen` closing the group (review r3: an `\end`-time
/// check inserted `\right.` there, and looped in an `\edef`). The `\edef` shape (pdflatex errs on it) terminates.
#[test]
fn group_ends_inside_a_left_group() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\nBefore $\\begingroup \\left( a \\endgroup b$ c.\n\nAfter para.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(
    warning_lines(&log),
    ["Warning:expected:\\right Missing \\right. inserted"],
    "{log}"
  );
  assert_eq!(paragraphs(&xml), ["Before [M] c.", "After para."], "{xml}");
  assert_eq!(
    math_tags(&xml),
    [r#"mode="inline" tex="\left(a\right.b" text="delimited-(@(a * b)" xml:id="p1.m1""#],
    "{xml}"
  );

  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\newenvironment{paren}{\\left(}{\\right)}\n\
     \\newenvironment{brk}{\\left[\\,}{\\,\\right]}\n\\begin{document}\nBefore $\\begin{paren} x+y \\end{paren}^2$ and\n\
     \\begin{equation}\\begin{brk} a \\\\ b \\end{brk} = c\\end{equation}\nAfter para.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_lines(&log), Vec::<&str>::new(), "{log}");
  let tex: Vec<&str> = math_tags(&xml)
    .iter()
    .filter_map(|t| t.split("tex=\"").nth(1).and_then(|t| t.split('"').next()))
    .collect();
  assert_eq!(
    tex,
    [r"\left(x+y\right)^{2}", r"\left[\,a\\&#10;b\,\right]=c"],
    "{xml}"
  );

  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\newenvironment{paren}{\\left(}{\\right)}\n\\begin{document}\n\
     Before $\\left( a \\edef\\z{\\end{paren}} b \\right)$ c.\n\nAfter para.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(paragraphs(&xml), ["Before [M] c.", "After para."], "{xml}");
}

/// The trial digestion that decides whether a parameterless command typesets a symbol reports nothing: `\left\hline`
/// gives one "\noalign cannot be used here" (from the real digestion), not two (review r3).
#[test]
fn delimiter_trial_digestion_is_silent() {
  let (log, _xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n$\\left\\hline x\\right.$\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(
    log.matches("\\noalign cannot be used here").count(),
    1,
    "{log}"
  );
}

/// An environment's end inside an open `\left` group (`\begin{equation}\left( x \end{equation}`): tex.web §1065
/// inserts `\right.` before the environment's math ends ("Missing \right. inserted", pdflatex 1 error for
/// `equation*`), a Warning. Review r1 of 65d: "Attempt to end mode display_math", and the formula ran on.
#[test]
fn environment_end_closes_an_open_left() {
  for (env, eq) in [("equation", "S0.E1"), ("equation*", "S0.Ex1")] {
    let tex = format!(
      "\\documentclass{{article}}\n\\begin{{document}}\nBefore\n\n\\begin{{{env}}} \\left( x \\end{{{env}}} e\n\nAfter para.\n\\end{{document}}\n"
    );
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(error_count(&log), 0, "{env}: {log}");
    assert_eq!(
      warning_lines(&log),
      ["Warning:expected:\\right Missing \\right. inserted"],
      "{env}: {log}"
    );
    assert_eq!(
      paragraphs(&xml),
      ["Before", "e", "After para."],
      "{env}: {xml}"
    );
    assert_eq!(
      math_tags(&xml),
      [format!(
        r#"mode="display" tex="\left(x\right." text="delimited-(@(x)" xml:id="{eq}.m1""#
      )],
      "{env}: {xml}"
    );
  }
}

/// tex.web §1160 for a parameterless command: a delimiter only when it typesets a symbol (`\lmoustache`, a `DefMath`
/// symbol); `\displaystyle`, `\nonumber`, `\par`, `\hfill`, `\,` are "Missing delimiter (. inserted)" Warnings, put
/// back, and no neighbour is made stretchy (review r1: `y\left\displaystyle` marked `y` stretchy; `\left\par` silent).
/// The input ending where a delimiter belongs (`\frac{a\left}{b}`) warns too.
#[test]
fn parameterless_commands_are_delimiters_only_as_symbols() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n$y\\left\\displaystyle x\\right)$ $z\\left\\nonumber x\\right)$ \
     $\\left\\lmoustache x\\right\\rmoustache$ $\\frac{a\\left}{b}$ $w\\left\\par x\\right)$\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  // The put-back `\par` then ends the inline formula (tex.web §1046-1047, "Missing $ inserted", the `\left` group first
  // given its `\right.`), so `\right)` meets no `\left` ("Extra \right.") and the formula's own `$` opens one the
  // document's end closes — pdflatex's sequence (5 errors; the delimiter and `\right.` ones are Warnings here, user ruling
  // 2026-10-11). Kept last so the cascade reaches no other formula (65n, OXIDIZED_DESIGN #553).
  assert_eq!(
    error_lines(&log),
    [
      "Error:expected:$ Missing $ inserted",
      "Error:unexpected:\\right Extra \\right.",
      "Error:expected:$ Missing $ inserted"
    ],
    "{log}"
  );
  assert_eq!(
    warning_lines(&log),
    [
      "Warning:unexpected:\\displaystyle Missing delimiter (. inserted)",
      "Warning:unexpected:\\lx@equation@nonumber Missing delimiter (. inserted)",
      "Warning:unexpected:\\left Missing delimiter (. inserted)",
      "Warning:expected:\\right Missing \\right. inserted",
      "Warning:unexpected:\\par Missing delimiter (. inserted)",
      "Warning:expected:\\right Missing \\right. inserted",
    ],
    "{log}"
  );
  for letter in ["y", "z", "w"] {
    assert!(
      xml.contains(&format!(
        r#"<XMTok font="italic" role="UNKNOWN">{letter}</XMTok>"#
      )),
      "{letter}: {xml}"
    );
  }
  assert!(
    xml.contains(r#"<XMTok role="UNKNOWN" stretchy="true">⎰</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok role="UNKNOWN" stretchy="true">⎱</XMTok>"#),
    "{xml}"
  );
}

/// tex.web §1160: a token that is no delimiter is "Missing delimiter (. inserted)" — it is put back and read as math,
/// the fence's left side null. A Warning: pdflatex errs, Perl is silent, and the token is still typeset (user ruling
/// 2026-10-11 on TeX-only errors). Base: one warning ("Missing delimiter; '.' inserted") for neither.
#[test]
fn missing_delimiter_inserts_a_null_one() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n$\\left\\mathopen(x\\right)$ $\\left a x \\right)$\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(
    warning_lines(&log),
    [
      "Warning:unexpected:\\mathopen Missing delimiter (. inserted)",
      "Warning:unexpected:a Missing delimiter (. inserted)"
    ],
    "{log}"
  );
  assert_eq!(
    math_tags(&xml),
    [
      r#"mode="inline" tex="\left.\mathopen{(}x\right)" text="x" xml:id="p1.m1""#,
      r#"mode="inline" tex="\left.ax\right)" text="delimited-)@(a * x)" xml:id="p1.m2""#,
    ],
    "{xml}"
  );
}

/// tex.web §1069: a `}` against the `\left` group is "Extra }, or forgotten \right." (pdflatex: that and "Missing
/// \right. inserted", 2 errors). The group closes, as Perl closes it, so a Warning (user ruling 2026-10-11). Base:
/// silent.
#[test]
fn brace_closing_a_left_group_warns() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n$\\left( a} b$ c\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(
    warning_lines(&log),
    ["Warning:unexpected:} Extra }, or forgotten \\right."],
    "{log}"
  );
  assert_eq!(
    math_tags(&xml),
    [r#"mode="inline" tex="\left(ab" text="delimited-(@(a * b)" xml:id="p1.m1""#],
    "{xml}"
  );
  assert!(xml.contains("</Math> c</p>"), "{xml}");
}

/// tex.web §1065 `off_save`: a formula (`$`, `\]`) or an argument (`\frac{\left( a}{b}`) ending inside a `\left`
/// group gets "Missing \right. inserted" and a null `\right.` (pdflatex: one error each). A Warning (user ruling
/// 2026-10-11). Base: no diagnostic for the open group, and the formula went on to the next `$`, swallowing the
/// paragraphs after it.
#[test]
fn formula_end_closes_an_open_left() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n$\\left( a$ b\n\n$\\frac{\\left( a}{b}$ c\n\n\\[ \\left( x \\] d\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(
    warning_lines(&log),
    [
      "Warning:expected:\\right Missing \\right. inserted",
      "Warning:expected:\\right Missing \\right. inserted",
      "Warning:expected:\\right Missing \\right. inserted"
    ],
    "{log}"
  );
  assert_eq!(
    math_tags(&xml),
    [
      r#"mode="inline" tex="\left(a\right." text="delimited-(@(a)" xml:id="p1.m1""#,
      r#"mode="inline" tex="\frac{\left(a\right.}{b}" text="delimited-(@(a) / b" xml:id="p2.m1""#,
      r#"mode="display" tex="\left(x\right." text="delimited-(@(x)" xml:id="S0.Ex1.m1""#,
    ],
    "{xml}"
  );
  for text in ["</Math> b</p>", "</Math> c</p>", ">d</p>"] {
    assert!(xml.contains(text), "{text}: {xml}");
  }
}

/// Run 336's X1 (62 Fatal:Timeout:IfLimit; 0705.0452 0802.0663 0804.1292 0809.0095 0903.4903): xy's cell scanner
/// expanded the expandable `\left`, which read `\{` ahead and desynchronized the conditionals (base: 12 errors). The
/// unexpandable `\left` leaves the cell whole.
#[test]
fn left_in_xymatrix_cell() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/left_in_xymatrix_cell.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    math_tags(&xml),
    [
      r#"mode="inline" tex="\textstyle{\left\{x\right\}}" text="set@(x)" xml:id="S0.Ex1.m1.pic1.m1""#,
      r#"mode="inline" tex="\textstyle{B}" text="B" xml:id="S0.Ex1.m1.pic1.m2""#,
    ],
    "{xml}"
  );
}

/// The case that made `\left` a macro (WISDOM #41; golden `tests/parse/parens.tex`:43): `\left\delimiter"426830A`
/// must read the code, which Rust's `TeXDelimiter` could not (no single-token read, no `undigested` invoke). §1160
/// matches `\delimiter` by meaning and scans the 27-bit number right there, so a `\let` copy works too (base: the copy
/// was the delimiter and its code digested as math, `Γ * " * 426830 * A`, 4 warnings).
#[test]
fn delimiter_codes_and_let_copies() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n\\[ \\left\\delimiter\"426830A {a \\over b} \\right\\delimiter\"526930B \\]\n\\let\\mydelim\\delimiter\n\\[ \\left\\mydelim\"426830A {a \\over b} \\right\\mydelim\"526930B \\]\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    math_tags(&xml),
    [
      r#"mode="display" tex="\left⟨{a\over b}\right⟩" text="delimited-⟨⟩@(a / b)" xml:id="S0.Ex1.m1""#,
      r#"mode="display" tex="\left⟨{a\over b}\right⟩" text="delimited-⟨⟩@(a / b)" xml:id="S0.Ex2.m1""#,
    ],
    "{xml}"
  );
}

/// Perl TeX_Math.pool.ltxml:782-790: `\right` is one constructor "because \right is used in the LaTeX 3 kernel with the
/// assumption that it is idempotent to expansion". expl3's `\tl_set:Nx` and latex.ltx's `\protected@edef` keep all three
/// unexpanded (pdflatex `macro:->\left (x\right )`; base: `\lx@delim@left (\lx@hidden@bgroup x\right )`), and the
/// stored formulas, mathtools' `\DeclarePairedDelimiter(X)` and physics' `\qty`/`\abs`/`\norm` read as before.
#[test]
fn left_right_survive_expansion_contexts() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{amsmath,mathtools,physics}
\DeclarePairedDelimiter\absx{\lvert}{\rvert}
\DeclarePairedDelimiterX\setx[2]{\{}{\}}{#1\;\delimsize\vert\;#2}
\begin{document}
\ExplSyntaxOn
\tl_set:Nx \l_tmpa_tl { \left( x \right) }
\cs_new:Npn \showtl { \texttt{\meaning\l_tmpa_tl} }
\ExplSyntaxOff
\showtl{} $\csname l_tmpa_tl\endcsname$
\makeatletter
\protected@edef\x{\left(a\middle|b\right)}
\makeatother
\texttt{\meaning\x} $\x$
$\absx*{\frac{a}{b}} \absx{x} \setx*{a}{b} \qty(\frac{a}{b}) \abs{x} \norm{y}$
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    xml.contains(r#"<text font="typewriter" xml:id="p1.1.1">macro:-&gt;\left (x\right )</text>"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<text font="typewriter" xml:id="p1.1.2">macro:-&gt;\left (a\middle |b\right )</text>"#
    ),
    "{xml}"
  );
  let texts: Vec<&str> = math_tags(&xml)
    .iter()
    .filter_map(|tag| {
      tag
        .split(" text=\"")
        .nth(1)
        .and_then(|t| t.split('"').next())
    })
    .collect();
  assert_eq!(
    texts,
    [
      "x",
      "conditional@(a, b)",
      "absolute-value@(a / b) * absolute-value@(x) * conditional-set@(a, b) * (a / b) * absolute-value@(x) * norm@(y)",
    ],
    "{xml}"
  );
}

/// braket's `\Set` splits at its first `|`, which braket.sty makes math-active; a `\left|` there is the fence's own
/// delimiter (tex.web §1160 takes the `\delcode`, never the active meaning). The binding split there (Perl too, KPE
/// #607; 2107.02628): base read `\Set{\omega:\left}{f(\omega)}` and dropped `< 1`, and since 65d's `\middle` check
/// that shape is an "Extra \middle." error.
#[test]
fn braket_set_keeps_fence_bars() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{braket}\n\\begin{document}\n$\\Set{\\omega : \\left|f(\\omega)\\right| < 1}$ and $\\Set{x | \\bigl|x\\bigr| > 0}$\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    math_tags(&xml),
    [
      r#"mode="inline" tex="\Set{\omega:\left|f(\omega)\right|&lt;1}" text="set@(omega colon absolute-value@(f@(omega)) &lt; 1)" xml:id="p1.m1""#,
      r#"mode="inline" tex="\Set{x}{\bigl|x\bigr|&gt;0}" text="set@(x, absolute-value@(x) &gt; 0)" xml:id="p1.m2""#,
    ],
    "{xml}"
  );
}

/// amsmath.sty:1121-1126 `\env@cases`: zhlineskip.tex:234's `\newenvironment{originalcases}{\env@cases}{\endarray
/// \right.}` reads as amsmath's `{cases}` (base: `\env@cases` undefined, 4 errors; with 65d's group check the
/// unmatched `\right.` met the environment's group, "Missing \endgroup inserted" and a cascade, zhlineskip 6 → 12).
#[test]
fn env_cases_opens_its_left_brace() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{amsmath}
\makeatletter
\newenvironment{originalcases}{\env@cases}{\endarray\right.}
\makeatother
\begin{document}
\[ f = \begin{originalcases} 0, & j=1,\\ 1, & j>1, \end{originalcases} \]
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    math_tags(&xml),
    [
      "mode=\"display\" tex=\"f=\\left\\{\\begin{array}[]{@{}l@{\\quad}l@{}}0,&amp;j=1,\\\\&#10;1,&amp;j&gt;1,\\end{array}\\right.\" text=\"f = cases@(Array[[0, j = 1], [1, j &gt; 1]])\" xml:id=\"S0.Ex1.m1\""
    ],
    "{xml}"
  );
}

/// An argument group the engine opens to digest a constructor's argument (`\frac`'s `InFractionStyle`, a K9 group
/// code) is a group `}` cannot close: the author's `}` after an inserted one is deleted there (pdflatex: "Missing }
/// inserted", "Extra }, or forgotten \endgroup" — LaTeX's `\frac` has `\begingroup`; pdftotext "(a)/b c d e."). Review
/// r3 b4: the `}` closed `\frac`'s argument group and `\frac`'s closer erred.
#[test]
fn argument_group_owes_the_brace() {
  assert_probe(
    r"$\frac{\left( {a \right)}}{b} c$",
    "d e.",
    &[
      "Warning:expected:} Missing } inserted",
      "Warning:unexpected:} Extra }, or forgotten \\endgroup",
    ],
    r#"tex="\frac{\left({a}\right)}{b}c" text="(a / b) * c""#,
    None,
  );
}

/// tex.web §1160 + §1069: the `}` a delimiter scan puts back after "Missing delimiter" (`\right}`) meets the formula's
/// own math shift and is deleted, "Extra }, or forgotten $" — an Error, as TeX and Perl err — whatever group is
/// outside the formula, and the formula ends at its own `$`. y2 `$\left\{ x \right}$ c` (pdflatex "Missing delimiter",
/// "Extra }, or forgotten $"; 65a 1 Error); g1 `\mbox{$\left\{ x \right}$} c.` (pdflatex the same two, pdftotext "Before
/// {x c."; 65a: the `}` closed the math, 1 Error). The put-back `}` is the author's delimiter, never a construct's
/// closer — the box's own `}` follows the `$` — so it is deleted even in a box body; every other `}` at an inline
/// formula's math shift follows 65n's unit-construct rule (OXIDIZED_DESIGN #553: the formula ends when the brace closes a
/// box body or an argument digested on its own, `boxes-groups/mbox_math_stray_brace_ends_box_not_math`, and the brace is
/// deleted otherwise, `paragraph-mode/stray_brace_then_dollar_keeps_text`).
#[test]
fn stray_brace_at_the_formula_level_is_deleted() {
  for (body, paragraphs_expected, math) in [
    (
      "Before\n\n$\\left\\{ x \\right}$ c\n\nAfter para.\n\n$z$ final.",
      &["Before", "[M] c", "After para.", "[M] final."][..],
      [
        r#"class="ltx_math_unparsed" mode="inline" tex="\left\{x\right." xml:id="p2.m1""#,
        r#"mode="inline" tex="z" text="z" xml:id="p4.m1""#,
      ],
    ),
    (
      "Before \\mbox{$\\left\\{ x \\right}$} c.\n\nAfter para.\n\n$z$ final.",
      &[
        r#"Before <text xml:id="p1.1.1">[M]</text> c."#,
        "After para.",
        "[M] final.",
      ][..],
      [
        r#"class="ltx_math_unparsed" mode="inline" tex="\left\{x\right." xml:id="p1.m1""#,
        r#"mode="inline" tex="z" text="z" xml:id="p3.m1""#,
      ],
    ),
  ] {
    let tex =
      format!("\\documentclass{{article}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n");
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert_eq!(
      error_lines(&log),
      ["Error:unexpected:} Extra }, or forgotten $"],
      "{body}: {log}"
    );
    assert_eq!(
      warning_lines(&log),
      ["Warning:unexpected:} Missing delimiter (. inserted)"],
      "{body}: {log}"
    );
    assert_eq!(paragraphs(&xml), paragraphs_expected, "{body}: {xml}");
    assert_eq!(math_tags(&xml), math, "{body}: {xml}");
  }
}

/// The put-back mark lives as long as its brace: the `}` an alignment cell digests after `\right}` clears it (the cell
/// loop is not one of the digestion loops that delete it), so the stale mark does not reach a later, unrelated `$x}`
/// (review r6: it deleted that `}` while no `\par` ended the formula, and "after one. Second para." became math). Since
/// 65n that `}` is deleted by the general tex.web §1069 rule ("Extra }, or forgotten $") and the `\par` ends the formula
/// ("Missing $ inserted", §1046-1047), so "after one." is math to the paragraph's end and "Second para." text, as
/// pdflatex (OXIDIZED_DESIGN #553). Shapes P1 (`cases`), P2 (`pmatrix` in display math), N1 (`array`).
#[test]
fn put_back_mark_lives_with_its_brace() {
  for (first, paragraphs_expected) in [
    (
      "$f=\\begin{cases} \\left( a \\right} & b \\\\ c & d \\end{cases}$ then.",
      &["[M] then.", "Before [M]", "Second para."][..],
    ),
    (
      "\\[ \\begin{pmatrix} \\left( a \\right} \\\\ c \\end{pmatrix} \\]",
      &["Before [M]", "Second para."][..],
    ),
    (
      "$\\begin{array}{c} \\left( a \\right} \\\\ b \\end{array}$ then.",
      &["[M] then.", "Before [M]", "Second para."][..],
    ),
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n\\usepackage{{amsmath}}\n\\begin{{document}}\n{first}\n\nBefore $x}} after one.\n\nSecond para.\n\\end{{document}}\n"
    );
    let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
    let errors = error_lines(&log);
    assert_eq!(errors.len(), 3, "{first}: {log}");
    assert_eq!(
      errors[1..],
      [
        "Error:unexpected:} Extra }, or forgotten $",
        "Error:expected:$ Missing $ inserted"
      ],
      "{first}: {log}"
    );
    assert_eq!(paragraphs(&xml), paragraphs_expected, "{first}: {xml}");
  }
}
