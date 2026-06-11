use std::collections::VecDeque;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: array.sty.ltxml

  // Perl L18: DefRegister('\extrarowheight' => Dimension('0pt'));
  DefRegister!("\\extrarowheight" => Dimension::new(0));

  // Not sure how to effect these
  DefMacro!("\\firsthline", "\\hline");
  DefMacro!("\\lasthline", "\\hline");

  DefColumnType!(">{}",  sub[(before)] {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_before_column(VecDeque::from(before.unlist()));
    });
  });
  DefColumnType!("<{}", sub[(after)] {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_after_column(after.unlist());
    });
  });

  // Same as p but with vertical alignment centered
  DefColumnType!("m{Dimension}", sub[(width)] {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(Align::Justify),
        width: Some(width),
        vattach: Some(String::from("middle")),
        ..Cell::default()
      });
    });
  });
  // Same as p but with vertical alignment bottom
  DefColumnType!("b{Dimension}", sub[(width)] {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vbox"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(Align::Justify),
        width: Some(width),
        vattach: Some(String::from("bottom")),
        ..Cell::default()
      });
    });
  });

  // Like @{}, but should NOT suppress intercolumn space
  DefColumnType!("!{}", sub[(filler)] {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_between_column(filler.unlist());
    });
  });

  // w column: specified alignment and width
  DefColumnType!("w{}{Dimension}", sub[(align_arg, width)] {
    let align_str = align_arg.to_string();
    let alignment = match align_str.as_str() {
      "l" => Align::Left,
      "r" => Align::Right,
      _ => Align::Center,
    };
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(alignment),
        width: Some(width),
        ..Cell::default()
      });
    });
  });

  // W column: same as w
  DefColumnType!("W{}{Dimension}", sub[(align_arg, width)] {
    let align_str = align_arg.to_string();
    let alignment = match align_str.as_str() {
      "l" => Align::Left,
      "r" => Align::Right,
      _ => Align::Center,
    };
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(alignment),
        width: Some(width),
        ..Cell::default()
      });
    });
  });

  // \newcolumntype — define new column types
  // Perl L68-71:
  //   DefPrimitive('\newcolumntype{}[Number][]{}', sub {
  //     my ($stomach, $char, $nargs, $opt, $replacement) = @_;
  //     DefMacroI(T_CS('\NC@rewrite@' . ToString($char)), convertLaTeXArgs($nargs, $opt), $replacement);
  //     return AddToPreamble(T_CS('\newcolumntype'), $char, $nargs, $opt, $replacement); });
  DefPrimitive!("\\newcolumntype{}[Number][]{}", sub[(ch, nargs, opt, replacement)] {
    let ch_str = ch.to_string();
    let nargs_val = nargs.value_of() as usize;
    let opt_clone = opt.clone();
    let replacement_toks: Tokens = replacement.revert().into();
    // Define \NC@rewrite@<char> as a macro with the replacement
    let cs_name = s!("\\NC@rewrite@{ch_str}");
    let cs_args = convert_latex_args(nargs_val, opt_clone)?;
    def_macro(T_CS!(cs_name), cs_args, Some(ExpansionBody::from(replacement_toks.clone())), None)?;
    // AddToPreamble: record as <?latexml preamble="\newcolumntype{C}[nargs][opt]{...}"?>
    // Perl L71 passes $nargs and $opt through verbatim — we must preserve
    // both so a rebuilt definition like `\newcolumntype{C}[1][c]{...}` round-trips.
    let mut pi_tokens = vec![T_CS!("\\lx@add@Preamble@PI")];
    pi_tokens.push(T_BEGIN!()); // start of Undigested arg
    pi_tokens.push(T_CS!("\\newcolumntype"));
    pi_tokens.push(T_BEGIN!());
    pi_tokens.extend(ExplodeText!(ch_str));
    pi_tokens.push(T_END!());
    if nargs_val > 0 {
      pi_tokens.push(T_OTHER!("["));
      pi_tokens.extend(ExplodeText!(s!("{nargs_val}")));
      pi_tokens.push(T_OTHER!("]"));
      // Perl L71: $opt is forwarded too. When present it's the default value
      // of the first argument — emit as `[<opt>]` so the preamble round-trips.
      if let Some(opt_tks) = opt {
        pi_tokens.push(T_OTHER!("["));
        pi_tokens.extend(opt_tks.unlist());
        pi_tokens.push(T_OTHER!("]"));
      }
    }
    pi_tokens.push(T_BEGIN!());
    pi_tokens.extend(replacement_toks.unlist());
    pi_tokens.push(T_END!());
    pi_tokens.push(T_END!()); // end of Undigested arg
    unread(Tokens::new(pi_tokens));
    // The gullet will read \lx@add@Preamble@PI{...} and the stomach will digest it
  });

  DefMacro!("\\arraybackslash", r"\let\\\tabularnewline");
});
