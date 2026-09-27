//! K13 stage 1 (KERNEL_CAPABILITIES): the chain walker and comparator of
//! `latexml::conformance`, on the kernel box family (the real macros from the LaTeX dump) and on
//! arydshln (the real `.sty`, read past its binding), with the two mutations the design names:
//! the pre-56jx `\hdashline` without its optional argument and the pre-56kb `\@makebox` that did
//! not start a paragraph.

use latexml::conformance::{Arg, Mismatch, View, compare, view_of};

use super::perfect_kernel_batch46::{convert_with_then, error_count, warning_count};

const BOX_FAMILY: [&str; 8] = [
  "\\makebox",
  "\\mbox",
  "\\framebox",
  "\\fbox",
  "\\raisebox",
  "\\parbox",
  "\\rule",
  "\\footnote",
];

/// The bindings' views, after `mutate`, then the real macros': the LaTeX dump loaded over the
/// session (raw latex.ltx, whose definitions the bindings had replaced; the session's State is
/// discarded afterwards).
fn kernel_views(
  names: &'static [&'static str],
  mutate: fn(),
) -> (Vec<Option<View>>, Vec<Option<View>>) {
  let tex = "\\documentclass{article}\n\\begin{document}\nx\n\\end{document}\n";
  let (log, _xml, views) = convert_with_then(tex, None, move |_| {
    mutate();
    let binding: Vec<Option<View>> = names.iter().map(|cs| view_of(cs)).collect();
    let dir = std::env::var("LATEXML_DUMP_DIR")
      .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/dumps").to_string());
    // The session's own year's dump (latex.rs prefers the ambient TeX Live year).
    let (path, _) = latexml_engine::dump_paths::resolve_versioned_in_dir(
      std::path::Path::new(&dir),
      "latex",
      latexml_engine::dump_paths::detect_ambient_texlive_year(),
    )
    .expect("a LaTeX dump (tools/make_formats.sh)");
    latexml_core::dump_reader::load_native_dump(&path).expect("the LaTeX dump loads");
    let raw: Vec<Option<View>> = names.iter().map(|cs| view_of(cs)).collect();
    (raw, binding)
  });
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  views
}

fn mismatches(raw: &[Option<View>], binding: &[Option<View>]) -> Vec<Vec<&'static str>> {
  raw
    .iter()
    .zip(binding)
    .map(|(r, b)| {
      let (r, b) = (
        r.as_ref().expect("raw view"),
        b.as_ref().expect("binding view"),
      );
      compare(r, b).iter().map(Mismatch::class).collect()
    })
    .collect()
}

fn opt(default: &str) -> Arg { Arg::Optional(Some(default.to_string())) }

/// latex.ltx's box commands as the walker reads them (latex.ltx:16077-16082, :16182-16195,
/// :16236-16242, :16359-16380), and every binding reading the same: `\parbox` alone is flagged,
/// PROLOGUE_UNKNOWN — `\@iiiparbox` starts with `\leavevmode`, while the binding's walk stops in
/// its wrapper macro's `\ifx.#2.\expandafter\@firstoftwo…` dispatch (a stage-1 walker limit).
/// The flag found KNOWN_PERL_ERRORS #309: the constructor `\lx@parbox` began
/// `inline_internal_vertical` without entering horizontal mode, as Perl's does
/// (latex_constructs.pool.ltxml:4763); since 57d it declares `enter_horizontal` (DIVERGENCES
/// #338), which the walk cannot reach, so the flag stays until the walker follows that dispatch.
#[test]
fn kernel_box_family_conforms_to_latex_ltx() {
  let (raw, binding) = kernel_views(&BOX_FAMILY, || {});
  let args = |view: &Option<View>| view.as_ref().expect("a view").args.clone();
  use Arg::{Mandatory as M, Optional, Peek};
  let picture = || Peek("(".to_string());
  assert_eq!(
    args(&raw[0]),
    vec![picture(), Optional(None), opt("c"), M],
    "\\makebox"
  );
  assert_eq!(args(&raw[1]), vec![M], "\\mbox");
  assert_eq!(
    args(&raw[2]),
    vec![picture(), Optional(None), opt("c"), M],
    "\\framebox"
  );
  assert_eq!(args(&raw[3]), vec![M], "\\fbox");
  assert_eq!(
    args(&raw[4]),
    vec![M, Optional(None), Optional(None), M],
    "\\raisebox"
  );
  assert_eq!(
    args(&raw[5]),
    vec![Optional(None), Optional(None), Optional(None), M, M],
    "\\parbox"
  );
  assert_eq!(args(&raw[6]), vec![opt("\\z@"), M, M], "\\rule");
  // `\footnote`'s argument is read by the last call of a body whose first call is not its last
  // (`\@xfootnote[#1]{\begingroup … \@footnotemark\@footnotetext}`, latex.ltx).
  assert_eq!(args(&raw[7]), vec![Optional(None), M], "\\footnote");
  assert!(
    raw[..7]
      .iter()
      .all(|view| view.as_ref().unwrap().prologue.enter_horizontal),
    "every box command starts a paragraph in latex.ltx"
  );
  let expected: Vec<Vec<&str>> = vec![
    vec![],
    vec![],
    vec![],
    vec![],
    vec![],
    vec!["PROLOGUE_UNKNOWN"],
    vec![],
    vec![],
  ];
  assert_eq!(mismatches(&raw, &binding), expected);
}

