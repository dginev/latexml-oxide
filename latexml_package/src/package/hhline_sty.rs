use crate::prelude::*;

/// The columns an `\hhline` specification rules (hhline.sty:61-96 `\HH@loop`, its `*{n}{…}` repeats expanded first
/// by `\HH@xexpast`, :49-60): each `-` (a rule), `=` (a double rule) and `~` (none) is a column, in order from the
/// first; `|`, `:`, `#` (vertical rules where they cross) and `t`, `b` (the halves of a double rule at a corner) mark
/// the rule between or on the columns, and spaces are skipped. Under colortbl a `>{<code>}` sets the rules' colours
/// (colortbl.sty:364-369), not modelled here. Returns the ruled columns, the doubly ruled ones, and what `\HH@loop`
/// ignores with its warning (a braced group, read by `\HH@let` as one argument, once).
fn hhline_columns(spec: &str, colortbl: bool) -> (Vec<usize>, Vec<usize>, Vec<String>) {
  let mut chars: Vec<char> = Vec::new();
  expand_repeats(&spec.chars().collect::<Vec<char>>(), &mut chars);
  let (mut ruled, mut doubled, mut ignored) = (Vec::new(), Vec::new(), Vec::new());
  let mut column = 0;
  let mut at = 0;
  // Past the item at `at`: a braced group whole, else one character.
  let skip_item = |at: usize| -> usize {
    if chars.get(at) != Some(&'{') {
      return at + 1;
    }
    let mut depth = 0;
    for (i, &c) in chars.iter().enumerate().skip(at) {
      match c {
        '{' => depth += 1,
        '}' => {
          depth -= 1;
          if depth == 0 {
            return i + 1;
          }
        },
        _ => {},
      }
    }
    chars.len()
  };
  while at < chars.len() {
    let c = chars[at];
    match c {
      '-' | '=' | '~' => {
        column += 1;
        if c != '~' {
          ruled.push(column);
        }
        if c == '=' {
          doubled.push(column);
        }
      },
      '|' | ':' | '#' | 't' | 'b' => {},
      c if c.is_whitespace() => {},
      '>' if colortbl => {
        // `>` and its code, whatever spaces stand between (`\HH@loop` reads `##1##2`).
        let mut next = at + 1;
        while chars.get(next).is_some_and(|c| c.is_whitespace()) {
          next += 1;
        }
        at = skip_item(next);
        continue;
      },
      '{' => {
        ignored.push("begin-group character {".to_string());
        at = skip_item(at);
        continue;
      },
      c => ignored.push(s!("the character {c}")),
    }
    at += 1;
  }
  (ruled, doubled, ignored)
}

/// `\HH@xexpast`'s `*<n><spec>` (as latex.ltx's `\@xexpast` for a column preamble): `<n>` and `<spec>` are each one
/// character or one braced group, the repeat expanded in place, nested repeats too.
fn expand_repeats(spec: &[char], out: &mut Vec<char>) {
  // One undelimited argument at `at`: a braced group's inside or one character, and where it ends.
  fn argument(spec: &[char], mut at: usize) -> (Vec<char>, usize) {
    while at < spec.len() && spec[at].is_whitespace() {
      at += 1;
    }
    if at >= spec.len() {
      return (Vec::new(), at);
    }
    if spec[at] != '{' {
      return (vec![spec[at]], at + 1);
    }
    let mut depth = 0;
    for (i, &c) in spec.iter().enumerate().skip(at) {
      match c {
        '{' => depth += 1,
        '}' => {
          depth -= 1;
          if depth == 0 {
            return (spec[at + 1..i].to_vec(), i + 1);
          }
        },
        _ => {},
      }
    }
    (spec[at + 1..].to_vec(), spec.len())
  }
  let mut at = 0;
  while at < spec.len() {
    if spec[at] == '*' {
      let (count, next) = argument(spec, at + 1);
      let (body, next) = argument(spec, next);
      let count: usize = count.iter().collect::<String>().trim().parse().unwrap_or(0);
      for _ in 0..count {
        expand_repeats(&body, out);
      }
      at = next;
    } else {
      out.push(spec[at]);
      at += 1;
    }
  }
}

#[rustfmt::skip]
LoadDefinitions!({
  // hhline.sty:61-96: `\hhline{<spec>}` rules the columns its specification names — `-` a rule, `=` a double rule,
  // `~` no rule, the vertical and corner marks `|`, `:`, `#`, `t`, `b` between them, `*{n}{…}` a repeat — as a row of
  // its own under the one above. Here the ruled columns are `\cline`s of the table model (sect10.rs `\lx@cline`), a
  // doubly ruled column twice (a doubled rule, as `\hline\hline`). Perl's binding (hhline.sty.ltxml:24) maps every
  // `\hhline` to a full-width `\hline` (KNOWN_PERL_ERRORS #1024: `\hhline{~~--}` ruled every column, `=` a single rule;
  // about 6,000 papers). What `\HH@loop` does not know is ignored with hhline's warning (:89-91). Guard
  // `perfect_kernel_batch78c::hhline_rules_the_columns_it_names`.
  DefMacro!("\\hhline Semiverbatim", sub[(spec)] {
    let spec = spec.to_string();
    let (ruled, doubled, ignored) = hhline_columns(&spec, lookup_bool("colortbl.sty_loaded"));
    for meaning in ignored {
      Warn!("unexpected", "\\hhline", s!("Package hhline Warning: {meaning} ignored in \\hhline argument"));
    }
    // One `\cline{a-b}` per run of adjacent columns, so the rule reverts as LaTeX writes it (a formula's `tex`).
    let mut code = String::new();
    for columns in [&ruled, &doubled] {
      let mut runs: Vec<(usize, usize)> = Vec::new();
      for &column in columns.iter() {
        match runs.last_mut() {
          Some((_, last)) if *last + 1 == column => *last = column,
          _ => runs.push((column, column)),
        }
      }
      for (first, last) in runs {
        code.push_str(&s!("\\noalign{{\\lx@cline{{{first}-{last}}}}}"));
      }
    }
    Ok(TokenizeInternal!(TeXString::assembled(code)))
  });
});
