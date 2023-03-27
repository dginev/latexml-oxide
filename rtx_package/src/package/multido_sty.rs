lazy_static! {
  static ref DNIR_REX : Regex = Regex::new("^\\\\((?i)[dnir])").unwrap();
}

use crate::package::*;
LoadDefinitions!(outer_stomach, state, {

  DefRegister!("\\multido@count" => Number::new(0));
  DefRegister!("\\multidocount"  => Number::new(0));
  DefRegister!("\\multido@stuff" => Tokens!());

  DefMacro!("\\multido",  r"\multido@{}{\begingroup}{\endgroup}");
  DefMacro!("\\mmultido", r"\multido@{\multido@stepvar}{\begingroup}{\endgroup}");
  DefMacro!("\\Multido",  r"\multido@{}{}{}");
  DefMacro!("\\MMultido", r"\multido@{\multido@stepvar}{}{}");

  DefMacro!("\\multido@{}{}{}{}{}{}","#2\
      \\multido@count=#5\\relax\
      \\ifnum\\multido@count=\\z@\\else\\multido@@{#1}{#4}{#6}\\fi\
      #3\
      \\ignorespaces");

  // Simplified...
  DefMacro!("\\multido@@{}{}{}",
    "\\multido@@initvars@@{#2}\
      \\ifnum\\multido@count<\\z@\\multido@count=-\\multido@count\\fi\
      \\multidocount=1\\relax#1\\multido@stuff{#3}\\multido@loop");
  DefMacro!("\\multido@loop",
    "\\the\\multido@stuff\
      \\ifnum\\multidocount<\\multido@count\
      \\advance\\multidocount\\@ne\
      \\multido@stepvar\
      \\expandafter\\multido@loop\\fi");
  DefMacro!("\\multidostop", "\\multidocount=\\multido@count");

  // Annoyances with variables:
  //   Dimensions are always printed in scaled points (sp)
  //   Number are fixed point (and print that way!)
  // concievably variables can be redefined in middle of loop?
  DefMacro!("\\multido@@initvars@@{}", sub[ogullet, (variables), ostate] {
    let reader_mouth = Mouth::new("", None, ostate)?;
    let read_result : Result<Vec<Token>> = ogullet.reading_from_mouth(reader_mouth, ostate, |gullet, state| {
      gullet.unread(variables);
      let mut inits : Vec<Token> = Vec::new();
      let mut steps = Vec::new();
      gullet.skip_spaces(state);
      while let Some(var) = gullet.read_token(state) {
        if let Some(cap) = DNIR_REX.captures(var.get_cs_name()) {
          let vtype = cap.get(1).map_or(String::new(), |m| m.as_str().to_lowercase());
          if gullet.read_keyword(&["="], state)?.is_none() {
            Error!("expected", "=", gullet, state, "Missing = in multido variables");
          }
          let init = match vtype.as_str() {
            "d" => Tokens!(Explode!(s!("{}sp", gullet.read_dimension(state)?.value_of()))),
            "n" => gullet.read_float(state)?.revert(state)?,
            "i" => gullet.read_number(state)?.revert(state)?,
            "r" => gullet.read_float(state)?.revert(state)?,
            _ => panic!("This voids the regex condition (d|n|i|r).")
          };
          inits.push(T_CS!("\\def"));
          inits.push(var.clone());
          inits.push(T_BEGIN!());
          inits.extend(init.unlist());
          inits.push(T_END!());
          if gullet.read_keyword(&["+"], state)?.is_none() {
            Error!("expected", "+", gullet, state, "Missing + in multido variables");
          }
          let needs_negate = state.lookup_int("\\multido@count") < 0;
          let step = match vtype.as_str() {
            "d" => {
              let mut stepv = gullet.read_dimension(state)?;
              if needs_negate { stepv = stepv.negate(); }
              stepv.revert(state)?
            },
            "n" => {
              let mut stepv = gullet.read_float(state)?;
              if needs_negate { stepv = stepv.negate(); }
              stepv.revert(state)?
            },
            "i" => {
              let mut stepv = gullet.read_number(state)?;
              if needs_negate { stepv = stepv.negate(); }
              stepv.revert(state)?
            },
            "r" => {
              let mut stepv = gullet.read_float(state)?;
              if needs_negate { stepv = stepv.negate(); }
              stepv.revert(state)?
            },
            _ => panic!("This voids the regex condition (d|n|i|r).")
          };
          steps.push(T_CS!(s!("\\multido@step@{vtype}")));
          steps.push(var);
          steps.push(T_BEGIN!());
          steps.extend(step.unlist());
          steps.push(T_END!());
          if gullet.read_keyword(&[","], state)?.is_none() {
            break;
          }
        }  else {
          Error!("unexpected", var, gullet, state,
            format!("Wrong format for multido variable {var:?}"));
        }
        gullet.skip_spaces(state);
      }
      DefMacro!(T_CS!("\\multido@stepvar"), None, Tokens::new(steps), state);
      // Return the tokens to initialize the vars
      Ok(inits)
    });
    read_result?
  });

  DefMacro!("\\multido@step@d DefToken {Dimension}", sub[gullet, (v,step), state] {
    let origin = Dimension::from_str(&Expand!(&v, gullet).to_string(), state)?;
    let value = origin.add(step);
    DefMacro!(v, None, Tokens!(Explode!(format!("{}sp",value.value_of())))); });
  DefMacro!("\\multido@step@i DefToken {Number}", sub[gullet, (v, step), state] {
    let value = Number::from(Expand!(&v, gullet).to_string()).add(step);
    DefMacro!(v, None, Tokens!(Explode!(value.value_of()))); });
  DefMacro!("\\multido@step@r DefToken {Float}", sub[gullet, (v, step), state] {
    let value = Float::from(Expand!(&v, gullet).to_string()).add(step);
    DefMacro!(v, None, Tokens!(Explode!(value.to_tight_string()))); });
  // Note: n _should_ be fixed point!
  DefMacro!("\\multido@step@n DefToken {}", "\\fpAdd{#1}{#2}{#1}");

  // Should evolve these to work in fixed point (particularly, the formatting?)
  DefMacro!("\\fpAdd {Float} {Float} DefToken", sub[gullet, (a,b,token), state] {
    let value = a.add(b);
    DefMacro!(token, None, Tokens!(Explode!(value.to_tight_string()))); });
  DefMacro!("\\fpSub {Float} {Float} DefToken", sub[gullet, (a,b,token), state] {
    let value = a.subtract(b);
    DefMacro!(token, None, Tokens!(Explode!(value.to_tight_string()))); });

});