//! One package's audit (K13 stage 2): every public macro its real `.sty` defines, read in a
//! session holding the real file and in one holding the binding, and compared.
//!
//! The real side loads the package past its binding (`require_package` with `noltxml`, the
//! `\lxAuditRawLoad` primitive); a snapshot diff around that load is exactly what the file (and the
//! raw files it pulled in) installed. The binding side is an ordinary `\usepackage`. A binding that
//! reads the real file for a macro (its definition's origin is a raw file) conforms by
//! construction and is counted, not compared.

use std::{cell::RefCell, rc::Rc};

use latexml_core::{
  T_CS,
  binding::{
    content::{RequireOptions, require_package},
    def::dialect::def_primitive,
  },
  common::{Config, OutputFormat, def_parser::parse_parameters, store::Stored},
  definition::{PrimitiveBody, origin::DefinitionOrigin, primitive::PrimitiveOptions},
  state::{self, TableName, lookup_meaning},
};

use super::{Mismatch, View, compare, view_of};
use crate::converter::Converter;

/// A public control sequence whose binding differs from the real macro.
#[derive(Clone, Debug)]
pub struct Finding {
  /// `\makebox`.
  pub cs:         String,
  /// The real macro's view.
  pub raw:        View,
  /// The binding's view; `None` when the binding session leaves the name undefined.
  pub binding:    Option<View>,
  /// The differences ([`compare`]); empty for an undefined name.
  pub mismatches: Vec<Mismatch>,
}

/// The audit of one package.
#[derive(Clone, Debug, Default)]
pub struct Audit {
  pub package:          String,
  /// Public macros the real file defines.
  pub defined:          usize,
  /// Of those, the binding reads from a raw file (conformant by construction).
  pub passthrough:      usize,
  /// Of the rest, the binding reads as the real macro does.
  pub conformant:       usize,
  pub findings:         Vec<Finding>,
  /// Each public macro's verdict, in order: `passthrough`, `conformant` or `finding`.
  pub verdicts:         Vec<(String, &'static str)>,
  /// Error-level diagnostics of the real and the binding session (a failed load audits nothing).
  pub raw_errors:       usize,
  pub binding_errors:   usize,
  /// Warnings of the two sessions.
  pub raw_warnings:     usize,
  pub binding_warnings: usize,
}

/// A session's diagnostics: (errors and fatals, warnings).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Diagnostics {
  pub errors:   usize,
  pub warnings: usize,
}

impl Diagnostics {
  fn of(log: &str) -> Self {
    // Inline `Warning:<class>:` markers, as `error_count` counts errors.
    let warnings = log
      .match_indices("Warning:")
      .filter(|(i, _)| {
        let tail = &log.as_bytes()[*i + 8..];
        let n = tail
          .iter()
          .take_while(|b| b.is_ascii_alphabetic() || **b == b'_')
          .count();
        n > 0 && tail.get(n) == Some(&b':')
      })
      .count();
    Diagnostics {
      errors: crate::util::test::error_count(log) + log.matches("Fatal:").count(),
      warnings,
    }
  }
}

/// A public name: letters only, as a document writes it (no `@`, no expl3 `_`/`:`), and not the
/// `\endX` closer of an environment the file also defines (environments are stage 3's).
fn is_public(name: &str, all: &[String]) -> bool {
  let Some(letters) = name.strip_prefix('\\') else {
    return false;
  };
  !letters.is_empty()
    && letters.chars().all(|c| c.is_ascii_alphabetic())
    && !letters
      .strip_prefix("end")
      .is_some_and(|env| !env.is_empty() && all.iter().any(|n| n.strip_prefix('\\') == Some(env)))
}

fn origin_of(cs: &str) -> Option<DefinitionOrigin> {
  lookup_meaning(&T_CS!(cs))?
    .to_definition()
    .map(|d| d.get_origin())
}

