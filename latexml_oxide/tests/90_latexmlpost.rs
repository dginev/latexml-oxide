//! Integration tests for the latexml_post processing pipeline.
//!
//! Port of LaTeXML/t/90_latexmlpost.t
//! For each test `$name` there should be `$name.xml` (input) and
//! `$name-post.xml` (expected output from `latexmlpost --keepXMath --pmml`).

use latexml_post::{
  Post,
  document::{PostDocument, PostDocumentOptions},
  mathml::MathML,
  processor::Processor,
};

const DIR: &str = "tests/post";

/// Normalize an XML file exactly the way `xmllint --format` does: parse and
/// re-serialize through libxml2's pretty-printer (the same C code path —
/// xmllint IS libxml2). Returns the formatted serialization split into lines.
///
/// Signal integrity: the predecessor of this helper piped through
/// `bash -c "diff <(xmllint --format …)"`, which vacuously PASSED with empty
/// output when xmllint was missing (this hid two stale goldens until the
/// macOS runner — which has xmllint — failed honestly, 2026-07-03), and
/// required a Unix userland (bash, xmllint, diff, grep, wc) that native
/// Windows lacks. In-process parsing fails toward flagging by construction:
/// a missing or malformed file panics instead of comparing empty-to-empty.
fn xmllint_format(path: &str) -> String {
  // no_blanks mirrors xmllint --format's xmlKeepBlanksDefault(0): blank
  // text nodes are dropped at parse time, which is what licenses libxml2's
  // pretty-printer to re-indent element-only content. Without it, files
  // whose stored indentation differs (compact vs indented) diff on pure
  // whitespace despite identical XML.
  let doc = libxml::parser::Parser::default()
    .parse_file_with_options(path, libxml::parser::ParserOptions {
      no_blanks: true,
      ..libxml::parser::ParserOptions::default()
    })
    .unwrap_or_else(|e| panic!("Failed to parse {path} for normalized comparison: {e:?}"));
  doc.to_string_with_options(libxml::tree::SaveOptions {
    format: true,
    ..libxml::tree::SaveOptions::default()
  })
}

/// Run a post-processing test: read input XML, apply PMML conversion with
/// keepXMath, compare against reference via libxml2-normalized line diff.
fn post_test(name: &str, max_allowed_diffs: usize) {
  let input_path = format!("{}/{}.xml", DIR, name);
  let reference_path = format!("{}/{}-post.xml", DIR, name);

  let input = std::fs::read_to_string(&input_path)
    .unwrap_or_else(|e| panic!("Failed to read {}: {}", input_path, e));

  let doc = PostDocument::new_from_string(&input, PostDocumentOptions::default())
    .unwrap_or_else(|e| panic!("Failed to parse {}: {}", input_path, e));

  let mut post = Post::new();
  let pmml = MathML::new_presentation().with_keep_xmath(true);
  let mut processors: Vec<Box<dyn Processor>> = vec![Box::new(pmml)];

  let results = post
    .process_chain(vec![doc], &mut processors)
    .expect("post-processing failed");

  assert_eq!(results.len(), 1, "Expected 1 output document");

  let result_doc = &results[0];
  let actual = result_doc.to_xml_string();

  // Save actual output for debugging
  let actual_path = format!("{}/{}-post-actual.xml", DIR, name);
  std::fs::write(&actual_path, &actual).ok();

  // Normalized comparison: libxml2 pretty-print both sides (the exact
  // `xmllint --format` normalization), then LCS line-diff. The count
  // matches the old `diff | grep '^[<>]' | wc -l` semantics: every
  // inserted or deleted line counts one (a changed line counts two).
  let actual_formatted = xmllint_format(&actual_path);
  let reference_formatted = xmllint_format(&reference_path);
  let diff = similar::TextDiff::from_lines(&actual_formatted, &reference_formatted);
  let diff_count = diff
    .iter_all_changes()
    .filter(|c| c.tag() != similar::ChangeTag::Equal)
    .count();

  if diff_count > max_allowed_diffs {
    // Show the leading changed lines for debugging (old behavior: head -40
    // of the raw diff).
    let detail: Vec<String> = diff
      .iter_all_changes()
      .filter(|c| c.tag() != similar::ChangeTag::Equal)
      .take(40)
      .map(|c| {
        let sigil = match c.tag() {
          similar::ChangeTag::Delete => '<',
          similar::ChangeTag::Insert => '>',
          similar::ChangeTag::Equal => unreachable!(),
        };
        format!("{} {}", sigil, c.value().trim_end_matches('\n'))
      })
      .collect();

    panic!(
      "Post-processing output for '{}' has {} diff lines (max allowed: {}).\n\n{}\n",
      name,
      diff_count,
      max_allowed_diffs,
      detail.join("\n")
    );
  } else {
    eprintln!(
      "{}: {} diff lines (max allowed: {})",
      name, diff_count, max_allowed_diffs
    );
  }
}

