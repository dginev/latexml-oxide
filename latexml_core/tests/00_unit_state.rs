use latexml_core::common::arena;
use latexml_core::common::arena::SymHashMap;
use latexml_core::common::locator::Locator;
use latexml_core::definition::expandable::Expandable;
use latexml_core::state::*;
use latexml_core::token::{Catcode, Token};
use latexml_core::tokens::Tokens;
use latexml_core::{CharToken, Explode, T_CS, T_SPACE, Token, s};
use std::collections::VecDeque;

#[test]
fn basic_state_init() {
  set_state(State::new(StateOptions::default()));
  assert_eq!(lookup_catcode('@'), None); // OTHER

  set_state(State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  }));
  assert_eq!(lookup_catcode('@'), None); // OTHER
  assert_eq!(lookup_catcode('\\'), Some(Catcode::ESCAPE));

  use_sty_state();
  assert_eq!(lookup_catcode('@'), Some(Catcode::LETTER));
  use_main_state();
}

#[test]
fn assign_lookup_value() {
  set_state(State::new(StateOptions::default()));
  // initially missing
  assert!(!has_value("STRICT"));

  let strict_value = "testing strict";
  assign_value("STRICT", strict_value, None);
  match lookup_value("STRICT") {
    None => panic!("Couldn't lookup STRICT value after assignment"),
    Some(Stored::String(ref received_value)) => {
      assert_eq!(arena::to_string(*received_value), strict_value)
    },
    Some(_) => panic!("Looked up value of STRICT didn't match assigned value"),
  };

  let mut hash_val = SymHashMap::default();
  hash_val.insert("a", Stored::Bool(true));
  let hash_store = Stored::HashStored(hash_val);

  assign_value("hashref_test", hash_store, Some(Scope::Global));
  match lookup_value("hashref_test") {
    None => panic!("Couldn't lookup hashref_test value after assignment"),
    Some(Stored::HashStored(ref received_hash)) => match received_hash.get("a") {
      None => panic!("Assigned hash was missing key!"),
      Some(Stored::Bool(b)) => assert!(b),
      Some(_) => panic!("Assigned hash had malformed key!"),
    },
    Some(_) => panic!("Looked up value of hashref_test didn't match assignment value"),
  };

  match remove_value("STRICT") {
    None => panic!("Couldn't lookup STRICT value on removal"),
    Some(Stored::String(received_value)) => {
      arena::with(received_value, |str| assert_eq!(str, strict_value))
    },
    Some(_) => panic!("Looked up value of STRICT didn't match removed value"),
  };

  // missing after removal
  assert!(!has_value("STRICT"));
}

#[test]
fn scoped_assign_lookup_value() {
  // Let us try some scoped assignments:
  // First, can we push/pop frames?
  set_state(State::new(StateOptions::default()));
  assert!(!has_value("foo"));
  assign_value("foo", s!("bar"), Some(Scope::Global));
  match lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(Stored::String(ref received_value)) => arena::with(*received_value, |rstr| {
      assert_eq!(rstr, "bar", "global assignment should have value bar")
    }),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  push_frame();

  assign_value("foo", s!("baz"), Some(Scope::Local));
  match lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(Stored::String(ref received_value)) => arena::with(*received_value, |rstr| {
      assert_eq!(rstr, "baz", "local assignment should have value baz")
    }),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  assign_value("foo", s!("overwrite"), Some(Scope::Local));
  match lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(Stored::String(ref received_value)) => arena::with(*received_value, |rstr| {
      assert_eq!(
        rstr, "overwrite",
        "second local assignment should have value overwrite"
      )
    }),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  assert!(pop_frame().is_ok());

  match lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(Stored::String(ref received_value)) => arena::with(*received_value, |rstr| {
      assert_eq!(rstr, "bar", "global assignment should have value bar")
    }),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };
}

