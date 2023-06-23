use crate::package::*;

//**********************************************************************
// C.8.2 Defining Environments
//**********************************************************************
// Note that \env & \endenv defined by \newenvironment CAN be
// invoked directly.
LoadDefinitions!({
  DefPrimitive!("\\newenvironment OptionalMatch:* {}[Number][]{}{}",
  sub[(star_opt, name, nargs, opt, begin, end)] {
    let mut gullet = gullet_mut!();
    let name = { Expand!(name).to_string() };
    let name_cs = T_CS!(format!("\\{name}"));
    if IsDefined!(&name_cs) {
      let is_locked = lookup_bool(&s!("\\{}:locked",name)) ||
       lookup_bool(&s!("\\begin{{{}}}:locked",name));
      if !is_locked {
        let message = s!("Ignoring redefinition (\\newenvironment) of Environment {:?}", name);
        Info!("ignore", name, message);
      }
    } else {
      // TODO: can we convince DefMacro! this is not a second mutable borrow of state::
      let converted_args = convert_latex_args(nargs.value_of() as usize, opt)?;
      let end_name_cs = T_CS!(s!("\\end{}",name));
      DefMacro!(name_cs, converted_args, begin);
      DefMacro!(end_name_cs, None, end);
    }
    Ok(Vec::new())
  });

  DefPrimitive!("\\renewenvironment OptionalMatch:* {}[Number][]{}{}",
  sub[(star, name, nargs, opt, begin, end)] {
    let mut gullet = gullet_mut!();
    let name = Expand!(name).to_string();
    let is_locked = lookup_bool(&s!("\\{}:locked",name)) ||
       lookup_bool(&s!("\\begin{{{}}}:locked",name));
    if !is_locked {
      let name_cs = T_CS!(s!("\\{}",name));
      let end_name_cs = T_CS!(s!("\\end{}",name));
      let converted_args = convert_latex_args(nargs.value_of() as usize, opt)?;

      DefMacro!(name_cs, converted_args, begin);
      DefMacro!(end_name_cs, None, end);
    }
    Ok(Vec::new())
  });
});
