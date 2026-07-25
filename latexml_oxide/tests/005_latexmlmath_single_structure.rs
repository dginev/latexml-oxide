// `latexmlmath_oxide` used to emit an empty `<mrow/>` for any formula whose
// ENTIRE body is one top-level structure (`\frac{1}{2}`, `\sqrt{2}`), while
// `\frac{a}{b}+c` — the same fraction with something beside it — worked.
//
// Cause: `MathParser::into_xmath` RETURNS the built tree, it does not attach it.
// The main parser path (`latexml_math_parser/src/parser.rs`, the reparent step)
// pairs it with `append_tree`; this binary unlinked the pre-parse children and
// then never put the parsed tree back, so a single-structure body left the
// `<XMath>` empty. Multi-part bodies only appeared to survive because their
// parts are pre-existing lexeme nodes already in the tree — which is also why
// an UNCONDITIONAL append is wrong: it renders `\frac{a}{b}+c` twice. The
// append is therefore gated on the tree being detached.
//
// Guards both halves: single-structure bodies must render their structure, and
// multi-part bodies must render it exactly ONCE.
use std::process::Command;

fn math(tex: &str) -> String {
  let out = Command::new(env!("CARGO_BIN_EXE_latexmlmath_oxide"))
    .args(["--quiet", tex])
    .output()
    .expect("failed to run latexmlmath_oxide");
  assert!(
    out.status.success(),
    "latexmlmath_oxide failed on {tex:?}: {}",
    String::from_utf8_lossy(&out.stderr)
  );
  String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn single_structure_formula_is_not_emptied() {
  for (tex, tag) in [
    (r"\frac{1}{2}", "<mfrac>"),
    (r"\sqrt{2}", "<msqrt>"),
    (r"x^2", "<msup>"),
  ] {
    let xml = math(tex);
    assert!(
      !xml.contains("<mrow/>"),
      "{tex}: emitted an EMPTY <mrow/> — the parsed tree was never attached:\n{xml}"
    );
    assert!(
      xml.contains(tag),
      "{tex}: expected {tag} in the output:\n{xml}"
    );
  }
}

#[test]
fn multi_part_formula_is_not_duplicated() {
  // `\frac{a}{b}+c` holds exactly one fraction; an unconditional append emitted
  // the whole expression twice (two <mfrac>, two <mo>+</mo>).
  let xml = math(r"\frac{a}{b}+c");
  assert_eq!(
    xml.matches("<mfrac>").count(),
    1,
    "expected exactly one <mfrac> — the parsed tree was attached twice:\n{xml}"
  );
  assert_eq!(
    xml.matches("<mo>+</mo>").count(),
    1,
    "expected exactly one <mo>+</mo>:\n{xml}"
  );
}
