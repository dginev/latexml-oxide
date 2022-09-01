use crate::package::*;

//**********************************************************************
// C.8.2 Defining Environments
//**********************************************************************
// Note that \env & \endenv defined by \newenvironment CAN be
// invoked directly.
LoadDefinitions!(state, {
  DefPrimitive!("\\newenvironment OptionalMatch:* {}[Number][]{}{}", sub[stomach, args, state] {
    unpack_opt!(args => star_opt, name_opt, nargs_opt, opt_opt, begin_opt, end_opt);
    let name = name_opt.owned_tokens().unwrap();
    let nargs = if nargs_opt.is_empty() { Number::new(0) } else { nargs_opt.owned_tokens().unwrap().to_number() };
    let begin = begin_opt.owned_tokens().unwrap();
    let end = end_opt.owned_tokens().unwrap();

    let name = { stomach.digest(name, state)?.to_string() };
    let name_cs = T_CS!(s!("\\{}",name));
    let end_name_cs = T_CS!(s!("\\end{}",name));
    if IsDefined!(&name_cs) {
      let is_locked = state.has_value(&s!("\\{}:locked",name)) ||
       state.has_value(&s!("\\begin{{{}}}:locked",name));
      if !is_locked {
        let message = s!("Ignoring redefinition (\\newenvironment) of Environment {:?}", name);
        Info!("ignore", name, stomach, state, message);
      }
    } else {
      let opt = if opt_opt.is_empty() { None } else { Some(opt_opt.owned_tokens().unwrap()) };
      let converted_args = convert_latex_args(nargs.value_of() as usize, opt, state)?; // TODO: can we convince DefMacro! this is not a second mutable borrow of state?
      DefMacro!(name_cs, converted_args, begin);
      DefMacro!(end_name_cs, None, end);
    }
    Ok(vec![])
  });

  // DefPrimitive('\renewenvironment OptionalMatch:* {}[Number][]{}{}', sub {
  //     my ($stomach, $star, $name, $nargs, $opt, $begin, $end) = @_;
  //     $name = ToString(Digest($name));
  //     DefMacroI(T_CS("\\$name"), convertLaTeXArgs($nargs, $opt), $begin);
  //     DefMacroI(T_CS("\\end$name"), undef, $end); });
});
