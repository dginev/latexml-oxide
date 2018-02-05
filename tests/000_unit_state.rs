#[macro_use]
extern crate rtx_core;

use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use rtx_core::state::*;
use rtx_core::token::Catcode;
use rtx_core::tokens::Tokens;
use rtx_core::definition::expandable::Expandable;

#[test]
fn basic_state_init() {
  let state = State::new(StateOptions::default());
  assert_eq!(state.lookup_catcode(&'@'), None); // OTHER

  let state_standard = State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  });
  assert_eq!(state_standard.lookup_catcode(&'@'), None); // OTHER
  assert_eq!(state_standard.lookup_catcode(&'\\'), Some(Catcode::ESCAPE));

  let state_style = State::new(StateOptions {
    catcodes: Some(Catcodes::Style),
    ..StateOptions::default()
  });
  assert_eq!(state_style.lookup_catcode(&'@'), Some(Catcode::LETTER));
}

#[test]
fn assign_lookup_value() {
  let mut state = State::new(StateOptions::default());
  // initially missing
  assert!(state.lookup_value("STRICT").is_none());

  let strict_value = "testing strict".to_string();
  let strict_store = ObjectStore::String(strict_value.clone());
  state.assign_value("STRICT", strict_store, None);
  match state.lookup_value("STRICT") {
    None => panic!("Couldn't lookup STRICT value after assignment"),
    Some(&ObjectStore::String(ref received_value)) => assert_eq!(*received_value, strict_value),
    Some(_) => panic!("Looked up value of STRICT didn't match assigned value"),
  };

  let mut hash_val = HashMap::new();
  hash_val.insert("a".to_string(), ObjectStore::Bool(true));
  let hash_store = ObjectStore::HashOS(hash_val);

  state.assign_value("hashref_test", hash_store, Some(Scope::Global));
  match state.lookup_value("hashref_test") {
    None => panic!("Couldn't lookup hashref_test value after assignment"),
    Some(&ObjectStore::HashOS(ref received_hash)) => match received_hash.get("a") {
      None => panic!("Assigned hash was missing key!"),
      Some(&ObjectStore::Bool(ref b)) => assert_eq!(*b, true),
      Some(_) => panic!("Assigned hash had malformed key!"),
    },
    Some(_) => panic!("Looked up value of hashref_test didn't match assignment value"),
  };

  match state.remove_value("STRICT") {
    None => panic!("Couldn't lookup STRICT value on removal"),
    Some(ObjectStore::String(received_value)) => assert_eq!(received_value, strict_value),
    Some(_) => panic!("Looked up value of STRICT didn't match removed value"),
  };

  // missing after removal
  assert!(state.lookup_value("STRICT").is_none());
}

#[test]
fn scoped_assign_lookup_value() {
  // Let us try some scoped assignments:
  // First, can we push/pop frames?
  let mut state = State::new(StateOptions::default());
  assert!(state.lookup_value("foo").is_none());
  state.assign_value(
    "foo",
    ObjectStore::String("bar".to_string()),
    Some(Scope::Global),
  );
  match state.lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(&ObjectStore::String(ref received_value)) => assert_eq!(
      received_value, "bar",
      "global assignment should have value bar"
    ),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  state.push_frame();

  state.assign_value(
    "foo",
    ObjectStore::String("baz".to_string()),
    Some(Scope::Local),
  );
  match state.lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(&ObjectStore::String(ref received_value)) => assert_eq!(
      received_value, "baz",
      "local assignment should have value baz"
    ),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  state.assign_value(
    "foo",
    ObjectStore::String("overwrite".to_string()),
    Some(Scope::Local),
  );
  match state.lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(&ObjectStore::String(ref received_value)) => assert_eq!(
      received_value, "overwrite",
      "second local assignment should have value overwrite"
    ),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };

  state.pop_frame();

  match state.lookup_value("foo") {
    None => panic!("Couldn't lookup foo value after assignment"),
    Some(&ObjectStore::String(ref received_value)) => assert_eq!(
      received_value, "bar",
      "global assignment should have value bar"
    ),
    Some(_) => panic!("Looked up value of foo didn't match assignment value"),
  };
}

