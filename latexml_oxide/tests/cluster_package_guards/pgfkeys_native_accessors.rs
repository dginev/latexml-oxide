//! Native pgfkeys dispatch (`pgfkeys_code_tex.rs`): the raw
//! `pgfkeys.code.tex` loads whole; the leaf accessors (slice 0) and the
//! `\pgfkeys{}`/`\pgfkeysalso{}`/`\pgfqkeys{}{}` loop (slice 1) are native,
//! on the raw `\pgfk@<key>` storage and with the raw handlers. The differential harness converts each
//! fixture with the natives ON and OFF (`LATEXML_PGFKEYS_NATIVE=0`) and
//! requires byte-identical core XML, so any semantic drift is a diff.

fn both_ways(fixture: &str) -> String {
  let tex = std::fs::read_to_string(format!("tests/cluster_regressions/pgfkeys/{fixture}.tex"))
    .expect("fixture");
  unsafe { std::env::remove_var("LATEXML_PGFKEYS_NATIVE") };
  let (stderr_on, on) = super::convert(&tex, true);
  assert_eq!(super::error_count(&stderr_on), 0, "native ON: {stderr_on}");
  unsafe { std::env::set_var("LATEXML_PGFKEYS_NATIVE", "0") };
  let (stderr_off, off) = super::convert(&tex, true);
  unsafe { std::env::remove_var("LATEXML_PGFKEYS_NATIVE") };
  assert_eq!(
    super::error_count(&stderr_off),
    0,
    "native OFF: {stderr_off}"
  );
  assert_eq!(
    on, off,
    "native and raw accessors must give byte-identical XML for {fixture}"
  );
  on
}

/// `\pgfkeyssetvalue`/`\pgfkeysaddvalue`/`\pgfkeysgetvalue`/`\pgfkeysvalueof`/
/// `\pgfkeysifdefined`/`\pgfkeyslet`/`\pgfkeysifassignable`, including a
/// stored value carrying `#` parameter characters.
#[test]
fn accessors_match_the_raw_engine() {
  let xml = both_ways("accessors");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>A:(2cm-3cm). B:(2cm-3cm). C:yes. D:no. E:(2cm-3cm). F:yesG:noH:yes. I:relax. J:undefined. K:[(2cm-3cm)].L:(2cm-3cm). N:yesO:yes.</p>"##,
  );
}

/// `.code`, `.code 2 args`, `.style` with arguments, `.append style`.
#[test]
fn code_and_style_keys_match() {
  let xml = both_ways("code_style");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>[a:1][b:2;3][a:4][b:x;y] [a:5][b:x;y][a:z] [a:6][b:x;y][a:z]</p>"##,
  );
}

/// `.initial`, `.default`, `.get`, `.store in`, `.is choice`, `.is if`.
#[test]
fn initial_default_choice_keys_match() {
  let xml = both_ways("initial_default");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>A:7. B:9. C:8. D:10. [two] E:on.</p>"##,
  );
}

/// `/.cd` mid-list, relative keys, `\pgfqkeys`, `\pgfkeysalso`, the
/// `.unknown` handler with `\pgfkeyscurrentname`.
#[test]
fn paths_and_unknown_keys_match() {
  let xml = both_ways("cd_qkeys");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>[ax:1][bx:2] [ax:3][bx:4][ax:5] [bx:6] [unknown nokey:7]</p>"##,
  );
}

/// Slice 1's loop shapes: a re-entrant `\\pgfkeysalso` inside a `.code`,
/// a style, `/.cd` mid-list with the default path restored after the call,
/// `.value required` with a value, `.initial` used as the value, spaces and
/// nested braces around values, `.is family`, and — with first-char syntax
/// handlers on — an item with two leading spaces (a wrapper's `, #1`), which
/// `\\pgfkeys@syntax@handlers` skips whole (pgfkeys.code.tex:340) where the
/// plain path drops one (zx-calculus's `\\zxGenericMulti`, 242 errors).
#[test]
fn loop_shapes_match() {
  let xml = both_ways("loop_shapes");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>[b:1][a:in1][a:2] [a:3][a:4]A:/. [d:5] xB:x. [a:6][a:7][a:8] [ha:9] [b:0][a:in0][a:10][b:0][a:in0][a:11][a:12]</p>"##,
  );
}

/// The raw loop's territory, handed back untouched: `/handler config=only
/// existing`, `\\pgfkeysfiltered`, family activation, first-char syntax
/// handlers.
#[test]
fn raw_fallback_shapes_match() {
  let xml = both_ways("raw_fallbacks");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>[unknown nokey/.code][a:1] [c:2] [a:3][b:4] [a:5] [?:?x]</p>"##,
  );
}

/// A `.code` body runs IN the token stream (pgfkeys.code.tex:328), so it
/// may open an `\hbox` or a `\begin{center}` that later stream tokens
/// close — zx-calculus's `/tikz/on layer` shape
/// (tikzlibraryzx-calculus.code.tex:2415), which a nested digest of the
/// body broke with "Attempt to end mode restricted_horizontal" (29 errors on
/// one ZX circuit).
#[test]
fn code_bodies_run_in_the_stream() {
  let xml = both_ways("inline_code");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>[in]boxed[after]</p>"##);
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[r#"align="center""#],
    r##"<p align="center">centered text</p>"##,
  );
}

/// `\pgfkeysvalueof` keeps the raw `\csname…\endcsname` shape
/// (pgfkeys.code.tex:194): the stored macro is reached in TWO expansion
/// steps, which circuitikz's `\unexpandedvalueof` counts with a triple
/// `\expandafter` (circuitikz-1.7.2-body.tex:987-998). The one-step native
/// of batch 56dk handed the checker the value's first token (`[H]ELLO`).
#[test]
fn valueof_expansion_depth_matches() {
  let xml = both_ways("valueof_depth");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>[“pgfk@/my/k][ relax][HELLO]</p>"##);
}

