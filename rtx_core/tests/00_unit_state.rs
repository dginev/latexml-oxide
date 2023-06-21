use rtx_core::common::arena;
use rtx_core::common::locator::Locator;
use rtx_core::definition::expandable::Expandable;
use rtx_core::state::*;
use rtx_core::token::{Catcode, Token};
use rtx_core::tokens::Tokens;
use rtx_core::{s, CharToken, Explode, Token, T_CS, T_SPACE};
use rustc_hash::FxHashMap as HashMap;
use std::collections::VecDeque;

#[test]
fn basic_state_init() {
  let state = State::new(StateOptions::default());
  assert_eq!(state.lookup_catcode('@'), None); // OTHER

  let state_std = State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  });
  assert_eq!(state_std.lookup_catcode('@'), None); // OTHER
  assert_eq!(state_std.lookup_catcode('\\'), Some(Catcode::ESCAPE));

  let std_sty = State::new(StateOptions {
    catcodes: Some(Catcodes::Style),
    ..StateOptions::default()
  });
  assert_eq!(std_sty.lookup_catcode('@'), Some(Catcode::LETTER));
}

#[test]
fn assign_lookup_value() {
  let mut state = State::new(StateOptions::default());
  // initially missing
  assert!(!state.has_value("STRICT"));

  let strict_value = "testing strict";
  state.assign_value("STRICT", strict_value, None);
  match state.lookup_value("STRICT") {
    None => panic!("Couldn't lookup STRICT value after assignment"),
    Some(&Stored::String(ref received_value)) => {
      assert_eq!(arena::to_string(*received_value), strict_value)
    },
    Some(_) => panic!("Looked up value of STRICT didn't match assigned value"),
  };

  let mut hash_val = HashMap::default();
  hash_val.insert(s!("a"), Stored::Bool(true));
  let hash_store = Stored::HashStored(hash_val);

  state.assign_value("hashref_test", hash_store, Some(Scope::Global));
  match state.lookup_value("hashref_test") {
    None => panic!("Couldn't lookup hashref_test value after assignment"),
    Some(&Stored::HashStored(ref received_hash)) => match received_hash.get("a") {
      None => panic!("Assigned hash was missing key!"),
      Some(&Stored::Bool(ref b)) => assert_eq!(*b, true),
      Some(_) => panic!("Assigned hash had malformed key!"),
    },
    Some(_) => panic!("Looked up value of hashref_test didn't match assignment value"),
  };

  match state.remove_value("STRICT") {
    None => panic!("Couldn't lookup STRICT value on removal"),
    Some(Stored::String(received_value)) => {
      arena::with(received_value, |str| assert_eq!(str, strict_value))
    },
    Some(_) => panic!("Looked up value of STRICT didn't match removed value"),
  };

  // missing after removal
  assert!(!state.has_value("STRICT"));
}

#[test]
fn scoped_assign_lookup_value() {
  // Let us try some scoped assignments:
  // First, can we push/pop frames?
  let mut state = State::new(StateOptions::default());
  assert!(!state.has_value("foo"));
  state.assign_value("foo", s!("bar"), Some(Scope::Global));
  match state.lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(&Stored::String(ref received_value)) => arena::with(*received_value, |rstr| {
      assert_eq!(rstr, "bar", "global assignment should have value bar")
    }),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  state.push_frame();

  state.assign_value("foo", s!("baz"), Some(Scope::Local));
  match state.lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(&Stored::String(ref received_value)) => arena::with(*received_value, |rstr| {
      assert_eq!(rstr, "baz", "local assignment should have value baz")
    }),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  state.assign_value("foo", s!("overwrite"), Some(Scope::Local));
  match state.lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(&Stored::String(ref received_value)) => arena::with(*received_value, |rstr| {
      assert_eq!(
        rstr, "overwrite",
        "second local assignment should have value overwrite"
      )
    }),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  assert!(state.pop_frame().is_ok());

  match state.lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(&Stored::String(ref received_value)) => arena::with(*received_value, |rstr| {
      assert_eq!(rstr, "bar", "global assignment should have value bar")
    }),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };
}

