#[macro_use]
extern crate latexml_codegen;
#[macro_use]
extern crate latexml_package;

mod helpers;

use latexml::util::test::*;
use latexml_core::common::error::*;
use std::rc::Rc;

const DIR: &str = "tests/namespace";

pub fn namespace_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "ns1" => Some(helpers::ns1_tex::load_definitions()),
    "ns2" => Some(helpers::ns2_tex::load_definitions()),
    "ns3" => Some(helpers::ns3_tex::load_definitions()),
    "ns4" => Some(helpers::ns4_tex::load_definitions()),
    "ns5" => Some(helpers::ns5_tex::load_definitions()),
    _ => None,
  }
}

#[test]
fn ns1_test() {
  latexml_test_single("tests/namespace/ns1.tex", "ns1", DIR, None, Some(Rc::new(namespace_dispatch)));
}

#[test]
fn ns2_test() {
  latexml_test_single("tests/namespace/ns2.tex", "ns2", DIR, None, Some(Rc::new(namespace_dispatch)));
}

#[test]
fn ns3_test() {
  latexml_test_single("tests/namespace/ns3.tex", "ns3", DIR, None, Some(Rc::new(namespace_dispatch)));
}

#[test]
fn ns4_test() {
  latexml_test_single("tests/namespace/ns4.tex", "ns4", DIR, None, Some(Rc::new(namespace_dispatch)));
}

#[test]
fn ns5_test() {
  latexml_test_single("tests/namespace/ns5.tex", "ns5", DIR, None, Some(Rc::new(namespace_dispatch)));
}
