static DNIR_REX: Lazy<Regex> = Lazy::new(|| Regex::new("^\\\\((?i)[dnir])").unwrap());

use crate::prelude::*;

/// multido.tex's `\fpAdd{a}{b}` (:217-220; `\fpSub` when `negate_b`, :221-223,
/// whose `\FPsub@` drops a leading `-` of `b` or puts one on, :226), on the
/// expanded texts: a literal port of `\FPadd@`, `\FPadd@@` and `\FPadd@@@`
/// (:227-282), so its output matches pdflatex's to the character. The sum
/// carries as many decimals as `b` — `a`'s decimals are read at that scale,
/// so `\fpAdd{0.5}{0.25}` is `0.30`, as in TeX — and a zero sum of a positive
/// `a` and a negative `b` prints as `-0`. Spaces count for nothing: TeX's
/// number scans and `\FPadd@@@`'s undelimited arguments pass them.
fn multido_fp_add(a: &str, b: &str, negate_b: bool) -> String {
  let unspaced = |x: &str| x.chars().filter(|c| !c.is_whitespace()).collect::<String>();
  let (a, b) = (unspaced(a), unspaced(b));
  let a = a.as_str();
  let b = if negate_b {
    let b = b.trim_start();
    match b.strip_prefix('-') {
      Some(rest) => rest.to_string(),
      None => s!("-{b}"),
    }
  } else {
    b.to_string()
  };
  // `#1.#2.#3\@nil` of `a..`: the integer part and the decimals.
  let split = |x: &str| -> (String, String) {
    let mut parts = x.splitn(3, '.');
    let int = parts.next().unwrap_or_default().to_string();
    let frac = parts.next().unwrap_or_default().to_string();
    (int, frac)
  };
  let (a_int, a_frac) = split(a);
  let (b_int, b_frac) = split(&b);
  let mut count = tex_integer(&a_int); // \count@
  let a_frac_value = tex_integer(&s!("0{a_frac}")); // \dimen@i
  let mut mcount = tex_integer(&b_int); // \multido@count
  let mut mdcount = tex_integer(&s!("0{b_frac}")); // \multidocount
  let mut dim = count; // \dimen@
  count = 1;
  // `\FPadd@@@#500000000\@nil`: ten to the number of b's decimals. Past 8
  // decimals TeX's `1#9` is no power of ten (its digits 9 on, then eight
  // zeros); that garbage is not reproduced, the scale is capped at 10^8.
  let scale = 10_i64.pow(b_frac.chars().count().min(8) as u32);
  if tex_integer(&s!("{a_int}1")) < 0 {
    count = -2;
    dim = -dim;
  }
  if tex_integer(&s!("{b_int}1")) < 0 {
    count = -count;
    mcount = -mcount;
  }
  if count > 0 {
    mcount += dim;
    mdcount += a_frac_value;
    if mdcount < scale {
      mdcount += scale;
    } else {
      mcount += 1;
    }
    count -= 3;
  } else {
    mcount -= dim;
    mdcount -= a_frac_value;
    if mcount < 0 {
      mcount = -mcount;
      mdcount = -mdcount;
      count += 1;
    } else if mcount == 0 && mdcount < 0 {
      mdcount = -mdcount;
      count += 1;
    }
    if mdcount < 0 {
      mdcount += scale;
      mcount -= 1;
    }
    mdcount += scale;
  }
  // `\FPadd@@`: the digits of \multidocount after its leading one.
  let digits = mdcount.to_string();
  let sign = if count == -1 { "-" } else { "" };
  match digits.get(1..).filter(|decimals| !decimals.is_empty()) {
    Some(decimals) => s!("{sign}{mcount}.{decimals}"),
    None => s!("{sign}{mcount}"),
  }
}

