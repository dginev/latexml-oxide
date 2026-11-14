//! K13 stage 2 (KERNEL_CAPABILITIES): the binding-conformance audit of one package.
//!
//! `binding_audit --list` prints the compiled package bindings, one name per line;
//! `binding_audit --views <package> [<preamble>]` the real side's view of each public macro.
//! `binding_audit <package> [<preamble>]` audits one (`latexml::conformance::audit_package`) and
//! prints a summary row `#<TAB>package<TAB>defined<TAB>passthrough<TAB>conformant<TAB>findings<TAB>
//! raw_errors<TAB>binding_errors<TAB>raw_warnings<TAB>binding_warnings`, then one row per finding: package, control sequence, severity,
//! mismatch classes, the real and the binding signature, the two chains, the two walks' notes. The driver is
//! `tools/perfect_kernel/binding_conformance.sh`, one process per package.

use latexml::conformance::{Arg, Severity, View, audit_package, real_views};

fn signature(view: &View) -> String {
  view
    .args
    .iter()
    .map(|arg| match arg {
      Arg::Mandatory => "M".to_string(),
      Arg::Optional(Some(default)) => format!("O{{{default}}}"),
      Arg::Optional(None) => "O".to_string(),
      Arg::Star => "S".to_string(),
      Arg::Peek(token) => format!("P{token}"),
      Arg::Delimited(until) => format!("D{{{until}}}"),
      Arg::Literal(text) => format!("L{{{text}}}"),
      Arg::Scan(kind) => format!("T{{{kind}}}"),
    })
    .collect::<Vec<_>>()
    .join(" ")
    + if view.complete { "" } else { " …" }
    + if view.prologue.enter_horizontal {
      " +H"
    } else {
      ""
    }
    + if view.prologue.leave_horizontal {
      " -H"
    } else {
      ""
    }
}

/// A report field: control characters as TeX's `^^` notation (`^^M` for a carriage return), so a
/// signature delimited by one cannot split a row.
fn clean(text: &str) -> String {
  text
    .chars()
    .map(|c| match c {
      '\t' | '\n' => " ".to_string(),
      c if (c as u32) < 0x20 || c as u32 == 0x7f => format!("^^{}", ((c as u8) ^ 0x40) as char),
      c => c.to_string(),
    })
    .collect()
}

fn main() {
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let args: Vec<String> = std::env::args().skip(1).collect();
  match args.first().map(String::as_str) {
    Some("--list") => {
      let mut names: Vec<&str> = latexml_package::binding_names()
        .iter()
        .chain(latexml_contrib::binding_names())
        .filter(|(_, ext)| *ext == "sty")
        .map(|(name, _)| *name)
        .collect();
      names.sort_unstable();
      names.dedup();
      for name in names {
        println!("{name}");
      }
    },
    Some("--views") => {
      // `--views <package> [<preamble>]`: the real side's view of every public macro, for triage.
      let package = args.get(1).map(String::as_str).unwrap_or_default();
      let preamble = args.get(2).map(String::as_str).unwrap_or("");
      let (diagnostics, views) = real_views(package, preamble);
      println!(
        "#\t{package}\t{}\t{}\t{}",
        views.len(),
        diagnostics.errors,
        diagnostics.warnings
      );
      for (cs, view) in &views {
        println!(
          "{cs}\t{}\t{}\t{}",
          clean(&signature(view)),
          clean(&view.chain.join(" ")),
          clean(&view.notes.join("; "))
        );
      }
    },
    Some(package) => {
      let preamble = args.get(1).map(String::as_str).unwrap_or("");
      let audit = audit_package(package, preamble);
      println!(
        "#\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        audit.package,
        audit.defined,
        audit.passthrough,
        audit.conformant,
        audit.findings.len(),
        audit.raw_errors,
        audit.binding_errors,
        audit.raw_warnings,
        audit.binding_warnings
      );
      for finding in &audit.findings {
        let (severity, classes) = if finding.binding.is_none() {
          ("LOW", "UNDEFINED".to_string())
        } else {
          let severity = match finding.mismatches.iter().map(|m| m.severity()).min() {
            Some(Severity::High) => "HIGH",
            Some(Severity::Medium) => "MEDIUM",
            _ => "LOW",
          };
          let classes: Vec<&str> = finding.mismatches.iter().map(|m| m.class()).collect();
          (severity, classes.join(","))
        };
        println!(
          "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
          audit.package,
          finding.cs,
          severity,
          classes,
          clean(&signature(&finding.raw)),
          finding
            .binding
            .as_ref()
            .map(|b| clean(&signature(b)))
            .unwrap_or_default(),
          clean(&finding.raw.chain.join(" ")),
          finding
            .binding
            .as_ref()
            .map(|b| clean(&b.chain.join(" ")))
            .unwrap_or_default(),
          clean(&finding.raw.notes.join("; ")),
          finding
            .binding
            .as_ref()
            .map(|b| clean(&b.notes.join("; ")))
            .unwrap_or_default(),
        );
      }
    },
    None => {
      eprintln!(
        "usage: binding_audit --list | --views <package> [<preamble>] | <package> [<preamble>]"
      );
      std::process::exit(2);
    },
  }
}