#[test]
fn assign_lookup_arrays() {
  set_state(State::new(StateOptions::default()));
  let mock_vec = ["a", "b", "c"]
    .iter()
    .map(|x| Stored::String(arena::pin(x)))
    .collect::<VecDeque<Stored>>();
  assign_value(
    "SEARCHPATHS",
    Stored::VecDequeStored(mock_vec.clone()),
    None,
  );
  match lookup_value("SEARCHPATHS") {
    None => panic!("Couldn't lookup SEARCHPATHS value after assignment"),
    Some(Stored::VecDequeStored(ref received_value)) => assert_eq!(
      received_value, &mock_vec,
      "looked up array has correct value"
    ),
    Some(_) => panic!("Looked up value of SEARCHPATHS didn't match assignment value"),
  };

  unshift_value("empty_key", vec![Stored::String(arena::pin_static(
    "mydir",
  ))]);
  let shifted = shift_value("empty_key").unwrap();
  if let Some(Stored::String(shifted)) = shifted {
    arena::with(shifted, |shifted_str| {
      assert_eq!(shifted_str, "mydir", "shift/unshift new key")
    });
  } else {
    panic!("shift_value returned wrong/no Stored")
  }

  unshift_value("SEARCHPATHS", vec![Stored::String(arena::pin_static("d"))]);
  match lookup_vecdeque("SEARCHPATHS") { Some(vdq) => {
    let mut vdq_expected = VecDeque::new();
    for entry in &["d", "a", "b", "c"] {
      vdq_expected.push_back(Stored::String(arena::pin_static(entry)));
    }
    assert_eq!(vdq, vdq_expected, "shift/unshift existing key");
  } _ => {
    panic!("state.lookup_vecdeque returned None");
  }}

  assert_eq!(
    shift_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("d"))),
    "shift searchpaths"
  );
  assert_eq!(
    pop_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("c"))),
    "pop searchpaths"
  );
  assert_eq!(
    shift_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("a"))),
    "shift searchpaths"
  );
  assert_eq!(
    pop_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("b"))),
    "pop searchpaths"
  );
  assert_eq!(
    shift_value("SEARCHPATHS").unwrap(),
    None,
    "shift searchpaths None"
  );
  assert_eq!(
    pop_value("SEARCHPATHS").unwrap(),
    None,
    "pop searchpaths None"
  );
  assert_eq!(
    lookup_value("SEARCHPATHS"),
    Some(Stored::VecDequeStored(VecDeque::new())),
    "lookup searchpaths []"
  );

  let mut vdq = ["a", "b", "c"]
    .iter()
    .map(|x| Stored::String(arena::pin_static(x)))
    .collect::<VecDeque<Stored>>();
  assign_value("SEARCHPATHS", Stored::VecDequeStored(vdq.clone()), None);
  let new_d = Stored::String(arena::pin_static("d"));
  assert!(push_value("SEARCHPATHS", new_d.clone()).is_ok());
  vdq.push_back(new_d.clone());
  assert_eq!(
    lookup_value("SEARCHPATHS"),
    Some(Stored::VecDequeStored(vdq)),
    "push works as intended"
  );
  assert_eq!(
    pop_value("SEARCHPATHS").unwrap(),
    Some(new_d),
    "pop searchpaths"
  );
  assert_eq!(
    shift_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("a"))),
    "shift searchpaths"
  );
  assert_eq!(
    pop_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("c"))),
    "pop searchpaths"
  );
  assert_eq!(
    pop_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("b"))),
    "pop searchpaths"
  );
  assert_eq!(
    shift_value("SEARCHPATHS").unwrap(),
    None,
    "shift searchpaths None"
  );
  assert_eq!(
    pop_value("SEARCHPATHS").unwrap(),
    None,
    "pop searchpaths None"
  );
  assert_eq!(
    lookup_value("SEARCHPATHS"),
    Some(Stored::VecDequeStored(VecDeque::new())),
    "lookup searchpaths []"
  );
}

#[test]
fn install_definition_and_meaning() {
  set_state(State::new(StateOptions::default()));
  ::latexml_core::stomach::initialize_stomach();
  let job_definition = Expandable {
    cs: T_CS!("\\jobname"),
    paramlist: None,
    //       expansion: SimpleExpansion!(Tokens::new(Explode!("name"))),
    expansion: Tokens::new(Explode!("name")).into(),
    locator: Locator::new("00_unit_test.rs", 180, 1, 188, 5),
    is_protected: get_prefix("protected"),
    ..Expandable::default()
  };

  // Assign a Meaning
  assign_meaning(&T_CS!("\\foobar"), job_definition, Some(Scope::Local));
  match lookup_meaning(&T_CS!("\\foobar")) { Some(Stored::Expandable(ref stored_meaning)) => {
    assert_eq!(stored_meaning.cs, T_CS!("\\jobname")); // Note: meaning for \foobar still has
  // definition for CS \jobname
  } _ => {
    panic!("Failed to lookup installed meaning!");
  }}

  let looked_up_meaning = { lookup_meaning(&T_CS!("\\foobar")).unwrap() };
  {
    assign_meaning(
      &T_CS!("\\foolet"),
      looked_up_meaning.clone(),
      Some(Scope::Local),
    );
  }
  assert_eq!(
    lookup_meaning(&T_CS!("\\foolet")),
    Some(looked_up_meaning),
    "Meanings match"
  );
}

