//! MACSYNC (AUDMAC rows 2-4, 6, 8, 9, 11): a binding macro means what Perl's same-named API means (DG 2026-11-14,
//! ruling 11). The macros run in the session a minimal conversion leaves, their Warnings read from a bound log.

use latexml_core::util::logger::{bind_log, flush_log};
use latexml_engine::prelude::*;

use super::perfect_kernel_batch46::{convert_files, convert_with_then, error_count, warning_count};

const DOC: &str = "\\documentclass{article}\n\\begin{document}\nx\n\\end{document}\n";

fn mode() -> String { lookup_string_from_sym(pin!("MODE")) }

fn vertical() {
  assign_value("MODE", Stored::String(pin("vertical")), None);
  assign_value("BOUND_MODE", Stored::String(pin("vertical")), None);
}

fn dimen(cs: &str) -> i64 { lookup_register_quiet(cs).map_or(i64::MIN, |v| v.value_of()) }

/// The macros' outcomes, one line each, then the Warnings they raised.
fn macro_rows() -> Result<Vec<String>> {
  let mut rows = Vec::new();
  // Row 2, Package.pm:393-402: `Digest` leaves the horizontal mode its digestion entered.
  vertical();
  latexml_engine::Digest!(Tokens::new(vec![T_LETTER!("A")]))?;
  rows.push(format!("Digest! mode={}", mode()));
  // Row 8, Package.pm:996: `RawTeX` ends with `leaveHorizontal_internal`.
  vertical();
  raw_tex("A")?;
  rows.push(format!("RawTeX! mode={}", mode()));
  // Row 4, Package.pm:1397: `AssignRegister($cs, $value, $scope)`, `$cs` a name or a Token.
  let before = dimen("\\dimen@");
  push_frame();
  latexml_core::AssignRegister!("\\dimen@", Dimension::new(1).into());
  latexml_core::AssignRegister!(
    T_CS!("\\dimen@ii"),
    Dimension::new(2).into(),
    Some(Scope::Global)
  );
  pop_frame()?;
  rows.push(format!(
    "AssignRegister! local reverted={} global={}",
    dimen("\\dimen@") == before,
    dimen("\\dimen@ii")
  ));
  latexml_core::AssignRegister!("\\nosuchreg", Number::new(1).into(), None, Vec::new());
  // Row 6, Package.pm:1360-1369: one Warning text, the macro's and the core's; the core lookup is quiet on an
  // undefined cs (kernel probes of optional registers), a defined non-register Warns.
  let value = latexml_engine::LookupRegister!("\\nosuchreg");
  rows.push(format!("LookupRegister! {value:?}"));
  rows.push(format!(
    "lookup_register {:?}",
    lookup_register("\\nosuchreg", Vec::new())?
  ));
  // Row 9, Dimension.pm:27-46: a number is sp, rounded; an unreadable spec is 0 with a Warning.
  for spec in ["3pt", "1.5", "", "abc"] {
    rows.push(format!(
      "Dimension!({spec:?}) = {}",
      latexml_engine::Dimension!(spec).value_of()
    ));
  }
  // Row 11, Package.pm:312: `InstallDefinition($definition[, $scope])`.
  if let Some(relax) = lookup_meaning(&T_CS!("\\relax")) {
    latexml_engine::InstallDefinition!(relax.clone());
    latexml_engine::InstallDefinition!(relax, Some(Scope::Local));
  }
  Ok(rows)
}

#[test]
fn binding_macros_mean_what_perls_names_mean() {
  let (log, _xml, (rows, warnings)) = convert_with_then(DOC, None, |_| {
    bind_log();
    let rows = macro_rows().expect("the macros run");
    let warnings: Vec<String> = flush_log()
      .lines()
      .filter(|l| l.starts_with("Warning:"))
      .map(str::to_string)
      .collect();
    (rows, warnings)
  });
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(rows, [
    "Digest! mode=vertical",
    "RawTeX! mode=vertical",
    "AssignRegister! local reverted=true global=2",
    "LookupRegister! Number(0)",
    "lookup_register None",
    "lookup_register None",
    "Dimension!(\"3pt\") = 196608",
    "Dimension!(\"1.5\") = 1",
    "Dimension!(\"\") = 0",
    "Dimension!(\"abc\") = 0",
  ]);
  // Perl's one text (Package.pm:1367-1369, 1404-1405) from AssignRegister!, LookupRegister! and the core lookup.
  let not_register = "Warning:expected:register The control sequence \\nosuchreg is not a register";
  assert_eq!(warnings, [
    not_register,
    not_register,
    "Warning:expected:register The control sequence \\relax is not a register",
    "Warning:expected:dimension \"abc\" is not a dimension; using 0",
  ]);
}

/// Row 3, Package.pm:2534-2549: `\g@addto@macro` on an undefined cs defines it with the addition, as latex.ltx does
/// after TeX's error (pdflatex 1 `!`, "AXYB"), with a Warning (OXIDIZED_DESIGN #1035); the preamble `\AtEndOfPackage`,
/// whose hook `\csname` makes, raises nothing.
#[test]
fn add_to_macro_on_an_undefined_cs_warns() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/g_addto_macro_undefined_defines.tex"
  );
  let (log, xml) = convert_files(tex, &[]);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Warning:undefined:\\nohook \\nohook is not defined"),
    "{log}"
  );
  assert!(xml.contains("<p>AXYB</p>"), "{xml}");
}
