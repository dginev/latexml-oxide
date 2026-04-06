use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMath!("\\bra{}", "\\langle#1|",            meaning => "bra");
  DefMath!("\\Bra{}", "\\left\\langle#1\\right|", meaning => "bra");
  DefMath!("\\ket{}", "|#1\\rangle",           meaning => "ket");
  DefMath!("\\Ket{}", "\\left|#1\\right\\rangle", meaning => "ket");
  DefMath!("\\lx@braket@{}", "\\langle#1\\rangle", meaning => "expectation");
  DefMath!("\\lx@Braket@{}", "\\left\\langle#1\\right\\rangle", meaning => "expectation");
  DefMath!("\\lx@braket@V{}{}", "\\langle#1\\,|\\,#2\\rangle", meaning => "inner-product");
  DefMath!("\\lx@braket@D{}{}", "\\langle#1\\,\\|\\,#2\\rangle", meaning => "inner-product");
  DefMath!("\\lx@Braket@V{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\right\\rangle", meaning => "inner-product");
  DefMath!("\\lx@Braket@D{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\right\\rangle", meaning => "inner-product");
  // All braket variants (Perl L90-114)
  DefMath!("\\lx@braket@VV{}{}{}", "\\langle#1\\,|#2\\,|\\,#3\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@braket@VD{}{}{}", "\\langle#1\\,|\\,#2\\,\\|\\,#3\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@braket@DV{}{}{}", "\\langle#1\\,\\|\\,#2\\,|\\,#3\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@braket@DD{}{}{}", "\\langle#1\\,\\|\\,#2\\,\\|\\,#3\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@Braket@VV{}{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\,\\middle|\\,#3\\right\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@Braket@VD{}{}{}", "\\left\\langle#1\\,\\middle|\\,#2\\,\\middle\\|\\,#3\\right\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@Braket@DV{}{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\,\\middle|\\,#3\\right\\rangle", meaning => "quantum-operator-product");
  DefMath!("\\lx@Braket@DD{}{}{}", "\\left\\langle#1\\,\\middle\\|\\,#2\\,\\middle\\|\\,#3\\right\\rangle", meaning => "quantum-operator-product");

  // \braket — splits argument on | bars to dispatch to V/D variants — Perl L57-66
  DefMacro!("\\braket{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let vbar = T_OTHER!("|");
    let parts: Vec<Tokens> = {
      let mut result = Vec::new();
      let mut current = Vec::new();
      for t in arg.unlist().iter() {
        if *t == vbar {
          result.push(Tokens::new(std::mem::take(&mut current)));
        } else {
          current.push(*t);
        }
      }
      result.push(Tokens::new(current));
      result
    };
    let expansion = match parts.len() {
      2 => format!("\\lx@braket@V{{{}}}{{{}}}", parts[0], parts[1]),
      n if n >= 3 => format!("\\lx@braket@VV{{{}}}{{{}}}{{{}}}", parts[0], parts[1], parts[2]),
      _ => format!("\\lx@braket@{{{}}}", parts[0]),
    };
    Ok(mouth::tokenize_internal(&expansion))
  });
  DefMacro!("\\Braket{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let vbar = T_OTHER!("|");
    let parts: Vec<Tokens> = {
      let mut result = Vec::new();
      let mut current = Vec::new();
      for t in arg.unlist().iter() {
        if *t == vbar {
          result.push(Tokens::new(std::mem::take(&mut current)));
        } else {
          current.push(*t);
        }
      }
      result.push(Tokens::new(current));
      result
    };
    let expansion = match parts.len() {
      2 => format!("\\lx@Braket@V{{{}}}{{{}}}", parts[0], parts[1]),
      n if n >= 3 => format!("\\lx@Braket@VV{{{}}}{{{}}}{{{}}}", parts[0], parts[1], parts[2]),
      _ => format!("\\lx@Braket@{{{}}}", parts[0]),
    };
    Ok(mouth::tokenize_internal(&expansion))
  });

  // Set notation (Perl L117-146)
  DefMath!("\\lx@set@{}", "\\{#1\\}", meaning => "set");
  DefMath!("\\lx@Set@{}", "\\left\\{#1\\right\\}", meaning => "set");
  DefMath!("\\lx@set@V{}{}", "\\{#1\\;|\\;#2\\}", meaning => "set");
  DefMath!("\\lx@set@D{}{}", "\\{#1\\;\\|\\;#2\\}", meaning => "set");
  DefMath!("\\lx@Set@V{}{}", "\\left\\{#1\\;\\middle|\\;#2\\right\\}", meaning => "set");
  DefMath!("\\lx@Set@D{}{}", "\\left\\{#1\\;\\middle\\|\\;#2\\right\\}", meaning => "set");
  // \set/\Set — split on | for set-builder notation — Perl L117-126
  DefMacro!("\\set{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let vbar = T_OTHER!("|");
    let parts: Vec<Tokens> = {
      let mut result = Vec::new();
      let mut current = Vec::new();
      for t in arg.unlist().iter() {
        if *t == vbar {
          result.push(Tokens::new(std::mem::take(&mut current)));
        } else {
          current.push(*t);
        }
      }
      result.push(Tokens::new(current));
      result
    };
    let expansion = match parts.len() {
      n if n >= 2 => format!("\\lx@set@V{{{}}}{{{}}}", parts[0], parts[1]),
      _ => format!("\\lx@set@{{{}}}", parts[0]),
    };
    Ok(mouth::tokenize_internal(&expansion))
  });
  DefMacro!("\\Set{}", sub[args] {
    let arg = args[0].clone().into_tokens_result()?;
    let vbar = T_OTHER!("|");
    let parts: Vec<Tokens> = {
      let mut result = Vec::new();
      let mut current = Vec::new();
      for t in arg.unlist().iter() {
        if *t == vbar {
          result.push(Tokens::new(std::mem::take(&mut current)));
        } else {
          current.push(*t);
        }
      }
      result.push(Tokens::new(current));
      result
    };
    let expansion = match parts.len() {
      n if n >= 2 => format!("\\lx@Set@V{{{}}}{{{}}}", parts[0], parts[1]),
      _ => format!("\\lx@Set@{{{}}}", parts[0]),
    };
    Ok(mouth::tokenize_internal(&expansion))
  });
});