#[test]
fn simplemath_post_test() {
  // 4 diff lines: spacing adjustments for && (BINOP) and !! (POSTFIX)
  // where our spacing algorithm adds spacing that Perl's doesn't
  post_test("simplemath", 4);
}

#[test]
fn opdecoration_post_test() {
  // FUNCTION APPLICATION (⁡) over-insertion: an operator whose presentation is an
  // <m:mo> (∇, ∂, ∑, ∫, …) must juxtapose its argument (∇ϕ, ∂f, ∑a, ∫g) — NOT
  // emit ∇⁡ϕ — matching Perl's is_mo rule (MathML.pm Apply:?:?). Regression guard
  // for presentation.rs op_base_is_mo.
  post_test("opdecoration", 0);
}

#[test]
fn hyperref_post_test() { post_test("hyperref", 0); }

#[test]
fn mathprimed_post_test() {
  // MathML-post audit F17 — `pmml_scriptsize_padded` (`MathML.pm` L925-934),
  // "This is to handle primed sums, etc.", and the `emb_right` detection in
  // `pmml_script_decipher` (L1015-1017) that feeds it.
  //
  // In `\mathop{X\'}\limits_{p}^{q}` the prime is an embellishment of the BASE,
  // not a script of the outer construct: Perl stops its downward walk on a post
  // script found below a mid (under/over) script, keeps the embellished
  // `Apply(post-sup, X, ')` as the base, and widens each limit with an invisible
  // copy of the `'` so the limits centre over the `X` rather than over the whole
  // `X'` box. Rust instead treated the prime as an outer postscript, which
  // INVERTED the nesting — `msup` outside `munderover` instead of inside — and
  // left the limits uncentred with no phantom padding at all.
  //
  // The fixture's other two formulas (`\mathop{\sum\'}\limits_{i=1}^{n}` and
  // `{\sum}\'\limits_{k}`) are the negative cases: they were already
  // byte-identical to Perl and must gain no phantom.
  //
  // Deliberately NO trailing operand on any formula. A following factor makes
  // Rust insert a FUNCTION APPLICATION `<m:mo>⁡</m:mo>` that Perl omits for this
  // base shape — the separate over-insertion family that `opdecoration_post_test`
  // guards — which would add an unrelated diff line and mask this one.
  // Golden generated by same-host Perl LaTeXML 0.8.8
  // `latexmlpost --keepXMath --pmml` on the identical core XML.
  post_test("mathprimed", 0);
}

#[test]
fn scriptlevels_post_test() {
  // `pmml_script_decipher` (`MathML.pm` L1005-1020) starts a NEW script pair when
  // the nested script sits at a different `scriptpos` LEVEL — the digit in
  // `post0`/`post1` — and not only when the pair's slot is already taken. Rust
  // ignored the level entirely and merged on slot-freeness alone, so `{x_a}^b`
  // collapsed from Perl's two-pair `m:mmultiscripts` (the `b` riding to the right
  // of the whole `x_a` box) into an `m:msubsup` that stacks the two — a different
  // formula, silently.
  //
  // The fixture pins both orders (`{x_a}^b`, `{x^a}_b`), a three-level nest
  // (`{{x_a}^b}_c` → three pairs), and the negative case: a genuine tensor
  // `{}^1_2X^3_4`, where all four scripts DO belong to one pre pair and one post
  // pair and must not be split.
  //
  // It also pins the empty-slot spelling: an absent script is an empty
  // `<m:mrow/>`, as Perl emits. Rust briefly used `<m:none/>` — MathML Core
  // REMOVED that element, and an empty `m:mrow` is the accepted placeholder for
  // an omitted subtree.
  //
  // Golden generated by same-host Perl LaTeXML 0.8.8
  // `latexmlpost --keepXMath --pmml --noscan --nocrossref` on the identical core
  // XML — byte-identical, no adjustments.
  //
  // CAVEAT on the three-level case: this harness feeds PERL's core XML to Rust's
  // post stage, so it proves the post stage only. End to end, Rust's math parser
  // still fails on `{{x_a}^b}_c` and emits `class="ltx_math_unparsed"`; the
  // one- and two-level formulas here ARE byte-identical end to end. That parser
  // gap belongs to the deferred R8 family — see SYNC_STATUS.
  post_test("scriptlevels", 0);
}

