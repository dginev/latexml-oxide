use crate::prelude::*;
use std::collections::VecDeque;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: array.sty.ltxml

  TeX!(r"\newdimen\extrarowheight \extrarowheight=0pt");

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
  // NOTE: Simplified — doesn't handle optional arg or AddToPreamble
  DefPrimitive!("\\newcolumntype{}[Number][]{}", sub[(ch, _nargs, _opt, replacement)] {
    let ch_str = ch.to_string();
    // Define \NC@rewrite@<char> as a macro with the replacement
    let cs_name = s!("\\NC@rewrite@{ch_str}");
    def_macro(T_CS!(cs_name), None, Some(ExpansionBody::from(Tokens::new(replacement.revert()))), None)?;
    Ok(())
  });

  DefMacro!("\\arraybackslash", r"\let\\\tabularnewline");
});
