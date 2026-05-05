use crate::prelude::*;
use latexml_core::common::numeric_ops::{NumericOps, UNITY, kround};
use latexml_core::definition::register::RegisterValue;

// calc.sty expression parser — ported from Perl calc.sty.ltxml
// Grammar:
//   <expression> -> <term> ((+|-) <term>)*
//   <term>       -> <value> ((*|/) <value>)*  (where multiply/divide factor is float-aware)
//   <value>      -> <literal> | \real{...} | \ratio{...}{...}
//                 | \minof{...}{...} | \maxof{...}{...}
//                 | \widthof{...} | ( <expression> )

/// Internal representation for calc values: either an integer-based RegisterValue
/// or a floating-point factor (from \real or \ratio).
enum CalcValue {
  Reg(RegisterValue),
  Flt(f64),
}

impl CalcValue {
  fn into_regval(self, expr_type: &str) -> RegisterValue {
    match self {
      CalcValue::Reg(rv) => rv,
      CalcValue::Flt(f) => {
        if expr_type == "Number" {
          RegisterValue::Number(Number::new(f as i64))
        } else {
          RegisterValue::Glue(Glue::new(f as i64))
        }
      },
    }
  }
}

fn read_expression(expr_type: &str, tokens: Tokens) -> Result<RegisterValue> {
  let reader_mouth = Mouth::new("", None)?;
  gullet::reading_from_mouth(reader_mouth, move || {
    gullet::unread(tokens);
    let mut result = read_term(expr_type)?;
    while let Some(op) = gullet::read_keyword(&["+", "-"])? {
      let term2 = read_term(expr_type)?;
      result = if op == "+" {
        result.add(term2)
      } else {
        result.subtract(term2)
      };
    }
    Ok(result)
  })
}

fn read_term(expr_type: &str) -> Result<RegisterValue> {
  gullet::skip_spaces()?;
  let mut factor = read_value(expr_type)?.into_regval(expr_type);
  while let Some(op) = gullet::read_keyword(&["*", "/"])? {
    let factor2 = read_value("Number")?;
    factor = match factor2 {
      CalcValue::Flt(f) => apply_float_op(&factor, &op, f),
      CalcValue::Reg(rv) => {
        if op == "*" {
          factor.multiply(rv)
        } else {
          factor.divide(rv)
        }
      },
    };
  }
  Ok(factor)
}