/// The pre-56kb `\@makebox` (restricted horizontal mode, no paragraph start) is reported: the
/// binding's view comes from the mode record the definers keep (`DeclaredMode`).
#[test]
fn makebox_without_entering_horizontal_is_reported() {
  const MAKEBOX: [&str; 1] = ["\\makebox"];
  let (raw, binding) = kernel_views(&MAKEBOX, || {
    use latexml_core::{common::store::Stored, state::lookup_meaning};
    let cs = latexml_core::T_CS!("\\@makebox");
    let Some(Stored::Constructor(constructor)) = lookup_meaning(&cs) else {
      panic!("\\@makebox is a constructor");
    };
    let mut mutated = (*constructor).clone();
    let mut record = mutated.declared_mode.expect("a mode record");
    record.enter_horizontal = false;
    mutated.declared_mode = Some(record);
    latexml_core::state::install_definition(mutated, None);
  });
  assert_eq!(mismatches(&raw, &binding), vec![vec![
    "PROLOGUE_ENTERH_MISSING"
  ]]);
}

const ARYDSHLN: [&str; 4] = [
  "\\hdashline",
  "\\cdashline",
  "\\firsthdashline",
  "\\lasthdashline",
];

/// `\lxAuditRawLoad{name}`: loads package `name` from its `.sty` even when a binding exists
/// (`require_package` with `noltxml`), so a session can hold the real macros.
fn install_raw_loader() {
  use std::rc::Rc;

  use latexml_core::{
    binding::{
      content::{RequireOptions, require_package},
      def::dialect::def_primitive,
    },
    common::def_parser::parse_parameters,
    definition::{PrimitiveBody, primitive::PrimitiveOptions},
  };
  let cs = latexml_core::T_CS!("\\lxAuditRawLoad");
  let params = parse_parameters("{}", &cs, true).expect("parameters");
  def_primitive(
    cs,
    params,
    Some(PrimitiveBody::Closure(Rc::new(|args| {
      let name = args[0].to_string();
      require_package(name.trim(), RequireOptions {
        noltxml: Some(true),
        ..Default::default()
      })?;
      Ok(Vec::new())
    }))),
    PrimitiveOptions::default(),
  )
  .expect("\\lxAuditRawLoad");
}

/// The views of the arydshln commands in a session that loads array and then arydshln (so that
/// the first/last rules exist, arydshln.sty:66-68), after `mutation` (TeX in the preamble).
/// `raw`: the real `.sty` is read (`\lxAuditRawLoad`), not the binding.
fn arydshln_views(raw: bool, mutation: &'static str) -> Vec<Option<View>> {
  use std::rc::Rc;

  use latexml::converter::Converter;
  use latexml_core::common::{Config, OutputFormat};
  let load = if raw {
    "\\makeatletter\\lxAuditRawLoad{arydshln}\\makeatother"
  } else {
    "\\usepackage{arydshln}"
  };
  let tex = format!(
    "\\documentclass{{article}}\n\\usepackage{{array}}\n{load}\n{mutation}\n\\begin{{document}}\nx\n\\end{{document}}\n"
  );
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let opts = Config {
        format: OutputFormat::XML,
        include_comments: Some(false),
        bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let mut converter = Converter::from_config(opts.clone());
      converter.prepare_session(&opts).expect("a session");
      install_raw_loader();
      let response = converter.convert_content_with_provenance("t.tex", tex);
      let views = ARYDSHLN.iter().map(|cs| view_of(cs)).collect();
      latexml_core::reset_thread_engine();
      assert_eq!(error_count(&response.log), 0, "{}", response.log);
      assert_eq!(warning_count(&response.log), 0, "{}", response.log);
      views
    })
    .expect("spawn")
    .join()
    .expect("arydshln views")
}