/// The consumer itself: a circuitikz flipflop whose pin-presence tests run
/// `\unexpandedvalueof` inside `\ifx…\fi` blocks — circuitikzmanual.tex:7749
/// gained a stray `\fi` under the one-step native.
#[test]
fn circuitikz_flipflop_pins_match() {
  let xml = both_ways("circuitikz_flipflop");
  assert_eq!(xml.matches("<svg:path").count(), 11, "{xml}");
  // The pins the `\ifx…\fi` blocks place: `rst` from `td=rst`, `Q` from `t6=Q`.
  latexml::util::test::assert_element(
    &xml,
    "text",
    &[r#"fontsize="50%""#],
    r##"<text fontsize="50%">rst </text>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "text",
    &[r#"fontsize="90%""#],
    r##"<text fontsize="90%">B </text>"##,
  );
}

/// A handler that does not exist yet is run as `\relax`: every raw site
/// loads `\pgfkeys@code` through `\pgfkeysgetvalue` (pgfkeys.code.tex:175),
/// so `\pgfkeys@unknown` (:494) no-ops while `/handlers/.unknown` is absent
/// — a rawstyles pgf load reaches `\pgfkeys{/pgf/.is family}`
/// (pgfsys.code.tex:19) before the handlers exist (sweep #95: scsnowman-sample
/// 318 → 1001 errors, chuushaku 423 → 994, all `undefined:\pgfkeys@code`).
#[test]
fn missing_handler_runs_as_relax() {
  let xml = both_ways("missing_handler");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>[a:1][a:2][end]</p>"##);
}

/// A handler that scans forward in the stream sees the raw stream: the
/// native step is `\\pgfkeys@parse` itself and the remaining keys lie flat
/// behind it (pgfkeys.code.tex:359). robust-externalize's placeholder
/// re-scan captured a Rust-side continuation token instead and dispatched it
/// as the key `/robExt/\\lx@pgfkeys@continue` (sweep #95: 14 → 204 errors),
/// the remaining keys lost with it.
#[test]
fn forward_scanning_handlers_see_the_raw_stream() {
  let xml = both_ways("forward_scan");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>[grabbed: /t/a=3][a:2] [fi][a:4] [“fi “fi ][a:5][end]</p>"##,
  );
}

/// Under `first char syntax` every leading space of a key is skipped
/// (`\\pgfkeys@syntax@handlers`, pgfkeys.code.tex:342), as the native binding
/// (and Perl's, pgfkeys.code.tex.ltxml:216-241) always does: a `\\newtcolorbox`
/// body's `, #1` meeting an indented `[<newline> title=…]` (neoschool.tex:469,
/// which loads tikz's quotes library under LuaTeX). The default path trims
/// exactly one (`\\pgfkeys@spdef`), a delimiter space matching one space
/// token since batch 56gy; pdflatex matches the fixture byte for byte.
#[test]
fn leading_spaces_are_all_stripped() {
  let xml = both_ways("leading_spaces");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>[a:0][a:1][a:0][a:2][a:3][end]</p>"##);
}

/// The entry points are macros (pgfkeys.code.tex:320, :589, :607): a
/// group-closing `}` where their list should be is refused and left, as
/// for any macro argument — `{{\\tikzset}\\marg{options}}` (sa-tikz-doc.tex:336,
/// `\\tikzset` = `\\pgfqkeys{/tikz}`); a primitive's `{}` parameter consumed
/// it and the enclosing group never closed (sweep #95: 8 → 9 errors).
#[test]
fn closing_brace_is_not_a_list() {
  let xml = both_ways("brace_argument");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>XYZ[a:1][end]</p>"##);
}

/// Slice 2's native definition handlers store what the raw ones store
/// (pgfkeys.code.tex:648-652, :772, :826, :842, :852, :994): `.code` packs
/// `#1`/`##`, `/.@body` keeps the code for `.append style`, `.style` is
/// `.code=\\pgfkeysalso{…}`, `.initial`/`.default`, `.cd`; a redefined
/// `/handlers/.code` is honored (the raw definition is snapshotted at load).
#[test]
fn definition_handlers_store_the_raw_shape() {
  let xml = both_ways("definitions");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>[a:1]¡2¿[a:3][a:33] [a:4][a:44][a:z] A:i. B:d. D:d. [mine][b:5]</p>"##,
  );
}

/// `\\pgfkeys@split@path` (pgfkeys.code.tex:547-570) names the first segment
/// followed by an empty one and leaves a key's surplus text (`/p/q/` → `/`,
/// `/p//r` → `r//`) in the stream, typeset before the handler — pdflatex
/// prints it too; the native split had named the empty last segment. A
/// segment that is exactly a brace group loses its braces and, re-supplied
/// to the splitter, splits on the `/` it hid (`/a/{x/y}/c` → path `/a/x/y`).
#[test]
fn path_split_leaves_the_surplus_in_the_stream() {
  let xml = both_ways("path_split");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>/[U:/p—q] /[U:/p—q] /[U:/p—q] r//[U:—p] [U:/a/x/y—c] /[U:/a—x,y]</p>"##,
  );
}

/// A styled tikz node with a `.default` and a tcolorbox style: the real
/// consumers of the key tree.
#[test]
fn tikz_and_tcolorbox_styles_match() {
  let xml = both_ways("tikz_tcb");
  assert!(xml.matches("<svg:g").count() > 0, "{xml}");
}