/// Apply a float multiply/divide to a RegisterValue, preserving Glue components
fn apply_float_op(val: &RegisterValue, op: &str, f: f64) -> RegisterValue {
  match val {
    RegisterValue::Glue(g) => {
      if op == "*" {
        RegisterValue::Glue(Glue {
          skip:  (g.skip as f64 * f) as i64,
          plus:  g.plus.map(|p| (p as f64 * f) as i64),
          pfill: g.pfill,
          minus: g.minus.map(|m| (m as f64 * f) as i64),
          mfill: g.mfill,
        })
      } else {
        let div = if f == 0.0 { 1.0 } else { f };
        RegisterValue::Glue(Glue {
          skip:  (g.skip as f64 / div) as i64,
          plus:  g.plus.map(|p| (p as f64 / div) as i64),
          pfill: g.pfill,
          minus: g.minus.map(|m| (m as f64 / div) as i64),
          mfill: g.mfill,
        })
      }
    },
    RegisterValue::Number(n) => {
      let v = n.value_of() as f64;
      if op == "*" {
        RegisterValue::Number(Number::new((v * f) as i64))
      } else {
        let div = if f == 0.0 { 1.0 } else { f };
        RegisterValue::Number(Number::new((v / div) as i64))
      }
    },
    RegisterValue::Dimension(d) => {
      let v = d.value_of() as f64;
      if op == "*" {
        RegisterValue::Dimension(Dimension::new((v * f) as i64))
      } else {
        let div = if f == 0.0 { 1.0 } else { f };
        RegisterValue::Dimension(Dimension::new((v / div) as i64))
      }
    },
    _ => val.clone(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use latexml_core::common::glue::Glue;
  use latexml_core::common::numeric_ops::NumericOps;

  #[test]
  fn apply_float_op_number_multiply() {
    let n = RegisterValue::Number(Number::new(10));
    let result = apply_float_op(&n, "*", 2.5);
    match result {
      RegisterValue::Number(v) => assert_eq!(v.value_of(), 25),
      other => panic!("expected Number, got {other:?}"),
    }
  }

  #[test]
  fn apply_float_op_number_divide() {
    let n = RegisterValue::Number(Number::new(100));
    let result = apply_float_op(&n, "/", 4.0);
    match result {
      RegisterValue::Number(v) => assert_eq!(v.value_of(), 25),
      other => panic!("expected Number, got {other:?}"),
    }
  }

  #[test]
  fn apply_float_op_divide_by_zero_is_identity_divisor() {
    // Guard: division by zero uses divisor=1 → returns the original value.
    let n = RegisterValue::Number(Number::new(42));
    let result = apply_float_op(&n, "/", 0.0);
    match result {
      RegisterValue::Number(v) => assert_eq!(v.value_of(), 42),
      other => panic!("expected Number, got {other:?}"),
    }
  }

  #[test]
  fn apply_float_op_dimension_multiply() {
    let d = RegisterValue::Dimension(Dimension::new(1000));
    let result = apply_float_op(&d, "*", 0.5);
    match result {
      RegisterValue::Dimension(v) => assert_eq!(v.value_of(), 500),
      other => panic!("expected Dimension, got {other:?}"),
    }
  }

  #[test]
  fn apply_float_op_glue_multiply_scales_all_components() {
    let g = RegisterValue::Glue(Glue {
      skip:  100,
      plus:  Some(10),
      pfill: None,
      minus: Some(5),
      mfill: None,
    });
    let result = apply_float_op(&g, "*", 3.0);
    match result {
      RegisterValue::Glue(v) => {
        assert_eq!(v.skip, 300);
        assert_eq!(v.plus, Some(30));
        assert_eq!(v.minus, Some(15));
      },
      other => panic!("expected Glue, got {other:?}"),
    }
  }

  #[test]
  fn apply_float_op_glue_divide_scales_all_components() {
    let g = RegisterValue::Glue(Glue {
      skip:  600,
      plus:  Some(60),
      pfill: None,
      minus: Some(30),
      mfill: None,
    });
    let result = apply_float_op(&g, "/", 2.0);
    match result {
      RegisterValue::Glue(v) => {
        assert_eq!(v.skip, 300);
        assert_eq!(v.plus, Some(30));
        assert_eq!(v.minus, Some(15));
      },
      other => panic!("expected Glue, got {other:?}"),
    }
  }

  #[test]
  fn apply_float_op_glue_preserves_none_components() {
    let g = RegisterValue::Glue(Glue {
      skip:  100,
      plus:  None,
      pfill: None,
      minus: None,
      mfill: None,
    });
    let result = apply_float_op(&g, "*", 2.0);
    match result {
      RegisterValue::Glue(v) => {
        assert_eq!(v.skip, 200);
        assert!(v.plus.is_none());
        assert!(v.minus.is_none());
      },
      other => panic!("expected Glue, got {other:?}"),
    }
  }

  #[test]
  fn apply_float_op_unsupported_variant_is_identity() {
    // Anything not Number / Dimension / Glue hits the `_ => val.clone()` arm.
    let muglue = RegisterValue::MuGlue(Default::default());
    let result = apply_float_op(&muglue, "*", 3.125);
    // Muglue clones through untouched.
    assert_eq!(result, muglue);
  }
}

fn read_value(expr_type: &str) -> Result<CalcValue> {
  gullet::skip_spaces()?;
  let peek = gullet::read_x_token(None, false, None)?;
  let peek = match peek {
    Some(t) => t,
    None => {
      if expr_type == "Number" {
        return Ok(CalcValue::Reg(RegisterValue::Number(Number::new(0))));
      } else {
        return Ok(CalcValue::Reg(RegisterValue::Glue(Glue::new(0))));
      }
    },
  };
  // \widthof{...} — Perl calc.sty.ltxml L139-143
  if peek == T_CS!("\\widthof") {
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let box_result = digest(arg)?;
    if expr_type == "Number" {
      Error!(
        "unexpected",
        "\\widthof",
        format!("\\widthof not expected here (reading {expr_type})")
      );
    }
    let width = box_result
      .get_width(None)?
      .unwrap_or(RegisterValue::Dimension(Dimension::new(0)));
    return Ok(CalcValue::Reg(width));
  }
  // \heightof{...} — Perl calc.sty.ltxml L144-148
  if peek == T_CS!("\\heightof") {
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let box_result = digest(arg)?;
    if expr_type == "Number" {
      Error!(
        "unexpected",
        "\\heightof",
        format!("\\heightof not expected here (reading {expr_type})")
      );
    }
    let height = box_result
      .get_height()
      .unwrap_or(RegisterValue::Dimension(Dimension::new(0)));
    return Ok(CalcValue::Reg(height));
  }
  // \depthof{...} — Perl calc.sty.ltxml L149-153
  if peek == T_CS!("\\depthof") {
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let box_result = digest(arg)?;
    if expr_type == "Number" {
      Error!(
        "unexpected",
        "\\depthof",
        format!("\\depthof not expected here (reading {expr_type})")
      );
    }
    let depth = box_result
      .get_depth()
      .unwrap_or(RegisterValue::Dimension(Dimension::new(0)));
    return Ok(CalcValue::Reg(depth));
  }
  // \totalheightof{...} — Perl calc.sty.ltxml L154-158
  if peek == T_CS!("\\totalheightof") {
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let box_result = digest(arg)?;
    if expr_type == "Number" {
      Error!(
        "unexpected",
        "\\totalheightof",
        format!("\\totalheightof not expected here (reading {expr_type})")
      );
    }
    let height = box_result
      .get_height()
      .unwrap_or(RegisterValue::Dimension(Dimension::new(0)));
    let depth = box_result
      .get_depth()
      .unwrap_or(RegisterValue::Dimension(Dimension::new(0)));
    return Ok(CalcValue::Reg(height.add(depth)));
  }
  // \real{<decimal>} — returns a Float factor for multiplication
  if peek == T_CS!("\\real") {
    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let reader_mouth = Mouth::new("", None)?;
    let float = gullet::reading_from_mouth(reader_mouth, move || {
      gullet::unread(arg);
      let f = gullet::read_float()?;
      // Round as fixpoint, matching Perl calc.sty.ltxml line 164
      Ok(kround(f.value_f64() * UNITY as f64) as f64 / UNITY as f64)
    })?;
    return Ok(CalcValue::Flt(float));
  }
  // \ratio{<dimen expr>}{<dimen expr>} — returns a Float factor
  if peek == T_CS!("\\ratio") {
    let arg_x = gullet::read_arg(ExpansionLevel::Off)?;
    let arg_y = gullet::read_arg(ExpansionLevel::Off)?;
    let x = read_expression("Glue", arg_x)?;
    let y = read_expression("Glue", arg_y)?;
    let y_val = y.value_of() as f64;
    let y_pt = if y_val == 0.0 { 1.0 } else { y_val };
    return Ok(CalcValue::Flt(x.value_of() as f64 / y_pt));
  }
  // \minof{<expr>}{<expr>}
  if peek == T_CS!("\\minof") {
    let arg_x = gullet::read_arg(ExpansionLevel::Off)?;
    let arg_y = gullet::read_arg(ExpansionLevel::Off)?;
    let x = read_expression(expr_type, arg_x)?;
    let y = read_expression(expr_type, arg_y)?;
    return Ok(CalcValue::Reg(x.smaller(y)));
  }
  // \maxof{<expr>}{<expr>}
  if peek == T_CS!("\\maxof") {
    let arg_x = gullet::read_arg(ExpansionLevel::Off)?;
    let arg_y = gullet::read_arg(ExpansionLevel::Off)?;
    let x = read_expression(expr_type, arg_x)?;
    let y = read_expression(expr_type, arg_y)?;
    return Ok(CalcValue::Reg(x.larger(y)));
  }
  // Parenthesized subexpression: ( <expression> )
  if peek == T_OTHER!("(") {
    let inner = gullet::read_until(&Tokens!(T_OTHER!(")")));
    return Ok(CalcValue::Reg(read_expression(expr_type, inner?)?));
  }
  // Else: literal value — put back token and read normally
  gullet::unread_one(peek);
  if expr_type == "Number" {
    Ok(CalcValue::Reg(
      RegisterValue::Number(gullet::read_number()?),
    ))
  } else {
    Ok(CalcValue::Reg(RegisterValue::Glue(gullet::read_glue()?)))
  }
}

LoadDefinitions!({
  // Stub primitives so they're defined but NOT expandable.
  // The expression parser recognizes these tokens and handles them.
  // (Perl calc.sty.ltxml lines 23-28)
  DefPrimitive!("\\minof", None);
  DefPrimitive!("\\maxof", None);
  DefPrimitive!("\\widthof", None);
  DefPrimitive!("\\heightof", None);
  DefPrimitive!("\\ratio", None);
  DefPrimitive!("\\real", None);

  // \setcounter{<ctr>}{<integer expression>}
  DefPrimitive!("\\setcounter{}{}", sub[(ctr, arg)] {
    let ctr_str = Expand!(ctr).to_string();
    let value = read_expression("Number", arg)?;
    let num = Number::new(value.value_of());
    SetCounter!(&ctr_str, num);
  });

  // \addtocounter{<ctr>}{<integer expression>}
  DefPrimitive!("\\addtocounter{}{}", sub[(ctr, arg)] {
    let ctr_str = Expand!(ctr).to_string();
    let value = read_expression("Number", arg)?;
    let num = Number::new(value.value_of());
    AddToCounter!(&ctr_str, num);
  });

  // \setlength{Variable}{} — Perl parity: silently no-op on undefined variable
  // (Perl: `return unless $defn && ($defn ne 'missing');`).
  DefPrimitive!("\\setlength{Variable}{}", sub[(variable, arg)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      if let Some(defn) = rtoken.to_register() {
        let value = read_expression("Glue", arg)?;
        defn.set_value(value, None, params);
      }
    }
  });

  // \addtolength{Variable}{} — Perl parity: silently no-op on undefined variable.
  DefPrimitive!("\\addtolength{Variable}{}", sub[(variable, arg)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      if let Some(defn) = rtoken.to_register() {
        let old_value = defn.value_of(params.clone()).unwrap_or_default();
        let delta = read_expression("Glue", arg)?;
        defn.set_value(old_value.add(delta), None, params);
      }
    }
  });

  // \settowidth, \settoheight, \settodepth, \settototalheight
  // Keep the LaTeX default implementations which use \@settodim
  // (calc.sty enhances \setlength to parse expressions, so \settowidth -> \setlength works)
});
