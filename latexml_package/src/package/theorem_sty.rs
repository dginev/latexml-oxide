use crate::{engine::latex_constructs::*, prelude::*};
#[rustfmt::skip]
LoadDefinitions!({
  // This is close enough to amsthm to just use it.
  RequirePackage!("amsthm");

  // However, theorem.sty's styles do NOT record the headfont!
  set_savable_theorem_parameters(vec![
    "\\thm@bodyfont", "\\thm@headpunct",
    "\\thm@styling", "\\thm@headstyling", "thm@swap",
  ]);

  // And headpunct defaults to none.
  DefRegister!("\\thm@headpunct" => Tokens!());

  def_macro_noop("\\FMithmInfo")?;

  DefMacro!("\\theoremheaderfont{}", sub[(font)] {
    assign_register("\\thm@headfont",
      RegisterValue::Tokens(font.clone()), None, vec![])?;
    assign_register("\\thm@notefont",
      RegisterValue::Tokens(font), None, vec![])?;
    Ok(Tokens!())
  });

  // \lx@theorem@newtheoremstyle{name}{bodyfont}{headstyle}{swap}
  DefPrimitive!("\\lx@theorem@newtheoremstyle{}{}{}{}", sub[(name, bodyfont, headstyle, swap)] {
    let name_str = name.to_string();
    let swap_val = swap.eq_text("S");
    save_theorem_style(&name_str, vec![
      ("\\thm@bodyfont".into(), Stored::Tokens(bodyfont)),
      ("\\thm@headstyling".into(), Stored::Tokens(headstyle)),
      ("thm@swap".into(), Stored::Bool(swap_val)),
    ]);
    let name_for_closure = name_str.clone();
    DefMacro!(
      T_CS!(s!("\\th@{name_str}")),
      None,
      Some(ExpansionBody::Closure(Rc::new(move |_args| {
        use_theorem_style(&name_for_closure);
        Ok(Tokens!())
      })))
    );
  });

  RawTeX!(r"\lx@theorem@newtheoremstyle{plain}{\itshape}{\lx@makerunin}{N}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{break}{\slshape}{}{N}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{change}{\slshape}{\lx@makerunin}{S}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{margin}{\slshape}{\lx@makerunin\lx@makeoutdent}{S}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{marginbreak}{\slshape}{\lx@makeoutdent}{S}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{changebreak}{\slshape}{}{S}");
  // Redefine so we get correct parameters recorded. Perl passes
  // `\normalfont` as the 4th (swap) arg — ToString(\normalfont) != 'S',
  // so swap resolves to false, same as 'N'. We mirror Perl's literals.
  // Also: Perl's `remark` style uses EMPTY headstyle (no \lx@makerunin).
  RawTeX!(r"\lx@theorem@newtheoremstyle{definition}{}{\lx@makerunin}{\normalfont}");
  RawTeX!(r"\lx@theorem@newtheoremstyle{remark}{}{}{\normalfont}");
  RawTeX!(r"\th@plain");
});
