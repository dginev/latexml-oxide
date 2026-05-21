use super::metadata::Meta;
use super::tree::{Args, Operator, XM, XProps};
use libxml::tree::Node;

impl From<&str> for Operator {
  fn from(a: &str) -> Operator { Operator(Box::new(a.into())) }
}
impl From<String> for Operator {
  fn from(a: String) -> Operator { Operator(Box::new(a.as_str().into())) }
}
impl From<Option<XM>> for Operator {
  fn from(opt: Option<XM>) -> Operator {
    if let Some(tree) = opt {
      Operator(Box::new(tree))
    } else {
      "missing_operator".into()
    }
  }
}
impl From<&Node> for Operator {
  fn from(node: &Node) -> Operator {
    let xmnode: XM = node.into();
    Operator(Box::new(xmnode))
  }
}
impl From<Option<XM>> for Args {
  fn from(opt: Option<XM>) -> Args { Args(vec![opt]) }
}

impl From<&str> for XM {
  fn from(a: &str) -> XM { XM::Lexeme(std::rc::Rc::from(a), Meta::default()) }
}
impl From<(&str, &str)> for XM {
  fn from(a: (&str, &str)) -> XM {
    XM::Apply(
      a.0.into(),
      Args(vec![Some(a.1.into())]),
      XProps::default(),
      Meta::default(),
    )
  }
}
impl From<(&str, (&str, &str))> for XM {
  fn from(args: (&str, (&str, &str))) -> XM {
    XM::Apply(
      args.0.into(),
      args.1.into(),
      XProps::default(),
      Meta::default(),
    )
  }
}

impl From<(&str, (&str, (&str, &str)))> for XM {
  fn from(args: (&str, (&str, (&str, &str)))) -> XM {
    XM::Apply(
      args.0.into(),
      Args(vec![Some(args.1.into())]),
      XProps::default(),
      Meta::default(),
    )
  }
}
// One element arrays as argument containers (since we can't do one element tuple types?)
impl From<(&str, (&str, (&str, [&str; 1])))> for XM {
  fn from(args: (&str, (&str, (&str, [&str; 1])))) -> XM {
    XM::Apply(
      args.0.into(),
      Args(vec![Some(args.1.into())]),
      XProps::default(),
      Meta::default(),
    )
  }
}
impl From<(&str, (&str, [&str; 1]))> for XM {
  fn from(args: (&str, (&str, [&str; 1]))) -> XM {
    XM::Apply(
      args.0.into(),
      Args(vec![Some(args.1.into())]),
      XProps::default(),
      Meta::default(),
    )
  }
}
impl From<(&str, [&str; 1])> for XM {
  fn from(args: (&str, [&str; 1])) -> XM {
    XM::Apply(
      args.0.into(),
      Args(vec![Some(args.1[0].into())]),
      XProps::default(),
      Meta::default(),
    )
  }
}

impl From<[&str; 1]> for Args {
  fn from(args: [&str; 1]) -> Args {
    Args(args.iter().map(|&x| x.into()).map(Option::Some).collect())
  }
}
impl<OP: ToString + Sized, LEFT: Into<XM>, RIGHT: Into<XM>> From<(OP, LEFT, RIGHT)> for XM {
  fn from(args: (OP, LEFT, RIGHT)) -> XM {
    XM::Apply(
      args.0.to_string().into(),
      Args(vec![Some(args.1.into()), Some(args.2.into())]),
      XProps::default(),
      Meta::default(),
    )
  }
}
impl<OP: ToString + Sized> From<(OP, Vec<XM>)> for XM {
  fn from(args: (OP, Vec<XM>)) -> XM {
    let op_str = args.0.to_string();
    if op_str == "choices" {
      XM::Choices(args.1.to_vec())
    } else {
      XM::Apply(
        op_str.into(),
        Args(args.1.iter().cloned().map(Some).collect()),
        XProps::default(),
        Meta::default(),
      )
    }
  }
}

impl From<(&str, &str)> for Args {
  fn from(args: (&str, &str)) -> Args {
    Args(
      <[&str; 2]>::from(args)
        .iter()
        .map(|&x| x.into())
        .map(Option::Some)
        .collect(),
    )
  }
}
impl From<[&str; 2]> for Args {
  fn from(args: [&str; 2]) -> Args {
    Args(
      args
        .iter()
        .map(|s| XM::Lexeme(std::rc::Rc::from(*s), Meta::default()))
        .map(Option::Some)
        .collect(),
    )
  }
}
impl From<[&str; 3]> for Args {
  fn from(args: [&str; 3]) -> Args {
    Args(
      args
        .iter()
        .map(|s| XM::Lexeme(std::rc::Rc::from(*s), Meta::default()))
        .map(Option::Some)
        .collect(),
    )
  }
}