#[test]
fn assign_lookup_arrays() {
  let mut state = State::new(StateOptions::default());
  let mock_vec = ["a", "b", "c"]
    .iter()
    .map(|x| ObjectStore::String(x.to_string()))
    .collect::<VecDeque<ObjectStore>>();
  state.assign_value(
    "SEARCHPATHS",
    ObjectStore::VecDequeOS(mock_vec.clone()),
    None,
  );
  match state.lookup_value("SEARCHPATHS") {
    None => panic!("Couldn't lookup SEARCHPATHS value after assignment"),
    Some(&ObjectStore::VecDequeOS(ref received_value)) => assert_eq!(
      received_value, &mock_vec,
      "looked up array has correct value"
    ),
    Some(_) => panic!("Looked up value of SEARCHPATHS didn't match assignment value"),
  };

  state.unshift_value("empty_key", vec![ObjectStore::String("mydir".to_string())]);
  let shifted = state.shift_value("empty_key");
  if let Some(ObjectStore::String(shifted_str)) = shifted {
    assert_eq!(shifted_str, "mydir", "shift/unshift new key");
  } else {
    panic!("state.shift_value returned wrong/no ObjectStore")
  }

  state.unshift_value("SEARCHPATHS", vec![ObjectStore::String("d".to_string())]);
  if let Some(vdq) = state.lookup_vecdeque("SEARCHPATHS") {
    let mut vdq_expected = VecDeque::new();
    for entry in ["d", "a", "b", "c"].into_iter() {
      vdq_expected.push_back(ObjectStore::String(entry.to_string()));
    }
    assert_eq!(vdq, &vdq_expected, "shift/unshift existing key");
  } else {
    panic!("state.lookup_vecdeque returned None");
  }

  assert_eq!(
    state.shift_value("SEARCHPATHS"),
    Some(ObjectStore::String("d".to_owned())),
    "shift searchpaths"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS"),
    Some(ObjectStore::String("c".to_owned())),
    "pop searchpaths"
  );
  assert_eq!(
    state.shift_value("SEARCHPATHS"),
    Some(ObjectStore::String("a".to_owned())),
    "shift searchpaths"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS"),
    Some(ObjectStore::String("b".to_owned())),
    "pop searchpaths"
  );
  assert_eq!(
    state.shift_value("SEARCHPATHS"),
    None,
    "shift searchpaths None"
  );
  assert_eq!(state.pop_value("SEARCHPATHS"), None, "pop searchpaths None");
  assert_eq!(
    state.lookup_value("SEARCHPATHS"),
    Some(&ObjectStore::VecDequeOS(VecDeque::new())),
    "lookup searchpaths []"
  );

  let mut vdq = ["a", "b", "c"]
    .iter()
    .map(|x| ObjectStore::String(x.to_string()))
    .collect::<VecDeque<ObjectStore>>();
  state.assign_value("SEARCHPATHS", ObjectStore::VecDequeOS(vdq.clone()), None);
  let new_d = ObjectStore::String("d".to_owned());
  state.push_value("SEARCHPATHS", new_d.clone());
  vdq.push_back(new_d.clone());
  assert_eq!(
    state.lookup_value("SEARCHPATHS"),
    Some(&ObjectStore::VecDequeOS(vdq)),
    "push works as intended"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS"),
    Some(new_d),
    "pop searchpaths"
  );
  assert_eq!(
    state.shift_value("SEARCHPATHS"),
    Some(ObjectStore::String("a".to_owned())),
    "shift searchpaths"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS"),
    Some(ObjectStore::String("c".to_owned())),
    "pop searchpaths"
  );
  assert_eq!(
    state.pop_value("SEARCHPATHS"),
    Some(ObjectStore::String("b".to_owned())),
    "pop searchpaths"
  );
  assert_eq!(
    state.shift_value("SEARCHPATHS"),
    None,
    "shift searchpaths None"
  );
  assert_eq!(state.pop_value("SEARCHPATHS"), None, "pop searchpaths None");
  assert_eq!(
    state.lookup_value("SEARCHPATHS"),
    Some(&ObjectStore::VecDequeOS(VecDeque::new())),
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
    expansion: SimpleExpansion!(Tokens::new(Explode!("name"))),
    locator: "from unit test, line 99".to_owned(),
    is_protected: state.get_prefix("protected"),
    ..Expandable::default()
  };
  let job_definition_os = ObjectStore::Expandable(Rc::new(job_definition));
  // Install a Definition
  state.install_definition(job_definition_os.clone(), None);
  if let Some(ObjectStore::Expandable(stored_definition)) =
    state.lookup_definition(&T_CS!("\\jobname"))
  {
    assert_eq!(stored_definition.cs, T_CS!("\\jobname"));
  } else {
    panic!("Failed to lookup installed definition!");
  }

  // Assign a Meaning
  state.assign_meaning(&T_CS!("\\foobar"), job_definition_os, Some(Scope::Local));
  if let Some(&ObjectStore::Expandable(ref stored_meaning)) =
    state.lookup_meaning(&T_CS!("\\foobar"))
  {
    assert_eq!(stored_meaning.cs, T_CS!("\\jobname")); // Note: meaning for \foobar still has definition for CS \jobname
  } else {
    panic!("Failed to lookup installed meaning!");
  }

  let looked_up_meaning = { state.lookup_meaning(&T_CS!("\\foobar")).unwrap().clone() };
  {
    state.assign_meaning(
      &T_CS!("\\foolet"),
      looked_up_meaning.clone(),
      Some(Scope::Local),
    );
  }
  assert_eq!(
    state.lookup_meaning(&T_CS!("\\foolet")),
    Some(&looked_up_meaning),
    "Meanings match"
  );
}