#[test]
fn mathouter_post_test() {
  // MathML-post audit F17 — `outerWrapper` (`MathML.pm` L77-100) was dropping two
  // whole attribute families from `<m:math>`:
  //   * the **image fallback** (`altimg`, `altimg-width`, `altimg-height`,
  //     `altimg-valign`) that `--mathimages` exists to advertise, so a renderer
  //     without MathML support had nothing to fall back to. Note Perl NEGATES the
  //     depth ("Note the sign!"): `imagedepth="5"` becomes `-5px`.
  //   * the **RDFa** set (`about resource property rel rev typeof datatype
  //     content`), taken from the Math element or else the XMath — so a document
  //     using `lxRDFa` to annotate a formula lost the annotation at the MathML
  //     boundary.
  // The fixture sets these directly on `ltx:Math` rather than going through
  // `--mathimages`/`lxRDFa`, which is what makes it a test of `outerWrapper`
  // itself; it also pins the two negative cases — a formula with neither family
  // must gain nothing, and an image with no depth must OMIT `altimg-valign`
  // rather than emit a bare `-px`.
  // Golden generated by same-host Perl LaTeXML 0.8.8
  // `latexmlpost --keepXMath --pmml` on the identical core XML.
  post_test("mathouter", 0);
}

#[test]
fn mtextstyle_post_test() {
  // MathML-post audit F17 — `pmml_text_aux` threading Perl's `%attr`
  // (`MathML.pm` L1029-1045) so an `m:mtext` keeps the styling of the
  // `ltx:text` that wrapped it. Every arm of the function is exercised:
  // a whitespace run (` and ` → `\u{a0}and\u{a0}`, which used to be
  // `trim_start()`ed away), inherited `color` → `mathcolor`, `font` →
  // `ltx_mathvariant_*` / `ltx_font_*` class (never `mathvariant`, which Perl
  // clears for `m:mtext` at L756-757), `fontsize` → `mathsize`, a nested
  // `ltx:Math`, a framed `XMText`, and the raw-markup arm (`ltx:ref`).
  // Golden generated by same-host Perl LaTeXML 0.8.8
  // `latexmlpost --keepXMath --pmml` on the identical core XML.
  post_test("mtextstyle", 0);
}

#[test]
fn alignrows_operand_slot_keeps_relop_infix() {
  // Issue #312: an `align` continuation row (`& = RHS`, whose LHS is inherited
  // from the row above) parses as `Apply(=, absent, RHS)`. Presentation MathML
  // must keep an operand slot for that `absent`, because MathML infers an
  // `<mo>`'s form from its POSITION — first child of its `<mrow>` ⇒ prefix —
  // and the form selects the operator-dictionary spacing. Suppressing the slot
  // (Task #264) made `<mo>=</mo>` the first child, so every continuation row
  // lost its infix spacing and the `=` column stopped lining up.
  //
  // Asserted STRUCTURALLY rather than by diffing a Perl golden: our placeholder
  // is `<m:mphantom/>` where Perl's is `<m:mi/>` (see presentation.rs — an
  // empty `<m:mi>` is banned by a debug_assert in document.rs), and a diff budget
  // cannot separate "different placeholder" from "no placeholder" — omitting it
  // scores FEWER diff lines (5) than emitting ours (10), so any budget that
  // passes the correct output also passes the regression.
  let input = std::fs::read_to_string(format!("{DIR}/alignrows.xml")).expect("read alignrows.xml");
  let doc = PostDocument::new_from_string(&input, PostDocumentOptions::default())
    .expect("parse alignrows.xml");
  let mut post = Post::new();
  let mut processors: Vec<Box<dyn Processor>> =
    vec![Box::new(MathML::new_presentation().with_keep_xmath(true))];
  let results = post
    .process_chain(vec![doc], &mut processors)
    .expect("post-processing failed");
  let actual = results[0].to_xml_string();

  // The fixture has three continuation rows (`&=`, `&=`, `&\leq`), so three
  // relational operators must carry a preceding operand slot.
  let slots = actual.matches("<m:mphantom/><m:mo").count()
    + actual.matches("<m:mphantom></m:mphantom><m:mo").count();
  assert!(
    slots >= 3,
    "expected >=3 continuation-row operand slots before a relational <m:mo>,      found {slots} — the `absent` placeholder was dropped, so the relop renders      with PREFIX spacing (issue #312):\n{actual}"
  );

  // And no relational operator may open its own <m:mrow> (the prefix-form
  // signature the bug produced).
  for op in ["=", "\u{2264}"] {
    let bad = format!("<m:mrow><m:mo>{op}</m:mo>");
    assert!(
      !actual.contains(&bad),
      "a relational <m:mo>{op}</m:mo> is the FIRST child of its <m:mrow>, so \
       MathML infers prefix form and drops the infix spacing (issue #312)"
    );
  }
}

