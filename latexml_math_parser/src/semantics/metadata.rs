use std::{cmp::Ordering, error::Error, fmt, fmt::Display};

use super::curry::{CurryConstraint, CurryConstraints, CurryTerm};
use crate::util::distill_lexeme;

/// Discussion: The meta struct should be auto-derived from the grammar, and we should really be
/// talking about Box<MetaTrait> at this level.
/// To speed things up to an initial prototype, will hardcode some desired fields
///
/// Update: Now that we use the generalized curry approach, maybe we can indeed define a Meta
/// capable of servicing arbitrary field names.
#[derive(Debug, Clone, Default)]
pub struct Meta {
  pub syntax_trace:          Vec<String>,
  pub fenced:                Option<String>,
  pub specialize:            Option<String>,
  pub curry_level:           Option<CurryTerm>,
  pub curry_constraints:     CurryConstraints,
  /// Perl: _bumplevel — tracks nested float script level for proper scriptpos indexing
  bumplevel:                 u32,
  /// Perl: _wasfloat — marks XMApp as result of a float (prescript) script
  wasfloat:                  bool,
  /// A differential `d x` (`diffop_apply`): Perl's `IntFactor` `diffd` form (MathGrammar:643-646)
  /// is a finished factor, not a big operator whose operand the factors after it join — though
  /// the `d` carries the DIFFOP role; and a differential operator's application to its one factor
  /// (`differential_operator_apply`: `\partial_x u`, iopart's `\rmd x`; divergence #374), which Perl's
  /// `bigop` reads greedily (MathGrammar:717).
  pub differential:          bool,
  /// An absolute value or norm between single bars whose content holds an evaluation bar
  /// (`fenced`, `norm_fenced`), counted by `XM::prefer_fewest_evaluation_bars_inside` (divergence
  /// #350).
  pub evaluation_bar_inside: bool,
  /// An absolute value or norm between VERTBAR tokens — `|…|`, `\|…\|`, `\left|…\right|`,
  /// `\left\|…\right\|` (Perl re-roles a `\left` ∥ a VERTBAR, TeX_Math.pool.ltxml:755), and
  /// `||…||` (divergence #353) — Perl's `VERTBAR absExpression VERTBAR` (MathGrammar:329-330), a
  /// bare item of a function's argument, never the group of its easy arguments. Marked by the fence:
  /// bars read through XMRefs in a content branch (`split`) carry no content or name to tell them
  /// by. A bare `\lvert…\rvert`, `\lVert…\rVert` are OPEN/CLOSE delimiters, groups as in Perl.
  pub bar_fence:             bool,
  /// A `bar_fence` between two single `|` (VERTBAR) tokens.
  pub single_bar_pair:       bool,
  /// A fence whose close does not balance its open — `(0,1]`, `[a,b)` (`interval`): no function's
  /// argument, as Perl's `addEasyArgs` needs a `balancedClose` (MathGrammar:571-576).
  pub unbalanced_fence:      bool,
  /// A `bar_fence`'s nesting: 1, and one more for each bar fence it holds — Perl's `absExpression`
  /// depth (MathGrammar:410-412), which a parse may not exceed (`semantics::max_abs_depth`).
  pub abs_depth:             u8,
  /// A comma formulae that `separate_ellipsis_items` gave a plain ellipsis item: a list once the
  /// enumerations are attached if the ellipsis stays a bare item, as its comma'd twin reads
  /// (`semantics::name_an_elided_formulae`, 57bv.1).
  pub elided_formulae:       bool,
}

impl Meta {
  /// The metadata of a differential's application (see [`Meta::differential`]).
  pub fn for_differential() -> Self {
    Meta {
      differential: true,
      ..Meta::default()
    }
  }
}

impl PartialEq for Meta {
  fn eq(&self, _other: &Self) -> bool {
    true // we won't compare metadata for the moment, skewed towards "it's all good"
  }
}
impl Eq for Meta {}
impl Display for Meta {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // Keep output terse — callers that need full detail should Debug-print.
    write!(f, "Meta(bump={},", self.bumplevel)?;
    if self.wasfloat {
      write!(f, "float")?;
    }
    write!(f, ")")
  }
}

impl Meta {
  pub fn with_bumplevel(level: u32) -> Self {
    Meta {
      bumplevel: level,
      ..Meta::default()
    }
  }
  pub fn bumplevel(&self) -> u32 { self.bumplevel }
  pub fn wasfloat(&self) -> bool { self.wasfloat }
  pub fn set_wasfloat(&mut self) { self.wasfloat = true; }

  /// Instatiate a default Meta object for a given rule name
  pub fn from_rule(rule_name: &str) -> Self {
    if rule_name.is_empty() {
      Meta::default()
    } else {
      Meta {
        syntax_trace: vec![rule_name.to_owned()],
        ..Meta::default()
      }
    }
  }