/// The real arydshln reads `\hdashline[dash/gap]` and `\cdashline{m-n}[dash/gap]` by a peek at
/// the end of a `\noalign` its brace trick leaves open, passing its continuation as an argument
/// (arydshln.sty:432-462); the binding (56jx) reads the same.
#[test]
fn arydshln_binding_conforms_to_the_sty() {
  let raw = arydshln_views(true, "");
  let binding = arydshln_views(false, "");
  let dash = opt("\\dashlinedash/\\dashlinegap");
  let args: Vec<Vec<Arg>> = raw
    .iter()
    .map(|v| v.as_ref().unwrap().args.clone())
    .collect();
  assert_eq!(args, vec![
    vec![dash.clone()],
    vec![Arg::Mandatory, dash.clone()],
    vec![dash.clone()],
    vec![dash],
  ]);
  assert!(
    raw[0]
      .as_ref()
      .unwrap()
      .chain
      .contains(&"\\adl@ihdashline".to_string()),
    "the raw view is arydshln's own: {:?}",
    raw[0]
  );
  // The binding side is the binding (`\\hdashline[]` → `\\hline`), not the real macros again.
  let binding_args: Vec<Vec<Arg>> = binding
    .iter()
    .map(|v| v.as_ref().unwrap().args.clone())
    .collect();
  let optional = || Arg::Optional(None);
  assert_eq!(binding_args, vec![
    vec![optional()],
    vec![Arg::Mandatory, optional()],
    vec![optional()],
    vec![optional()],
  ]);
  assert!(
    binding.iter().all(|v| !v
      .as_ref()
      .unwrap()
      .chain
      .iter()
      .any(|cs| cs.starts_with("\\adl@"))),
    "{binding:?}"
  );
  let none: Vec<Vec<&str>> = vec![vec![]; 4];
  assert_eq!(mismatches(&raw, &binding), none);
}

/// The binding before 56jx (`\let\hdashline\hline`) dropped the optional argument: reported.
#[test]
fn hdashline_without_its_optional_is_reported() {
  let raw = arydshln_views(true, "");
  let binding = arydshln_views(false, "\\let\\hdashline\\hline");
  assert_eq!(mismatches(&raw[..1], &binding[..1]), vec![vec![
    "OPT_MISSING"
  ]]);
}

/// A delimited parameter takes the given items up to its delimiter, and reads on into the document
/// when the delimiter is not among them; a call is a body's tail only when it consumes every item
/// after it and reads more; an assignment's operands are not calls. `\x` gives `\ya` its first
/// argument and the `.` ending it, so the second runs to the document's `\relax`; `\z` gives both;
/// `\w` gives no delimiter; `\q`'s delimited head consumes the body and reads `#3`; `\s` only
/// `\let`s.
#[test]
fn delimited_parameters_read_on_past_their_given_items() {
  let tex = "\\documentclass{article}\n\\def\\ya#1.#2\\relax{}\n\\def\\yb#1.#2\\relax#3{}\n\\def\\x{\\ya{ab}.}\n\\def\\z{\\ya{ab}.c\\relax}\n\\def\\w{\\ya{ab}}\n\\def\\q{\\yb a.b\\relax}\n\\def\\s{\\let\\sa\\ya}\n\\begin{document}\nx\n\\end{document}\n";
  let (log, _xml, views) = convert_with_then(tex, None, |_| {
    ["\\x", "\\z", "\\w", "\\q", "\\s"].map(|cs| view_of(cs).expect("defined").args)
  });
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  let delimited = |d: &str| Arg::Delimited(d.to_string());
  assert_eq!(views, [
    vec![delimited("\\relax")],
    vec![],
    vec![delimited("."), delimited("\\relax")],
    vec![Arg::Mandatory],
    vec![],
  ]);
}
