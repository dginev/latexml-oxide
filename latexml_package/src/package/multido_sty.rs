static DNIR_REX: Lazy<Regex> = Lazy::new(|| Regex::new("^\\\\((?i)[dnir])").unwrap());

use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  DefRegister!("\\multido@count" => Number::new(0));
  DefRegister!("\\multidocount"  => Number::new(0));
  DefRegister!("\\multido@stuff" => Tokens!());

  DefMacro!("\\multido", r"\multido@{}{\begingroup}{\endgroup}");
  DefMacro!(
    "\\mmultido",
    r"\multido@{\multido@stepvar}{\begingroup}{\endgroup}"
  );
  DefMacro!("\\Multido", r"\multido@{}{}{}");
  DefMacro!("\\MMultido", r"\multido@{\multido@stepvar}{}{}");

  DefMacro!(
    "\\multido@{}{}{}{}{}{}",
    "#2\\multido@count=#5\\relax\\ifnum\\multido@count=\\z@\\else\\multido@@{#1}{#4}{#6}\\fi#3\\ignorespaces"
  );

  // Simplified...
  DefMacro!(
    "\\multido@@{}{}{}",
    "\\multido@@initvars@@{#2}\\ifnum\\multido@count<\\z@\\multido@count=-\\multido@count\\fi\\multidocount=1\\relax#1\\multido@stuff{#3}\\multido@loop"
  );
  DefMacro!(
    "\\multido@loop",
    "\\the\\multido@stuff\\ifnum\\multidocount<\\multido@count\\advance\\multidocount\\@ne\\multido@stepvar\\expandafter\\multido@loop\\fi"
  );
  DefMacro!("\\multidostop", "\\multidocount=\\multido@count");

  // Annoyances with variables:
  //   Dimensions are always printed in scaled points (sp)
  //   Number are fixed point (and print that way!)
  // concievably variables can be redefined in middle of loop?
  DefMacro!("\\multido@@initvars@@{}", sub[(variables)] {
    let reader_mouth = Mouth::new("", None)?;
    let read_result : Result<Vec<Token>> =
    reading_from_mouth(reader_mouth, || {

      unread(variables);
      let mut inits : Vec<Token> = Vec::new();
      let mut steps = Vec::new();
      skip_spaces()?;
      while let Some(var) = read_token()? {
        // TODO: this defeats the point of the performance optimization
        // but it is so much *simpler* to allocate...
        let csname = var.with_cs_name(ToString::to_string);
        if let Some(cap) = DNIR_REX.captures(&csname) {
          let vtype = cap.get(1).map_or(String::new(), |m| m.as_str().to_lowercase());
          if read_keyword(&["="])?.is_none() {
            Error!("expected", "=", "Missing = in multido variables");
          }
          let init = match vtype.as_str() {
            "d" => Tokens!(Explode!(s!("{}sp", read_dimension()?.value_of()))),
            "n" => read_float()?.revert()?,
            "i" => read_number()?.revert()?,
            "r" => read_float()?.revert()?,
            _ => panic!("This voids the regex condition (d|n|i|r).")
          };
          inits.push(T_CS!("\\def"));
          inits.push(var);
          inits.push(T_BEGIN!());
          inits.extend(init.unlist());
          inits.push(T_END!());
          if read_keyword(&["+"])?.is_none() {
            Error!("expected", "+", "Missing + in multido variables");
          }
          let needs_negate = lookup_int("\\multido@count") < 0;
          let step = match vtype.as_str() {
            "d" => {
              let mut stepv = read_dimension()?;
              if needs_negate { stepv = stepv.negate(); }
              stepv.revert()?
            },
            "n" => {
              let mut stepv = read_float()?;
              if needs_negate { stepv = stepv.negate(); }
              stepv.revert()?
            },
            "i" => {
              let mut stepv = read_number()?;
              if needs_negate { stepv = stepv.negate(); }
              stepv.revert()?
            },
            "r" => {
              let mut stepv = read_float()?;
              if needs_negate { stepv = stepv.negate(); }
              stepv.revert()?
            },
            _ => panic!("This voids the regex condition (d|n|i|r).")
          };
          steps.push(T_CS!(s!("\\multido@step@{vtype}")));
          steps.push(var);
          steps.push(T_BEGIN!());
          steps.extend(step.unlist());
          steps.push(T_END!());
          if read_keyword(&[","])?.is_none() {
            break;
          }
        }  else {
          Error!("unexpected", var, format!("Wrong format for multido variable {var:?}"));
        }
        skip_spaces()?;
      }
      DefMacro!(T_CS!("\\multido@stepvar"), None, Tokens::new(steps));
      // Return the tokens to initialize the vars
      Ok(inits)
    });
    read_result?
  });

  DefMacro!("\\multido@step@d DefToken {Dimension}", sub[(v,step)] {
    let origin = Dimension::from_str(&Expand!(&v).to_string())?;
    let value = origin.add(step);
    DefMacro!(v, None, Tokens!(Explode!(format!("{}sp",value.value_of())))); });
  DefMacro!("\\multido@step@i DefToken {Number}", sub[(v, step)] {
    let value = Number::from(Expand!(&v).to_string()).add(step);
    DefMacro!(v, None, Tokens!(Explode!(value.value_of()))); });
  DefMacro!("\\multido@step@r DefToken {Float}", sub[(v, step)] {
    let value = Float::from(Expand!(&v).to_string()).add(step);
    DefMacro!(v, None, Tokens!(Explode!(value.to_tight_string()))); });
  // Note: n _should_ be fixed point!
  DefMacro!("\\multido@step@n DefToken {}", "\\fpAdd{#1}{#2}{#1}");

  // Should evolve these to work in fixed point (particularly, the formatting?)
  DefMacro!("\\fpAdd {Float} {Float} DefToken", sub[(a,b,token)] {
    let value = a.add(b);
    DefMacro!(token, None, Tokens!(Explode!(value.to_tight_string()))); });
  DefMacro!("\\fpSub {Float} {Float} DefToken", sub[(a,b,token)] {
    let value = a.subtract(b);
    DefMacro!(token, None, Tokens!(Explode!(value.to_tight_string()))); });
});