/// The integer TeX's `\count@=<text>` reads from `text`: optional signs and
/// spaces, then decimal digits (none is 0, as `\FPadd@`'s `\next`=`\z@`
/// stands in for an empty integer part).
fn tex_integer(text: &str) -> i64 {
  let mut negative = false;
  let mut chars = text.chars().peekable();
  while let Some(&c) = chars.peek() {
    match c {
      '-' => negative = !negative,
      '+' | ' ' => {},
      _ => break,
    }
    chars.next();
  }
  let magnitude = chars.take_while(char::is_ascii_digit).fold(0_i64, |n, d| {
    n.saturating_mul(10)
      .saturating_add(i64::from(d as u8 - b'0'))
  });
  if negative { -magnitude } else { magnitude }
}
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
          let needs_negate = lookup_int("\\multido@count") < 0;
          if vtype == "n" {
            // multido.tex:193-198 `\multido@init@n`: a Number variable starts
            // as its initial value AS WRITTEN, `\edef#3{#1}` of the text before
            // the `+` (`\n=1+1` is `1`, `\n=2.00+…` is `2.00`), and steps by
            // `\fpAdd{0}{<step>}` (`\fpSub` counting down), the text after it.
            // Perl reverts the Float it reads (`1.0`), whose `.0` a calc
            // `\yunitlength*\n` then reports (bardiag.sty:667: the bardiag
            // manuals; OXIDIZED_DESIGN #317).
            let Some(init) = read_until(&Tokens!(T_OTHER!("+")))? else {
              Error!("expected", "+", "Missing + in multido variables");
              break;
            };
            inits.push(T_CS!("\\edef"));
            inits.push(var);
            inits.push(T_BEGIN!());
            inits.extend(init.unlist());
            inits.push(T_END!());
            // The step is delimited by the `,` or the list's end (`#3\@nil`),
            // which strips one pair of braces around it.
            let (step, more) = match read_until(&Tokens!(T_OTHER!(",")))? {
              Some(step) => (step, true),
              None => (Tokens::new(take_rest_of_mouth()), false),
            };
            let step = do_expand(step)?.to_string();
            let step = step.trim();
            let step = step
              .strip_prefix('{')
              .and_then(|inner| inner.strip_suffix('}'))
              .unwrap_or(step);
            let step = multido_fp_add("0", step, needs_negate);
            steps.push(T_CS!("\\multido@step@n"));
            steps.push(var);
            steps.push(T_BEGIN!());
            steps.extend(Explode!(step));
            steps.push(T_END!());
            if !more {
              break;
            }
            skip_spaces()?;
            continue;
          }
          let init = match vtype.as_str() {
            "d" => Tokens!(Explode!(s!("{}sp", read_dimension()?.value_of()))),
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
          let step = match vtype.as_str() {
            "d" => {
              let mut stepv = read_dimension()?;
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
  DefMacro!("\\multido@step@n DefToken {}", "\\fpAdd{#1}{#2}{#1}");

  // multido.tex:215-222: fixed-point, on the expanded texts (Perl reads two
  // Floats and prints the sum as a Float: `2.00+-3.05` stepped to `-4.1`).
  DefMacro!("\\fpAdd {} {} DefToken", sub[(a, b, token)] {
    let sum = multido_fp_add(&Expand!(a).to_string(), &Expand!(b).to_string(), false);
    DefMacro!(token, None, Tokens!(Explode!(sum))); });
  DefMacro!("\\fpSub {} {} DefToken", sub[(a, b, token)] {
    let sum = multido_fp_add(&Expand!(a).to_string(), &Expand!(b).to_string(), true);
    DefMacro!(token, None, Tokens!(Explode!(sum))); });
});

#[cfg(test)]
mod tests {
  use super::multido_fp_add;

  /// pdflatex's `\fpAdd{a}{b}\x[\x]` (multido 1.42, TL 2025), including its
  /// readings of mixed decimals (`0.5+0.25` is `0.30`) and its `-0`.
  #[test]
  fn fp_add_matches_pdflatex() {
    for (a, b, sum) in [
      ("1", "-1", "-0"),
      ("-1", "1", "0"),
      ("2.00", "-3.05", "-1.05"),
      ("-1.05", "-3.05", "-4.10"),
      ("0.5", "0.25", "0.30"),
      ("1", "0.5", "1.5"),
      ("1.25", "1", "3.5"),
      ("-0.5", "0.25", "0.20"),
      ("-0.50", "0.25", "-0.25"),
      ("0", "3", "3"),
      ("0", "-3.05", "-3.05"),
      ("10", "-3.5", "6.5"),
      ("-2.5", "2.5", "0.0"),
      ("3", "-5", "-2"),
      ("-3", "5", "2"),
      ("-3", "-5", "-8"),
      ("0.99", "0.01", "1.00"),
      ("-0.99", "-0.01", "-1.00"),
      ("1.5", "-0.7", "0.8"),
      ("-1.5", "0.7", "-0.8"),
      (".5", ".5", "1.0"),
      ("7", "", "7"),
      ("-", "1", "1"),
      ("2.", "1.", "3"),
    ] {
      assert_eq!(multido_fp_add(a, b, false), sum, "\\fpAdd{{{a}}}{{{b}}}");
    }
    // Spaces count for nothing (a step before the list's closing brace).
    assert_eq!(multido_fp_add("0", "0.5 ", false), "0.5");
    assert_eq!(multido_fp_add(" 1.5", " 0.5 ", false), "2.0");
    // `\fpSub` negates its second argument (`\FPsub@`).
    assert_eq!(multido_fp_add("1.75", "0.25", true), "1.50");
    assert_eq!(multido_fp_add("0", "-3.05", true), "3.05");
  }
}
