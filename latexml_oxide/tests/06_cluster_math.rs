//! Cluster regressions in the math parser and math-mode digestion.
//!
//! Split out of `06_cluster_regressions`; shares its helpers via
//! [`mod cluster`](cluster).

mod cluster;
use cluster::convert_to_xml;

/// The broad `^S\d+` prune sweep (`Document::prune_dangling_split_xmrefs`)
/// must NOT drop a `\Pr` (`\lx@dual` content-arm) ARGUMENT ref for
/// section-numbered aligned equations — that emitted a malformed
/// `apply(probability)` with no operand (silent content-MathML corruption).
/// The operand-protection guard keeps the ref (dangling rather than dropped,
/// closer to Perl which resolves it). See
/// docs/parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md (2026-06-26m/o).
/// A comma-list LEFT of a conditional bar parses with `|` binding to the LAST
/// item (Perl): `a,b|c` → `list@(a, conditional@(b, c))`, `a,b,c|d` →
/// `list@(a, b, conditional@(c, d))`, `x|y,z` → `conditional@(x, list@(y, z))`.
/// Previously `a,b|c` was UNPARSED — the root of the Class-B dangling-XMRef
/// witness (aligned `\Pr(s_A,s_B|\Omega)` arg failed to parse). The grammar rule
/// `statements punct statement vertbar statements => vertbar_modifier_listlhs`
/// fixes it; this asserts the exact Perl-matching tree shapes.
#[test]
fn cluster_comma_list_conditional() {
  let xml = convert_to_xml("tests/cluster_regressions/comma_list_conditional.tex");
  for expected in [
    "list@(a, conditional@(b, c))",
    "list@(a, b, conditional@(c, d))",
    "conditional@(x, list@(y, z))",
  ] {
    assert!(
      xml.contains(expected),
      "expected math text {expected:?} not found (comma-list conditional regressed)"
    );
  }
}
/// A `\quad`-separated formulae sequence whose first item is a
/// comma-list-left-of-relation (built by `distribute_list_relation`, which makes
/// a dual with a relation-`Apply` presentation, not an `XMWrap`) must NOT strand a
/// keyless bare `<XMRef/>` when a further `\quad` formula extends it. This was the
/// dominant `expected:id` "Missing idref" cluster (~370 papers). The Wrap-
/// presentation guard on the formulae/list extend paths fixes it. See
/// docs/parity/diagnostics/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md (2026-06-26v).
#[test]
fn cluster_formulae_distribute_no_bare_ref() {
  let xml = convert_to_xml("tests/cluster_regressions/formulae_distribute_no_bare_ref.tex");
  // A bare `<XMRef/>` (no idref) is the "Missing idref" symptom.
  let collapsed: String = xml.split_whitespace().collect::<Vec<_>>().join("");
  assert!(
    !collapsed.contains("<XMRef/>"),
    "keyless bare <XMRef/> present — distribute/formulae extend stranded a ref"
  );
}
/// A bare bigop as a `/`-fraction numerator (`\partial/\partial t`, Leibniz
/// partial-derivative notation) must PARSE — previously `ltx_math_unparsed`
/// (Rust-only; Perl: `partial-differential / partial-differential@(t)`). The
/// divide-scoped grammar rule `any_bigop divide term` fixes it without disturbing
/// the apply case (`\partial t`) or `\partial \times B`. See SYNC_STATUS.
#[test]
fn cluster_partial_over_partial() {
  let xml = convert_to_xml("tests/cluster_regressions/partial_over_partial.tex");
  // The \partial/\partial t formula must parse (no unparsed marker) and match Perl.
  assert!(
    !xml.contains("ltx_math_unparsed"),
    "\\partial/\\partial t left unparsed (bare-bigop fraction regressed)"
  );
  assert!(
    xml.contains("partial-differential / partial-differential"),
    "expected Perl-matching content text for \\partial/\\partial t not found"
  );
}
#[test]
fn cluster_xmref_pr_arg_not_dropped() {
  let xml = convert_to_xml("tests/cluster_regressions/xmref_pr_arg_not_dropped.tex");
  assert!(
    xml.contains(r#"meaning="probability""#),
    "probability operator missing from output"
  );
  // The probability XMApp must retain an operand: a bare
  // `<XMTok meaning="probability"/>` immediately followed by `</XMApp>`
  // (whitespace-insensitive) is the malformed/corrupted form we guard against.
  let collapsed: String = xml.split_whitespace().collect::<Vec<_>>().join("");
  assert!(
    !collapsed.contains(r#"meaning="probability"/></XMApp>"#),
    "malformed apply(probability) with no operand — content-arm arg ref was dropped"
  );
}
/// A leading relop (implied `absent` left operand) followed by a comma had no
/// derivation at all: `list_apply`'s fragment guard rejected any item carrying
/// an `absent` relop operand, and `formula relop formula_list` is deliberately
/// gone (KNOWN_PERL_ERRORS #37), so `$>50,000$` fell out as `ltx_math_unparsed`
/// while every neighbouring shape parsed. The guard now only rejects a comma
/// pair when BOTH items are fragments — matching the relaxation `formulae_apply`
/// already carried — and stays strict for `\quad`, where a run of align
/// fragments really is one broken-up equation (`tests/math/sampler`).
/// Witness: arXiv 2605.17646.
#[test]
fn cluster_leading_relop_comma_list() {
  let x = convert_to_xml("tests/cluster_regressions/leading_relop_comma_list.tex");
  assert!(
    !x.contains("ltx_math_unparsed"),
    "every formula here must parse; got an unparsed one:\n{x}"
  );
  // `50,000` is now recognized as ONE number by the thousands-separator rewrite
  // (see `cluster_thousands_separator_us_default`), so these two read as plain
  // relations rather than lists. The grammar fix is still what this test guards:
  // without it the LEADING-relop form has no derivation at all, which `$>a,b$`
  // below still exercises with a non-numeric list.
  assert!(
    x.contains(r#"text="absent &gt; 50000""#),
    "expected `absent > 50000` for the leading relop:\n{x}"
  );
  assert!(
    x.contains(r#"text="a &gt; 50000""#),
    "the binary-relop sibling must agree:\n{x}"
  );
  assert!(
    x.contains(r#"text="list@(absent &gt; a, b)""#),
    "the leading-relop comma list (non-numeric, so untouched by the thousands \
     rewrite) is the shape that had NO derivation before:\n{x}"
  );
}
/// Thousands separator, US default (owner policy 2026-07-25). `$50,000$` is ONE
/// number; the number ligature can never see that for English (its thousands arm
/// demands `role != PUNCT` and a math comma is always PUNCT) and cannot be fixed
/// there, since ligatures run per-token during building with no right context —
/// a merge-at-three-digits rule turns `$(1, 2024)$` into `12024`. The merge runs
/// in the post-build Rewriting phase instead, where each digit run is already
/// one token, which is what makes the "must NOT merge" half safe.
/// Witness: arXiv 2605.17646.
#[test]
fn cluster_thousands_separator_us_default() {
  let x = convert_to_xml("tests/cluster_regressions/thousands_separator.tex");
  let text_of = |tex: &str| -> String {
    let needle = format!(r#"tex="{tex}""#);
    let i = x
      .find(&needle)
      .unwrap_or_else(|| panic!("no formula {tex} in:\n{x}"));
    let rest = &x[i..];
    let t = rest.find(r#"text=""#).expect("text attr");
    let start = i + t + 6;
    x[start..start + x[start..].find('"').expect("close quote")].to_string()
  };
  // Merged into a single number, separators kept in the text but not the meaning.
  for (tex, want) in [
    ("50,000", "50000"),
    ("&gt;50,000", "absent &gt; 50000"),
    ("1,234,567", "1234567"),
    ("12,345,678,901", "12345678901"), // 4 groups — exercises every pass
    ("1,234.56", "1234.56"),           // thousands AND decimal
    ("3.14", "3.14"),
  ] {
    assert_eq!(text_of(tex), want, "{tex} should merge");
  }
  // Must NOT merge. `3,14` is the European decimal reading (two digits, so the
  // US rule declines); the rest would each be a real corruption.
  for (tex, want) in [
    ("3,14", "list@(3, 14)"),
    ("50,0001", "list@(50, 0001)"),          // 4-digit group
    ("f(x,000)", "f@(vector@(x, 000))"),     // no NUMBER left of the comma
    ("(1,2024)", "open-interval@(1, 2024)"), // the pair the ligature corrupted
    ("(12,3456)", "open-interval@(12, 3456)"),
    ("a,b", "list@(a, b)"),
  ] {
    assert_eq!(text_of(tex), want, "{tex} must stay unmerged");
  }
  // The merged token must be indistinguishable from an unmerged one: no
  // `font="italic"` stamped on by an ambient-font fallback.
  assert!(
    !x.contains(r#"<XMTok font="italic" meaning="50000""#),
    "merged number picked up an ambient italic font:\n{x}"
  );
}
/// The European half: for `de` the comma is the DECIMAL separator and the dot
/// the thousands one, handled by the language maps + the ligature's decimal arm.
/// Pins that the US-default rewrite leaves it alone.
#[test]
fn cluster_thousands_separator_eu() {
  let x = convert_to_xml("tests/cluster_regressions/thousands_separator_eu.tex");
  for want in [
    r#"tex="3,14" text="3.14""#,
    r#"tex="50.000" text="50000""#,
    r#"tex="1.234,56" text="1234.56""#,
  ] {
    assert!(x.contains(want), "missing {want} in:\n{x}");
  }
}
/// A bare operator used as an OPERAND — the argument-slot `f(\cdot)`, the inner
/// product `\langle\cdot,\cdot\rangle`, and operators NAMED rather than applied
/// (`(+)`, `(=)`, `(\times)`). The grammar admitted fenced singleton
/// bigops/OPERATORs but not the ADDOP/MULOP/BINOP/RELOP roles, so all of these
/// died as `ltx_math_unparsed`: measured against same-host Perl 0.8.8, Perl
/// parsed 7 of these 8 shapes and we parsed 0. `placeholder` /
/// `placeholder_list` admit them only where FENCED, so a stray `a + \times b`
/// still fails. Cases H/I cover the companion fix: a comma list mixing ONE
/// relation with a plain term, the `modified_term punct expression` variant the
/// grammar had deferred "until a witness shows them needed".
/// Witness: arXiv 2605.17646.
#[test]
fn cluster_fenced_bare_operator() {
  let x = convert_to_xml("tests/cluster_regressions/fenced_bare_operator.tex");
  for want in [
    r#"text="f@(cdot)""#,
    r#"text="g@(vector@(cdot, cdot))""#,
    r#"text="f@(vector@(cdot, x))""#,
    r#"text="delimited-⟨⟩@(list@(cdot, cdot))""#,
    // The mixed relation/plain comma list, inside a conditional and bare.
    r#"text="P@(conditional@(x, open-interval@(y &gt;= 0, z)))""#,
    r#"text="f@(vector@(a &gt;= 0, b))""#,
  ] {
    assert!(x.contains(want), "missing {want} in:\n{x}");
  }
  // `\|\cdot\|` is unparsed in Perl too — parity, deliberately still unparsed.
  // So exactly ONE formula here may carry the class, and it must be that one.
  assert_eq!(
    x.matches("ltx_math_unparsed").count(),
    0,
    "no formula in this fixture should be unparsed:\n{x}"
  );
}
/// Upstream LaTeXML #2837: `\hdotsfor[]{N}` spans N alignment columns (the
/// dots row gets N cells, `\hdots & … & \hdots`), instead of piling N
/// `\hdots` into one cell. 3+3+3 cells in the first matrix + 2+2 in the
/// second = 13 mtds, 5 of them dots. The optional spacing arg is consumed
/// and ignored, matching upstream.
#[test]
fn cluster_hdotsfor_columns() {
  let x = convert_to_xml("tests/cluster_regressions/hdotsfor.tex");
  // The harness returns the pre-XSLT XML, so count XMath cells.
  let cells = x.matches("<XMCell").count() + x.matches("<mtd").count();
  assert_eq!(
    cells, 13,
    "\\hdotsfor must span its column count (9 + 4 cells), got:\n{x}"
  );
  assert_eq!(
    x.matches('\u{2026}').count(),
    5,
    "expected 3 + 2 dots cells, got:\n{x}"
  );
}
/// amsmath's `\def\ext@arrow#1#2#3#4#5#6#7` (and `\arrowfill@#1#2#3#4`) take
/// plain TeX undelimited arguments — a single token OR a balanced group. Our
/// binding spelled the first four `Token`, which reads only the opening `{` of
/// a braced argument and spills the rest, including its `}`, back into the
/// stream; the stray `}` then closes the enclosing display math and everything
/// after it is swallowed into the leaked `<ltx:XMath>`. extpfeil.sty's
/// `\newextarrow{\xtwoheadleftarrow}{500{40}}{…}` is exactly that shape — an
/// `\mkern` amount of 40 has to be braced. Witness arXiv 2606.01903 (Perl,
/// which defines no `\ext@arrow` at all, reports 258 errors; we ran into the
/// 1000-error cap). Red without the fix: `Error:unexpected:} Attempt to close a
/// group that switched to mode display_math`, and the `\simeq` above-label is
/// replaced by a `0` SUBscript scavenged from the leaked `{40}`.
#[test]
fn cluster_ext_arrow_braced_mkern() {
  let x = convert_to_xml("tests/cluster_regressions/ext_arrow_braced_mkern.tex");
  assert!(
    x.contains(r#"<XMApp role="POSTSUPERSCRIPT""#) && x.contains(r#"name="simeq""#),
    "the \\ext@arrow above-label must survive as a superscript on the arrow:\n{x}"
  );
  assert!(
    !x.contains(r#"<XMApp role="POSTSUBSCRIPT""#),
    "a POSTSUBSCRIPT here means the braced `{{40}}` mkern amount leaked into the math:\n{x}"
  );
  assert!(
    x.contains("<p>Text after the display must survive.</p>"),
    "text after the display must stay OUTSIDE the math, in its own paragraph:\n{x}"
  );
}

/// A chain of POST scripts on one base must fold to arbitrary depth, and must
/// not care whether two adjacent scripts are the same KIND.
///
/// Perl's `addScripts` (`MathGrammar` L419-423) recurses on
/// `POSTSUPERSCRIPT`/`POSTSUBSCRIPT` with no depth bound and no alternation
/// requirement. Rust had hand-unrolled it to exactly two, alternating
/// (`scripted_factor_r2 = r12 postsuperarg | r11 postsubarg`), so **four whole
/// shapes fell out of the grammar and rendered `ltx_math_unparsed`**: the
/// same-kind repeats `{x^a}^b` and `{x_a}_b`, and every chain of three or more
/// such as `{{x_a}^b}_c`. Braces are what make these reachable — bare `x^a^b` is
/// rejected by TeX as "Double superscript" and never reaches the parser, which
/// is why the depth cap looked harmless.
///
/// Ground truth: same-host Perl LaTeXML 0.8.8 parses all six, and after the fix
/// the `XMath` trees and the resulting Presentation MathML are byte-identical to
/// Perl's for every one of them.
#[test]
fn cluster_script_chain_depth() {
  let x = convert_to_xml("tests/cluster_regressions/script_chain_depth.tex");
  assert!(
    !x.contains("ltx_math_unparsed"),
    "a braced script chain failed to parse:\n{x}"
  );
  // An unfolded chain leaves the pre-parse POSTSUBSCRIPT/POSTSUPERSCRIPT roles
  // sitting as siblings of the base; a folded one has SUBSCRIPTOP/SUPERSCRIPTOP
  // operator tokens instead.
  assert!(
    !x.contains(r#"role="POSTSUPERSCRIPT""#) && !x.contains(r#"role="POSTSUBSCRIPT""#),
    "post-script markers survived unfolded, so the chain rule did not apply:\n{x}"
  );
  // Depth is carried by `scriptpos`, numbered from the OUTERMOST script inward,
  // so the four-deep formula must reach `post4`.
  for level in ["post1", "post2", "post3", "post4"] {
    assert!(
      x.contains(&format!(r#"scriptpos="{level}""#)),
      "no script nested at {level}; the chain folded shallower than the source:\n{x}"
    );
  }
}

/// #703: nested `\sbox0{$#1$}\box0` boxes inside display math must not free a
/// libxml node twice. `\rulebox{\rulebox{foof}}` queues two nested subtrees for
/// deferred discard (B ⊂ A); `drain_pending_discards` freed A's subtree — B's C
/// node with it — then dropped B's unregistered handle, whose `_Node::drop`
/// dereferenced the freed `node->doc` (heap-use-after-free; Darwin's allocator
/// aborts, glibc tolerates). Verified red→green under AddressSanitizer. This
/// guard asserts the whole nested structure survives the drain.
#[test]
fn cluster_nested_sbox_discard_no_double_free() {
  let x = convert_to_xml("tests/cluster_regressions/mathparse_nested_sbox_discard.tex");
  // Innermost `$foof$` box, and the middle box wrapping it, both convert —
  // proving no subtree was freed out from under the parse.
  assert!(
    x.contains(r#"tex="foof""#),
    "innermost \\sbox math missing — nested discard corrupted the tree:\n{x}"
  );
  assert!(
    x.contains(r#"tex="\hbox{$foof$}\hrule barf""#),
    "middle \\sbox box missing — nested discard corrupted the tree:\n{x}"
  );
  // The trailing `barf` text after each `\box0\hrule` must still be present.
  assert!(x.contains("barf"), "trailing text lost:\n{x}");
}