#[test]
fn assign_lookup_mapping() {
  // # 10. assign Mapping
  // ok(!state!().lookupMapping('TAG_PROPERTIES', "tag"), "lookupMapping is false on new keys");
  // state_mut!().assignMapping('TAG_PROPERTIES', "tag" => {});
  // my $props = state!().lookupMapping('TAG_PROPERTIES', "tag");
  // is_deeply($props,{},"Empty mapping hash roundtrip");
  // my $undef_val = $$props{"afterOpen"};
  // assert_eq!($undef_val,undef,"Surviving a lookup of a new key");

  // my $wdr_url = "http://www.w3.org/2007/05/powder#";
  // state_mut!().assignMapping("RDFa_prefixes",
  //  "wdr"     => $wdr_url);
  // assert_eq!(state!().lookupMapping("RDFa_prefixes","wdr"),$wdr_url,"asssign/lookupMapping
  // roundtrip"); my %rdf_prefixes = (
  //   "cc"      => "http://creativecommons.org/ns#",
  //   "ctag"    => "http://commontag.org/ns#",
  //   "dc"      => "http://purl.org/dc/terms/",
  //   "dcterms" => "http://purl.org/dc/terms/",
  //   "ical"    => "http://www.w3.org/2002/12/cal/icaltzd#",
  //   "foaf"    => "http://xmlns.com/foaf/0.1/",
  //   "gr"      => "http://purl.org/goodrelations/v1#",
  //   "grddl"   => "http://www.w3.org/2003/g/data-view#",
  //   "ma"      => "http://www.w3.org/ns/ma-ont#",
  //   "og"      => "http://ogp.me/ns#",
  //   "owl"     => "http://www.w3.org/2002/07/owl#",
  //   "rdf"     => "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
  //   "rdfa"    => "http://www.w3.org/ns/rdfa#",
  //   "rdfs"    => "http://www.w3.org/2000/01/rdf-schema#",
  //   "rev"     => "http://purl.org/stuff/rev#",
  //   "rif"     => "http://www.w3.org/2007/rif#",
  //   "rr"      => "http://www.w3.org/ns/r2rml#",
  //   "schema"  => "http://schema.org/",
  //   "sioc"    => "http://rdfs.org/sioc/ns#",
  //   "skos"    => "http://www.w3.org/2004/02/skos/core#",
  //   "skosxl"  => "http://www.w3.org/2008/05/skos-xl#",
  //   "v"       => "http://rdf.data-vocabulary.org/#",
  //   "vcard"   => "http://www.w3.org/2006/vcard/ns#",
  //   "void"    => "http://rdfs.org/ns/void#",
  //   "xhv"     => "http://www.w3.org/1999/xhtml/vocab#",
  //   "xml"     => "http://www.w3.org/XML/1998/namespace",
  //   "xsd"     => "http://www.w3.org/2001/XMLSchema#",
  //   "wdr"     => "http://www.w3.org/2007/05/powder#",
  //   "wdrs"    => "http://www.w3.org/2007/05/powder-s#",
  // );
  // foreach my $p (keys %rdf_prefixes) {
  //  state_mut!().assignMapping('RDFa_prefixes', $p => $rdf_prefixes{$p}); }
  // is_deeply(state!().lookup_value('RDFa_prefixes'),\%rdf_prefixes,"Entire RDF mapping");
}

#[test]
fn push_pop_daemon_frames() {
  set_state(State::new(StateOptions::default()));
  assign_value("daemon_mode", Stored::Bool(false), Some(Scope::Global));
  push_daemon_frame();
  assign_value("daemon_mode", Stored::Bool(true), Some(Scope::Global));
  match lookup_value("daemon_mode") {
    None => panic!("Couldn't lookup daemon_mode value after assignment"),
    Some(Stored::Bool(b)) => assert!(b, "in daemon mode"),
    Some(_) => panic!("Looked up value of daemon_mode didn't match assignment value")
  };

  pop_daemon_frame().unwrap();
  match lookup_value("daemon_mode") {
    None => panic!("Couldn't lookup daemon_mode value after assignment"),
    Some(Stored::Bool(b)) => assert!(!b, "out of daemon mode"),
    Some(_) => panic!("Looked up value of daemon_mode didn't match assignment value")
  };
}

#[test]
fn texy_ops() {
  // # 13. TeXy ops
  // my $mock1 = T_CS('\mock1');
  // my $mock2 = T_CS('\mock2');
  // my $mock3 = T_CS('\mock3');
  // state_mut!().pushValue('DOCUMENT_REWRITE_RULES',
  //     $mock1);
  // my @mocks = ($mock2,$mock3);
  // state_mut!().pushValue('DOCUMENT_REWRITE_RULES',@mocks);
  // assert_eq!(state_mut!().shift_value('DOCUMENT_REWRITE_RULES'),$mock1,"shift_value 1");
  // assert_eq!(state_mut!().shift_value('DOCUMENT_REWRITE_RULES'),$mock2,"shift_value 2");
  // assert_eq!(state_mut!().shift_value('DOCUMENT_REWRITE_RULES'),$mock3,"shift_value 3");

  // state_mut!().pushValue("PENDING_RESOURCES" => ["resource1", foo => 1, bar => 2]);
  // state_mut!().pushValue("PENDING_RESOURCES" => ["resource2", baz => 3, bam => 4]);
  // state_mut!().pushValue("PENDING_RESOURCES" => ["resource3", a => 5, b => 6]);
  // my $resources = state!().lookup_value("PENDING_RESOURCES");
  // is_deeply($resources, [
  //   ["resource1", foo => 1, bar => 2],
  //   ["resource2", baz => 3, bam => 4],
  //   ["resource3", a => 5, b => 6]],"pending resources stored in arrayref of arrayrefs");
}

#[test]
fn semiverbatim() {
  set_state(State::new(StateOptions::default()));
  // TODO: Test with char catcodes

  begin_semiverbatim(None);
  assert!(end_semiverbatim().is_ok());
}
