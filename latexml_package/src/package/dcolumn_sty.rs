use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: dcolumn.sty.ltxml — decimal-aligned columns
  RequirePackage!("array");

  // Perl: \lx@unactivate DefToken — resets mathcode of a character
  DefPrimitive!("\\lx@unactivate DefToken", sub[(delim_tok)] {
    let delim_str = delim_tok.to_string();
    if let Some(ch) = delim_str.chars().next() {
      state::assign_mathcode(ch, 0u16, None);
    }
  });

  // Perl: \DC@{}{}{} — activates the decimal delimiter in math mode
  DefMacro!("\\DC@{}{}{}", sub[(delim, todelim, _ndec)] {
    let delim_str = delim.to_string();
    let todelim_str = todelim.to_string();
    if delim_str != todelim_str {
      if let Some(ch) = delim_str.chars().next() {
        // Make the delimiter math-active (code 0x8000)
        state::assign_mathcode(ch, 0x8000u16, None);
      }
      // Define the active character's expansion
      let expansion_body = s!(
        "\\lx@hidden@bgroup\\lx@unactivate{{{}}}\\lx@wrap[role=PERIOD]{{{}}}\\lx@hidden@egroup",
        delim_str, todelim_str
      );
      let expansion = mouth::tokenize_internal(&expansion_body);
      def_macro(T_CS!(delim_str), None, expansion, None)?;
    }
    // Save and deactivate $
    Let!("\\DC@saved@dollar", "$");
    state::let_i(&T_MATH!(), &T_CS!("\\relax"), None);
    // Start inline math if not already in math
    let in_math = state::lookup_value("IN_MATH").is_some();
    if in_math {
      Ok(Tokens::default())
    } else {
      Ok(Tokens!(T_CS!("\\lx@begin@inline@math")))
    }
  });

  // Perl: \DC@end — restores $ and ends inline math
  DefMacro!("\\DC@end", sub[_args] {
    state::let_i(&T_MATH!(), &T_CS!("\\DC@saved@dollar"), None);
    Ok(Tokens!(T_CS!("\\lx@end@inline@math")))
  });

  // Perl: DefColumnType('D{}{}{}', ...) — decimal alignment column
  // Simplified: uses center alignment with before/after wrappers
  DefColumnType!("D{}{}{}", sub[(delim, todelim, ndec)] {
    // Build before tokens: \DC@{delim}{todelim}{ndec}
    let mut before = vec![T_CS!("\\DC@"), T_BEGIN!()];
    before.extend(delim.unlist());
    before.push(T_END!());
    before.push(T_BEGIN!());
    before.extend(todelim.unlist());
    before.push(T_END!());
    before.push(T_BEGIN!());
    before.extend(ndec.unlist());
    before.push(T_END!());
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens::new(before)),
        after: Some(Tokens!(T_CS!("\\DC@end"))),
        align: Some(Align::Center),
        ..Cell::default()
      });
    });
  });
});