thread_local! {
  /// The macros the last `\lxAuditRawLoad` installed.
  static INSTALLED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// `\lxAuditRawLoad{name}`: loads package `name` from its `.sty` even when a binding exists, and
/// records the macros the load installed.
fn install_raw_loader() {
  let cs = T_CS!("\\lxAuditRawLoad");
  let params = parse_parameters("{}", &cs, true).expect("parameters");
  def_primitive(
    cs,
    params,
    Some(PrimitiveBody::Closure(Rc::new(|args| {
      let name = args[0].to_string();
      let before = state::take_snapshot();
      require_package(name.trim(), RequireOptions {
        noltxml: Some(true),
        ..Default::default()
      })?;
      let installed = state::diff_snapshot(&before)
        .into_iter()
        .filter(|(table, _, value)| {
          *table == TableName::Meaning && matches!(value, Stored::Expandable(_))
        })
        .map(|(_, key, _)| latexml_core::common::arena::to_string(key))
        .collect();
      INSTALLED.with(|cell| *cell.borrow_mut() = installed);
      Ok(Vec::new())
    }))),
    PrimitiveOptions::default(),
  )
  .expect("\\lxAuditRawLoad");
}

/// One session over `tex` on its own thread (a conversion's State is thread-local), then `inspect`
/// while that State is live.
fn session<R: Send + 'static>(
  tex: String,
  raw_loader: bool,
  inspect: impl FnOnce() -> R + Send + 'static,
) -> (Diagnostics, R) {
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
      converter.note_main_source(&format!("literal:{tex}"));
      if converter.prepare_session(&opts).is_err() {
        let inspected = inspect();
        latexml_core::reset_thread_engine();
        return (Diagnostics { errors: 1, warnings: 0 }, inspected);
      }
      if raw_loader {
        install_raw_loader();
      }
      let response = converter.convert_content_with_provenance("audit.tex", tex);
      let inspected = inspect();
      latexml_core::reset_thread_engine();
      (Diagnostics::of(&response.log), inspected)
    })
    .expect("spawn an audit session")
    .join()
    .expect("an audit session")
}

/// The real side: every public macro package `name`'s own `.sty` defines, read past its binding
/// after `preamble`, with its view; and the session's error count.
pub fn real_views(name: &str, preamble: &str) -> (Diagnostics, Vec<(String, View)>) {
  let raw_tex = format!(
    "\\documentclass{{article}}\n{preamble}\n\\makeatletter\\lxAuditRawLoad{{{name}}}\\makeatother\n\\begin{{document}}\nx\n\\end{{document}}\n"
  );
  session(raw_tex, true, || {
    let installed = INSTALLED.with(|cell| cell.take());
    let mut public: Vec<String> = installed
      .iter()
      .filter(|cs| is_public(cs, &installed))
      .cloned()
      .collect();
    public.sort();
    public
      .into_iter()
      .filter_map(|cs| view_of(&cs).map(|view| (cs, view)))
      .collect::<Vec<(String, View)>>()
  })
}

/// Audit package `name` (`arydshln`): after `preamble` (packages it needs loaded first).
pub fn audit_package(name: &str, preamble: &str) -> Audit {
  let (raw_diagnostics, raw) = real_views(name, preamble);
  let names: Vec<String> = raw.iter().map(|(cs, _)| cs.clone()).collect();
  let binding_tex = format!(
    "\\documentclass{{article}}\n{preamble}\n\\usepackage{{{name}}}\n\\begin{{document}}\nx\n\\end{{document}}\n"
  );
  // A binding that reads the package's own file for a macro conforms by construction. Only a raw
  // file the document's world loaded counts (`File`): a `Format` definition is the kernel's, which
  // a package redefining it and a binding that does not would otherwise pass as conformant.
  let (binding_diagnostics, binding) = session(binding_tex, false, move || {
    names
      .iter()
      .map(|cs| match origin_of(cs) {
        Some(DefinitionOrigin::File) => (true, None),
        _ => (false, view_of(cs)),
      })
      .collect::<Vec<(bool, Option<View>)>>()
  });
  let mut audit = Audit {
    package: name.to_string(),
    defined: raw.len(),
    raw_errors: raw_diagnostics.errors,
    binding_errors: binding_diagnostics.errors,
    raw_warnings: raw_diagnostics.warnings,
    binding_warnings: binding_diagnostics.warnings,
    ..Audit::default()
  };
  for ((cs, raw), (passthrough, binding)) in raw.into_iter().zip(binding) {
    if passthrough {
      audit.passthrough += 1;
      audit.verdicts.push((cs, "passthrough"));
      continue;
    }
    let mismatches = binding
      .as_ref()
      .map(|b| compare(&raw, b))
      .unwrap_or_default();
    if binding.is_some() && mismatches.is_empty() {
      audit.conformant += 1;
      audit.verdicts.push((cs, "conformant"));
    } else {
      audit.verdicts.push((cs.clone(), "finding"));
      audit
        .findings
        .push(Finding { cs, raw, binding, mismatches });
    }
  }
  audit
}