  // /// Provides tuples of field key values, intended for a display-level serialization
  // pub(crate) fn display_fields(&self) -> Vec<(&str, String)> {
  //   let mut fields = Vec::new();
  //   if let Some(ref level) = self.curry_level {
  //     fields.push(("curry", level.to_string()));
  //   }
  //   if !self.curry_constraints.is_empty() {
  //     let displayed_constraints = format!(
  //       "[{}]",
  //       self.curry_constraints.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")
  //     );
  //     fields.push(("where", displayed_constraints));
  //   }
  //   if let Some(ref kind) = self.fenced {
  //     fields.push(("fenced", kind.to_owned()));
  //   }
  //   fields
  // }

  pub fn can_specialize(&self) -> bool {
    // does the meta object have any fields that are meaningful during specialization? Namely:
    self.fenced.is_some()
      || self.curry_level.is_some()
      || !self.syntax_trace.is_empty()
      || !self.curry_constraints.is_empty()
  }

  /// Check if no type fields are set, as would be the default case
  pub fn is_empty(&self) -> bool {
    self.fenced.is_none() && self.curry_level.is_none() && self.curry_constraints.is_empty()
  }

  /// Override all fields of meta with the nonempty fields of the incoming data
  /// Special features:
  /// - fenced:* can be used to invalidate `self` if it has no fenced attribute set
  /// - fenced.overaccent will rename a single Var curry level to an embellished variant
  pub fn with(self, mut other: Meta) -> Result<Self, Box<dyn Error>> {
    // EMBELLISHED specialization
    // overaccents lead to renaming of all pieces
    let specialize = if other.specialize.is_some() {
      other.specialize
    } else {
      self.specialize
    };

    // TODO: We would need a smart tracking algorithm to extend the full trace
    // for now keep the last step.
    //self.syntax_trace.extend(other.syntax_trace.into_iter());
    let syntax_trace = if other.syntax_trace.is_empty() {
      self.syntax_trace
    } else {
      other.syntax_trace
    };

    let fenced = if let Some(ref fenced_kind) = other.fenced {
      // special case, the star requires some existing value, or we prune
      if fenced_kind == "*" {
        if self.fenced.is_none() {
          return Err("fenced.* requirement failed".into());
        } else {
          self.fenced
        }
      } else {
        other.fenced
      }
    } else {
      self.fenced
    };
    let mut curry_constraints = self.curry_constraints;
    for other_constraint in other.curry_constraints.drain() {
      curry_constraints.insert(other_constraint);
    }

    // Incoming levels are to be seen as constraints on the current level.
    // When no current level has been set, the incoming level is used as current with a >= 1
    // constraint.
    let curry_level = if let Some(current_level) = self.curry_level {
      if let Some(new_level) = other.curry_level {
        if current_level != new_level {
          curry_constraints.insert(CurryConstraint((
            current_level,
            Ordering::Equal,
            new_level.clone(),
          )));
        }
        Some(new_level)
      } else {
        Some(current_level)
      }
    } else {
      other.curry_level
    };

    Ok(Meta {
      syntax_trace,
      fenced,
      specialize,
      curry_level,
      curry_constraints,
      // Preserve sticky flags from either side
      bumplevel: self.bumplevel.max(other.bumplevel),
      wasfloat: self.wasfloat || other.wasfloat,
      // Belong to the node they mark, not to what is built from it.
      differential: false,
      evaluation_bar_inside: false,
      bar_fence: false,
      single_bar_pair: false,
      unbalanced_fence: false,
      abs_depth: 0,
      elided_formulae: false,
    })
  }

  /// Validate a metadata object, by checking its linear programming problem
  /// is solveable.
  pub fn validate(&self) -> Result<(), Box<dyn Error>> {
    // TODO: Reintroduce when we get lp solving reintegrated
    // if let Some(ref level) = self.curry_level {
    //   solve_lp_problem(level, &self.curry_constraints)
    // } else {
    //   Ok(())
    // }
    Ok(())
  }

  /// Specializing with a metadata object over a curry atom, implies we can
  /// substitute the curry level with the name of the leaf node, and
  /// instead create a constraint
  pub fn with_curry_atom(self, mut into: Meta, name: &str) -> Result<Self, Box<dyn Error>> {
    // massage the name so that cbc solver can handle it without weird glitches.
    let lex_parts = distill_lexeme(name);
    let curry_var = CurryTerm::Var(format!(":{}", lex_parts.2));
    // We don't have to build this in, since by default we will enforce all variables are in a fixed
    // curry range of [1,10]. into.curry_constraints.insert(CurryConstraint((
    //   curry_var.clone(),
    //   Ordering::Greater,
    //   CurryTerm::Literal(0),
    // )));

    if let Some(ref level) = into.curry_level
      && &curry_var != level
    {
      into.curry_constraints.insert(CurryConstraint((
        curry_var.clone(),
        Ordering::Equal,
        level.clone(),
      )));
    }
    into.curry_level = Some(curry_var);
    self.with(into)
  }
}
