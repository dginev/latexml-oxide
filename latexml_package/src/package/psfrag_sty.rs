//! psfrag.sty — PostScript fragment overlays on EPS images
//! Perl: psfrag.sty.ltxml — 166 lines
//! Stores psfrag commands for later use when including EPS graphics.
//! The actual overlay is done by LaTeX (we just preserve the fragments).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L25-27 — initial state: psfrag_scan_all defaults to the
  // 2.09_COMPATIBILITY flag (true when the document used \documentstyle),
  // psfrag_scan starts off. No reader consumes these in Rust yet, but
  // setting them at load time keeps state shape consistent with Perl
  // so the future \includegraphics hook can LookupValue them directly.
  AssignValue!("psfrag_scan_all" => state::lookup_bool("2.09_COMPATIBILITY"));
  AssignValue!("psfrag_scan"     => 0i32);

  // Options — Perl L28-32. Each declared option toggles psfrag_scan_all:
  //   209mode, scanall → true
  //   2emode           → false
  DeclareOption!("209mode", { AssignValue!("psfrag_scan_all" => true); });
  DeclareOption!("2emode",  { AssignValue!("psfrag_scan_all" => false); });
  DeclareOption!("scanall", { AssignValue!("psfrag_scan_all" => true); });
  ProcessOptions!();

  // \psfrag — stores fragment for later overlay — Perl L46-55
  // NOT a constructor since args should not be digested yet
  DefPrimitive!("\\psfrag OptionalMatch:* Semiverbatim [][][][]{}", None);
  DefConstructor!("\\lx@delayed@psfrag OptionalMatch:* Semiverbatim [][][][]{}", "");

  // Scan control — Perl L57-64.
  // Perl DefConstructor(...afterDigest {save_psfrag(cs); AssignValue(psfrag_scan=>0/1)});
  // Rust implements the state toggle (psfrag_scan int). save_psfrag()
  // would append to saved_psfragments, but the Rust \includegraphics
  // hook doesn't consult that list yet (see the L24-28 TODO note), so
  // skipping it is not observable. When that hook lands, extend these
  // to also append the CS invocation to saved_psfragments.
  DefPrimitive!("\\psfragscanon", {
    AssignValue!("psfrag_scan" => 1i32);
  });
  DefPrimitive!("\\psfragscanoff", {
    AssignValue!("psfrag_scan" => 0i32);
  });

  // The Perl version hooks into \includegraphics and \epsfbox to check
  // if the image is an EPS that needs psfrag processing, and if so,
  // wraps it in a <ltx:picture> with the TeX overlay.
  // This requires image type detection (psfrag_requirepicture) which
  // we don't have. For now, includegraphics works normally without overlay.

  // Rescan macros — Perl L78-85
  DefMacro!("\\tex Semiverbatim", "#1");
  DefMacro!("\\psfragrescan", "");
  DefMacro!("\\psfragrescanoff", "");
  DefMacro!("\\psfragrescanon", "");
  DefMacro!("\\psfragdebugon", "");
  DefMacro!("\\psfragdebugoff", "");

  // Perl psfrag.sty.ltxml L149: DefEnvironment('{psfrags}', '#body').
  // Pure grouping, no content transform. Previously unported.
  DefEnvironment!("{psfrags}", "#body");
});