#[test]
fn assign_lookup_mapping() {
  // # 10. assign Mapping
  // ok(!state.lookupMapping('TAG_PROPERTIES', "tag"), "lookupMapping is false on new keys");
  // state.assignMapping('TAG_PROPERTIES', "tag" => {});
  // my $props = state.lookupMapping('TAG_PROPERTIES', "tag");
  // is_deeply($props,{},"Empty mapping hash roundtrip");
  // my $undef_val = $$props{"afterOpen"};
  // assert_eq!($undef_val,undef,"Surviving a lookup of a new key");

  // my $wdr_url = "http://www.w3.org/2007/05/powder#";
  // state.assignMapping("RDFa_prefixes",
  //  "wdr"     => $wdr_url);
  // assert_eq!(state.lookupMapping("RDFa_prefixes","wdr"),$wdr_url,"asssign/lookupMapping
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
  //  state.assignMapping('RDFa_prefixes', $p => $rdf_prefixes{$p}); }
  // is_deeply(state.lookup_value('RDFa_prefixes'),\%rdf_prefixes,"Entire RDF mapping");
}

#[test]
fn push_pop_daemon_frames() {
  // TODO
  // state.assign_value("daemon_mode", ObjectStore::Bool(false), Some(Scope::Global));
  // state.push_daemon_frame();
  // state.assign_value("daemon_mode", ObjectStore::Bool(true),Some(Scope::Global));
  // match state.lookup_value("daemon_mode") {
  //   None => panic!("Couldn't lookup daemon_mode value after assignment"),
  //   Some(& ObjectStore::Bool(b)) => assert_eq!(b, true, "in daemon mode"),
  //   Some(_) => panic!("Looked up value of daemon_mode didn't match assignment value")
  // };

  // state.pop_daemon_frame();
  // match state.lookup_value("daemon_mode") {
  //   None => panic!("Couldn't lookup daemon_mode value after assignment"),
  //   Some(& ObjectStore::Bool(b)) => assert_eq!(b, false, "out of daemon mode"),
  //   Some(_) => panic!("Looked up value of daemon_mode didn't match assignment value")
  // };
}

#[test]
fn texy_ops() {
  // # 13. TeXy ops
  // my $mock1 = T_CS('\mock1');
  // my $mock2 = T_CS('\mock2');
  // my $mock3 = T_CS('\mock3');
  // state.pushValue('DOCUMENT_REWRITE_RULES',
  //     $mock1);
  // my @mocks = ($mock2,$mock3);
  // state.pushValue('DOCUMENT_REWRITE_RULES',@mocks);
  // assert_eq!(state.shift_value('DOCUMENT_REWRITE_RULES'),$mock1,"shift_value 1");
  // assert_eq!(state.shift_value('DOCUMENT_REWRITE_RULES'),$mock2,"shift_value 2");
  // assert_eq!(state.shift_value('DOCUMENT_REWRITE_RULES'),$mock3,"shift_value 3");

  // state.pushValue("PENDING_RESOURCES" => ["resource1", foo => 1, bar => 2]);
  // state.pushValue("PENDING_RESOURCES" => ["resource2", baz => 3, bam => 4]);
  // state.pushValue("PENDING_RESOURCES" => ["resource3", a => 5, b => 6]);
  // my $resources = state.lookup_value("PENDING_RESOURCES");
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

  state.end_semiverbatim();
}
