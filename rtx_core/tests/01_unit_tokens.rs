//======================================================================
// Unit tests for rtx
//======================================================================
// I'd like to have a directory of unit tests, so it can be more easily
// grown, but this will be a start
#[macro_use]
extern crate rtx_core;
use rtx_core::token::{Catcode, Token};
use std::borrow::Cow;

#[test]
fn unit_examples() {
  let mut letter = "x";
  let t = T_LETTER!(letter);
  // Basic Token() tests
  assert_eq!(t, T_LETTER!(letter), "Make Token (x)");
  assert_eq!(t.stringify(), "T_LETTER[x]", "Got correct token (x)");
  let tx = T_LETTER!(letter);
  letter = "z";
  assert_eq!(tx.stringify(), "T_LETTER[x]", "Got correct deref (x)");
  let tz = T_LETTER!(letter);
  assert_eq!(tz.stringify(), "T_LETTER[z]", "Got correct token (z)");

  // Basic Tokens() tests.
  let ts = Tokens!(t.clone());
  assert_eq!(ts.stringify(), "Tokens[x]", "Got correct token (x)");
  let ts3 = Tokens!(t.clone(), t.clone(), t.clone());
  assert_eq!(ts3.stringify(), "Tokens[x,x,x]", "Got correct tokens");
  let ts3x = Tokens!(t.clone(), t.clone(), t.clone());
  let t = T_LETTER!("z");
  assert_eq!(ts3x.stringify(), "Tokens[x,x,x]", "Got correct deref (x,x,x)");
  let ts3z = Tokens!(t.clone(), t.clone(), t.clone());
  assert_eq!(ts3z.stringify(), "Tokens[z,z,z]", "Got correct tokens (z,z,z)");
  let tss = Tokens!(ts3.unlist());
  assert_eq!(tss.stringify(), "Tokens[x,x,x]", "Got correct tokens (x,x,x)");

  // Balance Tokens tests
  let balanced = Tokens!(T_LETTER!("a"), T_BEGIN!(), T_OTHER!("..."), T_END!(), T_LETTER!("z"));
  assert!(balanced.is_balanced(), "Check is balanced");
  let unbalanced = Tokens!(T_LETTER!("a"), T_BEGIN!(), T_OTHER!("..."), T_LETTER!("z"));
  assert!(!unbalanced.is_balanced(), "Check is not balanced");

  // Macro arg substitution tests
  let nosubst = balanced.substitute_parameters(&[T_LETTER!("u").into(), T_LETTER!("v").into(), T_LETTER!("w").into()]);
  assert_eq!(nosubst.stringify(), "Tokens[a,{,...,},z]", "Got correct (non)substitution");
  let pattern = Tokens!(
    T_LETTER!("a"),
    T_BEGIN!(),
    T_PARAM!(),
    T_OTHER!("1"),
    T_LETTER!("m"),
    T_PARAM!(),
    T_OTHER!("2"),
    T_END!(),
    T_LETTER!("z")
  )
  .pack_parameters();
  let subst = pattern.substitute_parameters(&[T_LETTER!("u").into(), T_LETTER!("v").into(), T_LETTER!("w").into()]);
  assert_eq!(subst.stringify(), "Tokens[a,{,u,m,v,},z]", "Got correct substitution");
}