#[test]
fn assign_lookup_arrays() {
  let mut state = State::new(StateOptions::default());
  let mock_vec = ["a", "b", "c"]
    .iter()
    .map(|x| Stored::String(arena::pin(x)))
    .collect::<VecDeque<Stored>>();
  state.assign_value(
    "SEARCHPATHS",
    Stored::VecDequeStored(mock_vec.clone()),
    None,
  );
  match state.lookup_value("SEARCHPATHS") {
    None => panic!("Couldn't lookup SEARCHPATHS value after assignment"),
    Some(&Stored::VecDequeStored(ref received_value)) => assert_eq!(
      received_value, &mock_vec,
      "looked up array has correct value"
    ),
    Some(_) => panic!("Looked up value of SEARCHPATHS didn't match assignment value"),
  };

  state.unshift_value(
    "empty_key",
    vec![Stored::String(arena::pin_static("mydir"))],
  );
  let shifted = state.shift_value("empty_key").unwrap();
  if let Some(Stored::String(shifted)) = shifted {
    arena::with(shifted, |shifted_str| {
      assert_eq!(shifted_str, "mydir", "shift/unshift new key")
    });
  } else {
    panic!("state.shift_value returned wrong/no Stored")
  }

  state.unshift_value("SEARCHPATHS", vec![Stored::String(arena::pin_static("d"))]);
  if let Some(vdq) = state.lookup_vecdeque("SEARCHPATHS") {
    let mut vdq_expected = VecDeque::new();
    for entry in &["d", "a", "b", "c"] {
      vdq_expected.push_back(Stored::String(arena::pin_static(entry)));
    }
    assert_eq!(vdq, &vdq_expected, "shift/unshift existing key");
  } else {
    panic!("state.lookup_vecdeque returned None");
  }

  assert_eq!(
    state.shift_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("d"))),
    "shift searchpaths"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("c"))),
    "pop searchpaths"
  );
  assert_eq!(
    state.shift_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("a"))),
    "shift searchpaths"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("b"))),
    "pop searchpaths"
  );
  assert_eq!(
    state.shift_value("SEARCHPATHS").unwrap(),
    None,
    "shift searchpaths None"
  );
  assert_eq!(state.pop_value("SEARCHPATHS").unwrap(), None, "pop searchpaths None");
  assert_eq!(
    state.lookup_value("SEARCHPATHS"),
    Some(&Stored::VecDequeStored(VecDeque::new())),
    "lookup searchpaths []"
  );

  let mut vdq = ["a", "b", "c"]
    .iter()
    .map(|x| Stored::String(arena::pin_static(x)))
    .collect::<VecDeque<Stored>>();
  state.assign_value("SEARCHPATHS", Stored::VecDequeStored(vdq.clone()), None);
  let new_d = Stored::String(arena::pin_static("d"));
  assert!(state.push_value("SEARCHPATHS", new_d.clone()).is_ok());
  vdq.push_back(new_d.clone());
  assert_eq!(
    state.lookup_value("SEARCHPATHS"),
    Some(&Stored::VecDequeStored(vdq)),
    "push works as intended"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS").unwrap(),
    Some(new_d),
    "pop searchpaths"
  );
  assert_eq!(
    state.shift_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("a"))),
    "shift searchpaths"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("c"))),
    "pop searchpaths"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS").unwrap(),
    Some(Stored::String(arena::pin_static("b"))),
    "pop searchpaths"
  );
  assert_eq!(
    state.shift_value("SEARCHPATHS").unwrap(),
    None,
    "shift searchpaths None"
  );
  assert_eq!(state.pop_value("SEARCHPATHS").unwrap(), None, "pop searchpaths None");
  assert_eq!(
    state.lookup_value("SEARCHPATHS"),
    Some(&Stored::VecDequeStored(VecDeque::new())),
    "lookup searchpaths []"
  );
}

#[test]
fn install_definition_and_meaning() {
  let mut state = State::new(StateOptions::default());
  state.initialize_stomach();
  let job_definition = Expandable {
    cs: T_CS!("\\jobname"),
    paramlist: None,
    //       expansion: SimpleExpansion!(Tokens::new(Explode!("name"))),
    expansion: Tokens::new(Explode!("name")).into(),
    locator: Locator::new("00_unit_test.rs".to_string(), 180, 1, 188, 5),
    is_protected: state.get_prefix("protected"),
    ..Expandable::default()
  };

  // Assign a Meaning
  state.assign_meaning(&T_CS!("\\foobar"), job_definition, Some(Scope::Local));
  if let Some(Stored::Expandable(ref stored_meaning)) =
    state.lookup_meaning(&T_CS!("\\foobar")).as_deref()
  {
    assert_eq!(stored_meaning.cs, T_CS!("\\jobname")); // Note: meaning for \foobar still has
                                                       // definition for CS \jobname
  } else {
    panic!("Failed to lookup installed meaning!");
  }

  let looked_up_meaning = {
    state
      .lookup_meaning(&T_CS!("\\foobar"))
      .unwrap()
      .into_owned()
  };
  {
    state.assign_meaning(
      &T_CS!("\\foolet"),
      looked_up_meaning.clone(),
      Some(Scope::Local),
    );
  }
  assert_eq!(
    state.lookup_meaning(&T_CS!("\\foolet")).as_deref(),
    Some(&looked_up_meaning),
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
  // TODO
  // state_mut!().assign_value("daemon_mode", Stored::Bool(false), Some(Scope::Global));
  // state_mut!().push_daemon_frame();
  // state_mut!().assign_value("daemon_mode", Stored::Bool(true),Some(Scope::Global));
  // match state!().lookup_value("daemon_mode") {
  //   None => panic!("Couldn't lookup daemon_mode value after assignment"),
  //   Some(& Stored::Bool(b)) => assert_eq!(b, true, "in daemon mode"),
  //   Some(_) => panic!("Looked up value of daemon_mode didn't match assignment value")
  // };

  // state_mut!().pop_daemon_frame();
  // match state!().lookup_value("daemon_mode") {
  //   None => panic!("Couldn't lookup daemon_mode value after assignment"),
  //   Some(& Stored::Bool(b)) => assert_eq!(b, false, "out of daemon mode"),
  //   Some(_) => panic!("Looked up value of daemon_mode didn't match assignment value")
  // };
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
  let mut state = State::new(StateOptions::default());
  // TODO: Test with char catcodes

  state.begin_semiverbatim(None);

  assert!(state.end_semiverbatim().is_ok());
}
