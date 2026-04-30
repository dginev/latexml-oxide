// use crate::argument::Argument;
// use minilp::{ComparisonOp, Variable};
// use quote::ToTokens;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::cmp::Ordering;
use std::fmt::{self, Display};

/// A CurryConstraint is a simple linear constraint between named variables and literals
/// e.g. "x-y == 1", "x>=0", "y>=1", and so forth
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CurryTerm {
  Literal(i8),
  Var(String),
  Arg(usize),
  Sub(Box<CurryTerm>, Box<CurryTerm>),
  Add(Box<CurryTerm>, Box<CurryTerm>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurryConstraint(pub (CurryTerm, Ordering, CurryTerm));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurryConstraints(pub HashSet<CurryConstraint>);

impl Display for CurryTerm {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CurryTerm::Arg(v) => write!(f, "arg{v}"),
      CurryTerm::Var(v) => write!(f, "{v}"),
      CurryTerm::Literal(u) => write!(f, "{u}"),
      CurryTerm::Sub(x, y) => match &**y {
        CurryTerm::Sub(..) | CurryTerm::Add(..) => write!(f, "{x}-({y})"),
        _ => write!(f, "{x}-{y}"),
      },
      CurryTerm::Add(x, y) => write!(f, "{x}+{y}"),
    }
  }
}
impl Display for CurryConstraint {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let (lhs, cmp, rhs) = &self.0;
    let cmp_str = match cmp {
      Ordering::Equal => "=",
      Ordering::Less => "<",
      Ordering::Greater => ">",
    };
    write!(f, "{lhs}{cmp_str}{rhs}")
  }
}

// The ToTokens implemention here is special, as it switches from a Spec
// to a concrete LpExpression representation
// impl ToTokens for CurryTerm {
//   fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
//     stream.extend(match self {
//       CurryTerm::Literal(v) => quote!(CurryTerm::Literal(#v)),
//       CurryTerm::Var(v) => quote!(CurryTerm::Var(#v)),
//       CurryTerm::Arg(v) => quote!(
//           args[#v].as_ref().unwrap()
//             .get_meta().curry_level
//             .as_ref().expect(&format!("Failed to resolve the curry_level of an argument at {:?}",
// args)).clone()),       CurryTerm::Sub(lhs, rhs) => quote!(CurryTerm::Sub(Box::new(#lhs),
// Box::new(#rhs))),       CurryTerm::Add(lhs, rhs) => quote!(CurryTerm::Add(Box::new(#lhs),
// Box::new(#rhs))),     });
//   }
// }

impl CurryTerm {
  /// Fill in the argument id specified in the notation, e.g. #2
  /// with the actual array index for the lexeme
  /// e.g. for "{#1|#2}", the lexeme index corresponding to #1 is 1, to #2 is 3
  // pub fn fill_arguments(&self, args: &[Argument]) -> Self {
  //   match self {
  //     CurryTerm::Literal(v) => CurryTerm::Literal(*v),
  //     CurryTerm::Var(v) => CurryTerm::Var(v.clone()),
  //     CurryTerm::Sub(lhs, rhs) => CurryTerm::Sub(Box::new(lhs.fill_arguments(args)),
  // Box::new(rhs.fill_arguments(args))),     CurryTerm::Add(lhs, rhs) =>
  // CurryTerm::Add(Box::new(lhs.fill_arguments(args)), Box::new(rhs.fill_arguments(args))),
  //     CurryTerm::Arg(id) => {
  //       let arg = args
  //         .iter()
  //         .find(|arg| arg.id == *id)
  //         .expect("Bad specification for #arg, unknown argument id!");
  //       CurryTerm::Arg(arg.pattern_index)
  //     },
  //   }
  // }
  /// Simplify some obvious tautologies
  pub fn simplify(self) -> Self {
    use CurryTerm::*;
    match self {
      Literal(_) | Arg(_) | Var(_) => self,
      Sub(lhs, rhs) => {
        let lhs = lhs.simplify();
        let rhs = rhs.simplify();
        if lhs == rhs {
          Literal(0)
        } else if rhs == Literal(0) {
          lhs
        } else {
          Sub(lhs.into(), rhs.into())
        }
      },
      Add(lhs, rhs) => {
        let lhs = lhs.simplify();
        let rhs = rhs.simplify();
        let mut simpler = None;
        // neutral element
        if lhs == Literal(0) {
          simpler = Some(rhs.clone());
        }
        if rhs == Literal(0) {
          simpler = Some(lhs.clone());
        }
        if let Sub(ref slhs, ref srhs) = lhs {
          if **srhs == rhs {
            simpler = Some(*slhs.clone());
          }
        }
        if let Sub(ref slhs, ref srhs) = rhs {
          if **srhs == lhs {
            simpler = Some(*slhs.clone());
          }
        }
        match simpler {
          Some(new) => new,
          None => Add(lhs.into(), rhs.into()),
        }
      },
    }
  }

  pub fn to_minilp<'a>(
    &'a self,
    var_values: &mut HashMap<&'a String, f64>,
    rhs_minilp: &mut f64,
    negate: bool,
  ) {
    use CurryTerm::*;
    match self {
      Literal(v) => {
        if negate {
          // -literal on LHS, becomes +literal on RHS
          *rhs_minilp += f64::from(*v)
        } else {
          // +literal on LHS, becomes -literal on RHS
          *rhs_minilp -= f64::from(*v)
        }
      },
      Arg(_) => {
        panic!("a curry term argument was not filled in at to_minilp, this should never happen!")
      },
      Var(x) => {
        let x_val = var_values.entry(x).or_insert(0.0);
        if negate {
          *x_val -= 1.0;
        } else {
          *x_val += 1.0;
        }
      },
      Sub(slhs, srhs) => {
        slhs.to_minilp(var_values, rhs_minilp, negate);
        srhs.to_minilp(var_values, rhs_minilp, !negate);
      },
      Add(slhs, srhs) => {
        slhs.to_minilp(var_values, rhs_minilp, negate);
        srhs.to_minilp(var_values, rhs_minilp, negate);
      },
    }
  }
}

