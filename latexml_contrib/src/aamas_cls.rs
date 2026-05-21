//! Stub for AAMAS conference class.
//!
//! aamas.cls is an ACM-style class. Reuse acmart's macros for all the
//! ACM frontmatter (\setcopyright, \acmConference, \acmDOI, etc.) and
//! treat content like amsart.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("acmart");
});
