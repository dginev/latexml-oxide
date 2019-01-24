use crate::package::*;

//**********************************************************************
// C.8.2 Defining Environments
//**********************************************************************
// Note that \env & \endenv defined by \newenvironment CAN be
// invoked directly.
LoadDefinitions!(state, {
  DefPrimitive!("\\newenvironment OptionalMatch:* {}[Number][]{}{}", sub[stomach, args, state] {
    unpack!(args => star, name, nargs, opt, begin, end);
    let name = { stomach.digest(name, state)?.to_string() };
    let name_cs = T_CS!(&s!("\\{}",name));
    let end_name_cs = T_CS!(&s!("\\end{}",name));
    let nargs : usize = nargs.to_string().parse().unwrap_or(0);
    if IsDefined!(&name_cs) {
      let is_locked = LookupValue!(&s!("\\{}:locked",name)).is_some() || LookupValue!(&s!("\\begin{{{}}}:locked",name)).is_some();
      if !is_locked {
        info!(target:&s!("ignore:{}", name), "Ignoring redefinition (\\newenvironment) of Environment {:?}", name);
      }
    } else {
      let opt = if opt.is_empty() { None } else { Some(opt) };
      let converted_args = convert_latex_args(nargs, opt, state)?; // TODO: can we convince DefMacroI! this is not a second mutable borrow of state?
      DefMacroI!(name_cs, converted_args, begin);
      DefMacroI!(end_name_cs, None, end);
    }
    Ok(vec![])
  });

  // DefPrimitive('\renewenvironment OptionalMatch:* {}[Number][]{}{}', sub {
  //     my ($stomach, $star, $name, $nargs, $opt, $begin, $end) = @_;
  //     $name = ToString(Digest($name));
  //     DefMacroI(T_CS("\\$name"), convertLaTeXArgs($nargs, $opt), $begin);
  //     DefMacroI(T_CS("\\end$name"), undef, $end); });
});