impl CurryConstraint {
  /// rewrite/normalize the expression as closer to minilp solving
  /// 1. Reduce easy tautological cases.
  /// 2. Move all free literals to the RHS
  /// 3. Move all variable expressions to the LHS
  pub fn simplify(&self) -> Self {
    use CurryTerm::*;
    // destructure
    let CurryConstraint((lhs, cmp, rhs)) = self;
    let new_rhs;
    let new_lhs;
    let mut new_cmp = *cmp;
    match rhs {
      Literal(ref v) if *v > 0 => match lhs {
        Literal(lv) => {
          new_lhs = Literal(lv - v);
          new_rhs = Literal(0);
        },
        Var(_) => {
          new_lhs = lhs.clone();
          new_rhs = rhs.clone();
        },
        Arg(_) => panic!("Tried to solve a problem that has not filled in its arguments!"),
        Sub(slhs, srhs) => {
          if let Literal(srhsv) = **srhs {
            new_lhs = (**slhs).clone().simplify();
            new_rhs = Literal(v + srhsv);
          } else if let Literal(slhsv) = **slhs {
            new_lhs = (**srhs).clone().simplify();
            new_rhs = Literal(slhsv - v);
            // flip the operation
            new_cmp = match new_cmp {
              Ordering::Equal => Ordering::Equal,
              Ordering::Greater => Ordering::Less,
              Ordering::Less => Ordering::Greater,
            };
          } else {
            // compound subtraction, leave it be for now
            new_lhs = lhs.clone();
            new_rhs = rhs.clone();
          }
        },
        Add(slhs, srhs) => {
          if let Literal(ref srhsv) = **srhs {
            new_lhs = (**slhs).clone();
            new_rhs = Literal(v - srhsv);
          } else if let Literal(ref slhsv) = **slhs {
            new_lhs = (**srhs).clone();
            new_rhs = Literal(v - slhsv);
          } else {
            // compound addition, leave it be for now
            new_lhs = lhs.clone();
            new_rhs = rhs.clone();
          }
        },
      },
      Literal(_) => {
        new_lhs = lhs.clone();
        new_rhs = rhs.clone();
      },
      Arg(_) => {
        panic!("Tried to solve a problem that has not filled in its arguments!")
      },
      Sub(sub_left, sub_right) => {
        new_lhs = Sub(
          Add(lhs.clone().into(), sub_right.clone()).into(),
          sub_left.clone(),
        )
        .simplify();
        new_rhs = Literal(0);
      },
      Add(..) => {
        new_lhs = Sub(lhs.clone().into(), rhs.clone().into()).simplify();
        new_rhs = Literal(0);
      },
      Var(x) => {
        new_lhs = Sub(lhs.clone().into(), Var(x.to_owned()).into()).simplify();
        new_rhs = Literal(0);
      },
    }
    CurryConstraint((new_lhs, new_cmp, new_rhs))
  }

  // pub fn to_minilp(&self, varmap: &HashMap<&String, Variable>) -> (Vec<(Variable, f64)>,
  // ComparisonOp, f64) {   let (lhs, cmp, rhs) = &self.0;
  //   let mut rhs_minilp = if let CurryTerm::Literal(v) = rhs {
  //     (*v).into()
  //   } else {
  //     panic!("RHS of a CurryConstraint should be a literal by the time it is solved (i.e. mapped
  // to minilp)")   };
  //   // A bit of silly adjustments here, as minilp only has the "-or-equal" variants, while the
  // rust "Ordering" only has the strict relations.   let cmp_lp = match cmp {
  //     Ordering::Less => {
  //       rhs_minilp -= 1.0;
  //       ComparisonOp::Le
  //     },
  //     Ordering::Greater => {
  //       rhs_minilp += 1.0;
  //       ComparisonOp::Ge
  //     },
  //     Ordering::Equal => ComparisonOp::Eq,
  //   };
  //   let mut var_values: HashMap<&String, f64> = HashMap::default();

  //   lhs.to_minilp(&mut var_values, &mut rhs_minilp, false);

  //   let lhs_minilp: Vec<(Variable, f64)> = var_values.into_iter().map(|(var, val)|
  // (*varmap.get(&var).unwrap(), val)).collect();   (lhs_minilp, cmp_lp, rhs_minilp)
  // }
}

impl Default for CurryConstraints {
  fn default() -> Self { Self::new() }
}

impl CurryConstraints {
  pub fn new() -> Self { CurryConstraints(HashSet::default()) }
  pub fn insert(&mut self, value: CurryConstraint) -> bool { self.0.insert(value.simplify()) }
  pub fn iter(&self) -> std::collections::hash_set::Iter<'_, CurryConstraint> { self.0.iter() }

  pub fn drain(&mut self) -> std::collections::hash_set::Drain<'_, CurryConstraint> {
    self.0.drain()
  }

  pub fn is_empty(&self) -> bool { self.0.is_empty() }

  pub fn len(&self) -> usize { self.0.len() }
}
