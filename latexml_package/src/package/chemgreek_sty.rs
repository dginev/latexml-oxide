use crate::prelude::*;

// chemgreek.sty — greek-letter command interface for mhchem and
// related chemistry packages. Real TL chemgreek.sty depends on
// xparse + expl3.
//
// Perl LaTeXML has no `chemgreek.sty.ltxml`; it raw-loads the actual
// TL .sty. Fourth shim of the SYNC_STATUS "raw-load enablement"
// plan (after xfor + mfirstuc + datatool-base), groundwork for
// retiring the `mhchem_sty.rs` contrib stub.

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("chemgreek", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
