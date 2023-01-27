use crate::package::*;
// ======================================================================
// C.1.2 Environments
// ======================================================================

// In LaTeX, \newenvironment{env} defines \env and \endenv.
// \begin{env} & \end{env} open/close a group, and invoke these.
// In fact, the \env & \endenv don't have to have been created by
// \newenvironment; And in fact \endenv doesn't even have to be defined!
// [it is created by \csname, and equiv to \relax if no previous defn]

// We need to respect these usages here, but we also want to be able
// to define environment constructors that `capture' the body so that
// it can be processed specially, if needed.  These are the magic
// "\begin{env}", "\end{env}" control sequences created by DefEnvironment.
LoadDefinitions!(state, {
  AssignValue!("current_environment", String::new(), Some(Scope::Global));
  DefMacro!("\\@currenvir", "");
  DefPrimitive!("\\f{}", sub[stomach, args, state] {
    unpack!(args => env);
    let env_string = env.to_string();
    DefMacro!(T_CS!("\\@currenvir"), None, env);
    AssignValue!("current_environment", env_string);
  });

  DefPrimitive!(
  "\\lx@setcurrenvir{}", sub[stomach, args, state] {
    unpack!(args => env);
    let env_string = env.to_string();
    DefMacro!(T_CS!("\\@currenvir"), None, env);
    AssignValue!("current_environment", env_string);
  });
  Let!("\\@currenvline", "\\@empty");

  DefMacro!("\\begin{}", sub[gullet, (env), state] {
    let name = Expand!(env.clone(), gullet).to_string();
    let begin_name = s!("\\begin{{{}}}", name);
    let before_opt = state.lookup_tokens(&s!("@environment@{}@beforebegin", name));
    let after_opt  = state.lookup_tokens(&s!("@environment@{}@atbegin", name));

    if is_defined(&begin_name, state) {
      let mut tks = before_opt.map(Tokens::unlist).unwrap_or_default();
      tks.push(T_CS!(begin_name));
      Ok(Tokens::new(tks)) // Magic cs!
    } else {
      let token = T_CS!(s!("\\{}", name));
      if !is_defined_token(&token, state) {
        let undef = s!("{{{}}}", name); // this creates {name} , {{ and }} are escapes in Rust's format!
        let message = s!("The environment {} is not defined.", undef);
        Error!("undefined", undef, gullet, state, message);
        // TODO:
        // state.note_status("undefined", undef);
        // state.install_definition(LaTeXML::Core::Definition::Constructor->new($token, undef,
        //       sub { LaTeXML::Core::Stomach::makeError($_[0], "undefined", $undef); })); }
      }
      let mut out_tokens = if let Some(before) = before_opt { before.unlist() }
      else { Vec::new() };
      out_tokens.push(T_CS!("\\begingroup"));
      if let Some(after) = after_opt {
        out_tokens.extend(after.unlist());
      }
      out_tokens.extend(Invocation!(T_CS!("\\lx@setcurrenvir"), vec![env], gullet)?.unlist());
      out_tokens.push(token);
      Ok(Tokens::new(out_tokens))
    }
  });

  DefMacro!("\\end {}", sub[gullet, (env), state]{
    let name = Expand!(env, gullet).to_string();
    let before = state.lookup_value(&s!("@environment@{}@atend",name));
    let after  = state.lookup_value(&s!("@environment@{}@afterend",name));

    let mut t = T_CS!(s!("\\end{{{}}}", name));
    if is_defined_token(&t, state) {
      // Magic CS!
      // TODO: also add "after" tokens
      Ok(Tokens!(t))
    } else {
      t = T_CS!(s!("\\end{}", name));
      // TODO: also add "before" tokens
      if is_defined_token(&t, state) {
        Ok(Tokens!(t, T_CS!("\\endgroup")))
      } else {
        Ok(Tokens!(T_CS!("\\endgroup")))
      }
    }
  });
});
