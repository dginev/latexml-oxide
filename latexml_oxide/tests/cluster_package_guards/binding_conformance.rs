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
/// INCOMPLETE and PROLOGUE_UNKNOWN — `\@iiiparbox` starts with `\leavevmode`, while the binding's
/// walk stops in its wrapper macro's `\ifx.#2.\expandafter\@firstoftwo…` dispatch (a walker limit:
/// `\expandafter` over a conditional is TeX's to resolve).
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
    // The wrapper's `\expandafter\@firstoftwo\else…` jumps a conditional: incomplete (K13 stage 2).
    vec!["INCOMPLETE", "PROLOGUE_UNKNOWN"],
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
      // The diagnostics reach the response's log only through the installed logger.
      let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
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
      // The binding announces its stub: `Warning:missing_file:arydshln.sty … only minimally
      // stubbed` (arydshln_sty.rs).
      assert_eq!(
        warning_count(&response.log),
        usize::from(!raw),
        "{}",
        response.log
      );
      assert_eq!(
        response.log.contains("Warning:missing_file:arydshln.sty"),
        !raw,
        "{}",
        response.log
      );
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

/// The views of `names` after a preamble `defs` (with `\makeatletter`), in one session that must
/// convert clean.
fn views_after(defs: &'static str, names: &'static [&'static str]) -> Vec<View> {
  let tex = format!(
    "\\documentclass{{article}}\n\\makeatletter\n{defs}\n\\makeatother\n\\begin{{document}}\nx\n\\end{{document}}\n"
  );
  let (log, _xml, views) = convert_with_then(&tex, None, move |_| {
    names
      .iter()
      .map(|cs| view_of(cs).expect(cs))
      .collect::<Vec<View>>()
  });
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  views
}

/// K13 stage 2: the walker follows TeX through what a body does before it reads (each probe the
/// 57h review's): a `\def` takes its control sequence, parameter text and body, no more (`\rvx`
/// reads through to `\rvz`); `\futurelet\cs A` runs A next; a macro `\let` in both branches to
/// targets that read differently is unknown; one the body defines with parameters reads (unknown
/// what); an error in a branch is a check, not a stub; ltcmd commands read their grabbers in both
/// forms (the expandable `l` by its helper's delimiter); a robust wrapper hands its caller's items
/// on; a token saved by `\aftergroup` makes the tail unknown; a branch's tail before `\fi` is the
/// conditional's; a `\csname`-named `\let` target is skipped, not called; a `\let` outside a
/// conditional replaces the earlier one; a body ending in a bare `\def` does not panic.
#[test]
fn the_walker_follows_tex_through_body_assignments() {
  const NAMES: &[&str] = &[
    "\\rvx",
    "\\rvfl",
    "\\rvlet",
    "\\rvbd",
    "\\rvcall",
    "\\rvexp",
    "\\rvdoc",
    "\\rvhand",
    "\\rvag",
    "\\rvfi",
    "\\rvletg",
    "\\rvrep",
    "\\rvenddef",
    "\\rvxl",
    "\\rvdispatch",
  ];
  let views = views_after(
    "\\def\\rvz#1{}\\def\\rvx{\\def\\rvy{a}\\rvz}\n\
     \\def\\rvflA#1{}\\def\\rvfl{\\futurelet\\rvt\\rvflA}\n\
     \\def\\rvone#1{}\\def\\rvtwo#1#2{}\\def\\rvlet{\\ifx ab\\let\\next\\rvone\\else\\let\\next\\rvtwo\\fi\\next}\n\
     \\def\\rvbd{\\def\\rvbd@##1{x##1}\\rvbd@}\n\
     \\def\\rvguard{\\ifx ab\\PackageError{p}{m}{h}\\fi}\\def\\rvread#1{}\\def\\rvcall{\\rvguard\\rvread}\n\
     \\NewExpandableDocumentCommand\\rvexp{s O{dflt} m}{}\n\
     \\NewDocumentCommand\\rvdoc{s O{dflt} m}{}\n\
     \\DeclareRobustCommand\\rvrob[2]{}\\def\\rvhand{\\rvrob{a}}\n\
     \\def\\rvag{\\begingroup\\aftergroup\\rvread\\endgroup}\n\
     \\def\\rvpeek{\\@ifnextchar[\\rvread\\rvread}\\def\\rvfi{\\ifx ab\\rvpeek\\fi}\n\
     \\def\\rvletg{\\expandafter\\let\\csname rvq\\endcsname\\@gobble}\n\
     \\def\\rvrep{\\let\\next\\rvtwo\\let\\next\\rvone\\next}\n\
     \\def\\rvenddef{\\long\\def}\n\
     \\usepackage{xparse}\\NewExpandableDocumentCommand\\rvxl{l m}{}\n\
     \\def\\rvdispatch{\\ifx\\rvq\\relax\\let\\next\\relax\\else\\let\\next\\rvpeek\\fi\\next}",
    NAMES,
  );
  let complete = |i: usize, args: Vec<Arg>| {
    assert_eq!(views[i].args, args, "{}: {:?}", NAMES[i], views[i]);
    assert!(views[i].complete, "{}: {:?}", NAMES[i], views[i]);
  };
  let unknown = |i: usize, note: &str| {
    assert!(!views[i].complete, "{}: {:?}", NAMES[i], views[i]);
    assert!(
      views[i].notes.iter().any(|n| n.starts_with(note)),
      "{}: {:?}",
      NAMES[i],
      views[i]
    );
  };
  complete(0, vec![Arg::Mandatory]);
  complete(1, vec![Arg::Mandatory]);
  unknown(
    2,
    "calls a macro its body `\\let`s to targets that read differently",
  );
  unknown(3, "calls a macro its body defines");
  complete(4, vec![Arg::Mandatory]);
  assert!(
    matches!(views[5].args.as_slice(), [
      Arg::Star,
      Arg::Optional(_),
      Arg::Mandatory
    ]),
    "{:?}",
    views[5]
  );
  complete(6, vec![Arg::Star, opt("dflt"), Arg::Mandatory]);
  complete(7, vec![Arg::Mandatory]);
  unknown(8, "saves a token for later");
  // A branch's tail before `\fi`: which branch runs is TeX's to decide.
  unknown(9, "ends in a conditional");
  // `\let\csname…\endcsname\@gobble` assigns; `\@gobble` is not called.
  complete(10, vec![]);
  // A `\let` outside any conditional replaces the earlier one.
  complete(11, vec![Arg::Mandatory]);
  // A body ending in a bare `\def` (expl3's `\cs_set_protected:Npn` shape) reads on.
  assert!(!views[12].complete, "{:?}", views[12]);
  complete(13, vec![Arg::Delimited("{".to_string()), Arg::Mandatory]);
  // `\let\next\relax` against `\let\next\rvpeek`: the dispatch reads `[…]{…}` or nothing.
  unknown(
    14,
    "calls a macro its body `\\let`s to targets that read differently",
  );
}

/// K13 stage 2: the walker on real packages (`binding_audit --views`), each session clean. A body's
/// tail reads on through a peek (color.sty's `\pagecolor` ends in `\color`, whose `\@ifnextchar[`
/// reads the model); ltcmd commands read their grabbers (tabularray's `\SetCell`, the expandable
/// `\DeclareTblrTemplate`); a command that raises an error where it is read is incomplete
/// (amsmath's `\intertext` outside an alignment); a call of a macro the body `\edef`s is unknown
/// (amsmath's `\genfrac` ends in the `\@tempb` it builds), and a robust wrapper hands its caller's
/// items on (`\dfrac` gives `\genfrac` four of them); an expansion past 20,000 tokens stops the
/// walk (tcolorbox's key handlers had reached gigabytes).
#[test]
fn the_walker_reads_what_real_bodies_hand_on() {
  use latexml::conformance::real_views;
  let views = |package: &str| {
    let (diagnostics, views) = real_views(package, "");
    assert_eq!(
      (diagnostics.errors, diagnostics.warnings),
      (0, 0),
      "{package}"
    );
    views
  };
  let view = |views: &[(String, View)], cs: &str| {
    views
      .iter()
      .find(|(name, _)| name == cs)
      .map(|(_, view)| view.clone())
      .unwrap_or_else(|| panic!("{cs}"))
  };
  let pagecolor = view(&views("color"), "\\pagecolor");
  assert_eq!(
    pagecolor.args,
    vec![Arg::Optional(None), Arg::Mandatory],
    "{pagecolor:?}"
  );
  assert!(pagecolor.complete);
  let tabularray = views("tabularray");
  let setcell = view(&tabularray, "\\SetCell");
  assert!(
    matches!(setcell.args.as_slice(), [Arg::Optional(_), Arg::Mandatory]),
    "{setcell:?}"
  );
  assert!(setcell.complete);
  let template = view(&tabularray, "\\DeclareTblrTemplate");
  assert_eq!(template.args, vec![Arg::Mandatory; 3], "{template:?}");
  let amsmath = views("amsmath");
  let intertext = view(&amsmath, "\\intertext");
  assert!(!intertext.complete, "{intertext:?}");
  assert!(
    intertext
      .notes
      .iter()
      .any(|n| n.starts_with("raises an error here")),
    "{intertext:?}"
  );
  let genfrac = view(&amsmath, "\\genfrac");
  assert_eq!(genfrac.args, vec![Arg::Mandatory; 4], "{genfrac:?}");
  assert!(!genfrac.complete);
  assert!(
    genfrac
      .notes
      .iter()
      .any(|n| n == "calls a macro its body defines"),
    "{genfrac:?}"
  );
  let dfrac = view(&amsmath, "\\dfrac");
  assert!(dfrac.chain.iter().any(|cs| cs == "\\genfrac "), "{dfrac:?}");
  assert!(dfrac.args.is_empty() && !dfrac.complete, "{dfrac:?}");
  let (diagnostics, tcolorbox) = real_views("tcolorbox", "");
  assert_eq!(diagnostics.errors, 0);
  assert!(
    tcolorbox.iter().any(|(_, view)| view
      .notes
      .iter()
      .any(|n| n.ends_with("expands past 20000 tokens"))),
    "the bound stops at least one tcolorbox walk"
  );
}

/// K13 stage 2: one package audited on both sides (`audit_package`, the driver's unit): arydshln's
/// public macros read from its `.sty` and from the binding, each of the four stage-1 commands
/// audited and conformant; no errors, and one warning — the binding announcing its stub
/// (`Warning:missing_file:arydshln.sty … only minimally stubbed`, arydshln_sty.rs).
#[test]
fn a_package_audit_reads_both_sides() {
  let audit = latexml::conformance::audit_package("arydshln", "\\usepackage{array}");
  assert_eq!(
    (audit.raw_errors, audit.binding_errors),
    (0, 0),
    "{audit:?}"
  );
  assert_eq!(
    (audit.raw_warnings, audit.binding_warnings),
    (0, 1),
    "{audit:?}"
  );
  for cs in ARYDSHLN {
    assert!(
      audit
        .verdicts
        .iter()
        .any(|(name, verdict)| name == cs && *verdict == "conformant"),
      "{cs}: {:?}",
      audit.verdicts
    );
  }
}
