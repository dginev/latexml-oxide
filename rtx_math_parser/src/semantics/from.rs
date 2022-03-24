use super::metadata::Meta;
use super::tree::{Args, Operator, Tree};

impl From<&str> for Operator {
  fn from(a: &str) -> Operator { Operator(Box::new(a.into())) }
}
impl From<String> for Operator {
  fn from(a: String) -> Operator { Operator(Box::new(a.as_str().into())) }
}
impl From<Option<Tree>> for Operator {
  fn from(opt: Option<Tree>) -> Operator {
    if let Some(tree) = opt {
      Operator(Box::new(tree))
    } else {
      "missing_operator".into()
    }
  }
}
impl From<Option<Tree>> for Args {
  fn from(opt: Option<Tree>) -> Args { Args(vec![opt]) }
}

impl From<&str> for Tree {
  fn from(a: &str) -> Tree { Tree::Lexeme(a.to_string(), Meta::default()) }
}
impl From<(&str, &str)> for Tree {
  fn from(a: (&str, &str)) -> Tree { Tree::Apply(a.0.into(), Args(vec![Some(a.1.into())]), Meta::default()) }
}
impl From<(&str, (&str, &str))> for Tree {
  fn from(args: (&str, (&str, &str))) -> Tree { Tree::Apply(args.0.into(), args.1.into(), Meta::default()) }
}

impl From<(&str, (&str, (&str, &str)))> for Tree {
  fn from(args: (&str, (&str, (&str, &str)))) -> Tree { Tree::Apply(args.0.into(), Args(vec![Some(args.1.into())]), Meta::default()) }
}
// One element arrays as argument containers (since we can't do one element tuple types?)
impl From<(&str, (&str, (&str, [&str; 1])))> for Tree {
  fn from(args: (&str, (&str, (&str, [&str; 1])))) -> Tree { Tree::Apply(args.0.into(), Args(vec![Some(args.1.into())]), Meta::default()) }
}
impl From<(&str, (&str, [&str; 1]))> for Tree {
  fn from(args: (&str, (&str, [&str; 1]))) -> Tree { Tree::Apply(args.0.into(), Args(vec![Some(args.1.into())]), Meta::default()) }
}
impl From<(&str, [&str; 1])> for Tree {
  fn from(args: (&str, [&str; 1])) -> Tree { Tree::Apply(args.0.into(), Args(vec![Some(args.1[0].into())]), Meta::default()) }
}

impl From<[&str; 1]> for Args {
  fn from(args: [&str; 1]) -> Args { Args(args.iter().map(|&x| x.into()).map(Option::Some).collect()) }
}
impl<OP: ToString + Sized, LEFT: Into<Tree>, RIGHT: Into<Tree>> From<(OP, LEFT, RIGHT)> for Tree {
  fn from(args: (OP, LEFT, RIGHT)) -> Tree {
    Tree::Apply(
      args.0.to_string().into(),
      Args(vec![Some(args.1.into()), Some(args.2.into())]),
      Meta::default(),
    )
  }
}
impl<OP: ToString + Sized> From<(OP, Vec<Tree>)> for Tree {
  fn from(args: (OP, Vec<Tree>)) -> Tree {
    if args.0.to_string() == "choices" {
      Tree::Choices(args.1.to_vec())
    } else {
      Tree::Apply(
        args.0.to_string().into(),
        Args(args.1.iter().cloned().map(Some).collect()),
        Meta::default(),
      )
    }
  }
}

impl From<(&str, &str)> for Args {
  fn from(args: (&str, &str)) -> Args { Args([args.0, args.1].iter().map(|&x| x.into()).map(Option::Some).collect()) }
}
impl From<[&str; 2]> for Args {
  fn from(args: [&str; 2]) -> Args {
    Args(
      args
        .iter()
        .map(ToString::to_string)
        .map(|x| Tree::Lexeme(x, Meta::default()))
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
        .map(ToString::to_string)
        .map(|x| Tree::Lexeme(x, Meta::default()))
        .map(Option::Some)
        .collect(),
    )
  }
}