#[test]
fn mathgolden_post_test() {
  // The MathML-post audit golden set (PR_READINESS): mathstyle transitions
  // (\tfrac/\dfrac/\displaystyle), inherited context color on frac/cancel/
  // sqrt/tokens, menclose (\boxed), minsize/maxsize (\bigl), author spacing
  // (\, \! \qquad), Inner-Punct array-comma spacing, movablelimits (lim/sum),
  // cfrac nesting and nth-root order. Golden generated by REFERENCE-tree
  // Perl latexmlpost --keepXMath --pmml over the identical core XML.
  // ZERO diff lines: byte-identical to reference-tree Perl.
  post_test("mathgolden", 0);
}

/// The three Plane-1 remapping modes, each byte-checked against same-host Perl
/// LaTeXML 0.8.8 (`latexmlpost --pmml`, `--noplane1`, `--hackplane1`).
///
/// MathML-post audit F17 — `preprocess` (`MathML.pm` L66-74) never wired its
/// config through. `MathML::plane1` existed as a struct field but was **never
/// read**: the token path remapped unconditionally, so `--noplane1` could not
/// have worked even if the flag had existed, and `hackplane1` was absent
/// entirely. Perl `stylizeContent` L734-736 picks the variant to remap WITH:
///
/// ```text
/// my $u_variant = $variant
///   && ($plane1hack ? $plane1hackable{$variant}
///   : ($plane1 ? $variant : undef));
/// ```
///
/// Driven through the real `MathML` processor rather than a unit call, so the
/// `set_plane1` handoff in `convert_node` is exercised too — a builder that set
/// the field without the thread-local reaching the token path would pass a
/// narrower test.
#[test]
fn plane1_modes_match_perl() {
  let input = std::fs::read_to_string(format!("{DIR}/plane1.xml")).expect("read plane1.xml");

  // (plane1, hack_plane1) → the expected `<m:mi>` run, in document order.
  // `\mathcal{A} \mathfrak{B} \mathbb{C} \mathbf{D} \mathbf{\mathcal{E}}`.
  let cases: [(bool, bool, &[&str]); 3] = [
    // Default: everything remaps, the codepoint carries the style, no mathvariant.
    (true, false, &["𝒜", "𝔅", "ℂ", "𝐃", "ℰ"]),
    // --noplane1: ASCII kept, style carried by `mathvariant` instead.
    (false, false, &["A", "B", "C", "D", "E"]),
    // --hackplane1: only script/fraktur/double-struck remap — `bold` is absent
    // from Perl's `%plane1hackable`, so `\mathbf{D}` stays ASCII+mathvariant,
    // while `\mathbf{\mathcal{E}}` remaps as PLAIN script (the whole point: no
    // font has a bold-script block).
    (true, true, &["𝒜", "𝔅", "ℂ", "D", "ℰ"]),
  ];

  for (plane1, hack, expected) in cases {
    let doc = PostDocument::new_from_string(&input, PostDocumentOptions::default())
      .expect("parse plane1.xml");
    let mut post = Post::new();
    let mut processors: Vec<Box<dyn Processor>> = vec![Box::new(
      MathML::new_presentation().with_plane1(plane1, hack),
    )];
    let actual = post
      .process_chain(vec![doc], &mut processors)
      .expect("post-processing failed")[0]
      .to_xml_string();

    let mis: Vec<&str> = expected
      .iter()
      .copied()
      .filter(|t| {
        !actual.contains(&format!("<m:mi>{t}</m:mi>")) && !actual.contains(&format!(">{t}</m:mi>"))
      })
      .collect();
    assert!(
      mis.is_empty(),
      "plane1={plane1} hack={hack}: missing token(s) {mis:?}\n{actual}"
    );

    // The mathvariant attribute and the remapped codepoint are mutually
    // exclusive per token, so assert the negative side too — otherwise a build
    // that emitted BOTH would pass on the positive checks alone.
    let has_bold_variant = actual.contains(r#"mathvariant="bold""#);
    assert_eq!(
      has_bold_variant,
      !plane1 || hack,
      "plane1={plane1} hack={hack}: mathvariant=\"bold\" presence is wrong — it \
       must appear exactly when \\mathbf did NOT remap:\n{actual}"
    );
  }
}
