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
LoadDefinitions!({
  AssignValue!("current_environment", String::new(), Some(Scope::Global));
  DefMacro!("\\@currenvir", "");
  DefPrimitive!("\\f{}", sub[ (env)] {
    let env_string = env.to_string();
    DefMacro!(T_CS!("\\@currenvir"), None, env);
    AssignValue!("current_environment", env_string);
  });

  DefPrimitive!(
  "\\lx@setcurrenvir{}", sub[ (env)] {
    let env_string = env.to_string();
    DefMacro!(T_CS!("\\@currenvir"), None, env);
    AssignValue!("current_environment", env_string);
  });
  Let!("\\@currenvline", "\\@empty");

  DefMacro!("\\begin{}", sub[ (env)] {
    let name = Expand!(env.clone()).to_string();
    let begin_name = format!("\\begin{{{name}}}");
    let before_opt = state!().lookup_tokens(&format!("@environment@{name}@beforebegin"));
    let after_opt  = state!().lookup_tokens(&format!("@environment@{name}@atbegin"));

    if is_defined(&begin_name) {
      let mut tks = before_opt.map(Tokens::unlist).unwrap_or_default();
      tks.push(T_CS!(begin_name));
      Ok(Tokens::new(tks)) // Magic cs!
    } else {
      let token = T_CS!(format!("\\{name}"));
      if !is_defined_token(&token) {
        // this creates {name} , {{ and }} are escapes in Rust's `format` macro
        let undef = format!("{{{name}}}");
        let message = s!("The environment {} is not defined.", undef);
        Error!("undefined", undef, message);
        note_status(LogStatus::Undefined, Some(&undef));
        // TODO:
        // state_mut!().install_definition(LaTeXML::Core::Definition::Constructor->new($token, undef,
        //       sub { LaTeXML::Core::Stomach::makeError($_[0], "undefined", $undef); })); }
      }
      let mut out_tokens = before_opt.map(Tokens::unlist).unwrap_or_default();
      out_tokens.push(T_CS!("\\begingroup"));
      if let Some(after) = after_opt {
        out_tokens.extend(after.unlist());
      }
      out_tokens.extend(Invocation!(T_CS!("\\lx@setcurrenvir"), vec![env])?.unlist());
      out_tokens.push(token);
      Ok(Tokens::new(out_tokens))
    }
  });

  DefMacro!("\\end {}", sub[ (env)]{
    let name = Expand!(env).to_string();
    let before = state!().lookup_tokens(&s!("@environment@{name}@atend"));
    let after = state!().lookup_tokens(&s!("@environment@{name}@afterend"));
    let mut t = T_CS!(s!("\\end{{{name}}}"));
    let mut out_tokens = Vec::new();
    if is_defined_token(&t) {
      // Magic CS!
      out_tokens.push(t);
      if let Some(afterend_toks) = after {
        out_tokens.extend(afterend_toks.unlist())
      }
    } else {
      out_tokens = before.map(Tokens::unlist).unwrap_or_default();
      t = T_CS!(s!("\\end{name}"));
      if is_defined_token(&t) {
        out_tokens.push(t);
      }
      out_tokens.push(T_CS!("\\endgroup"));
      if let Some(afterend_toks) = after {
        out_tokens.extend(afterend_toks.unlist())
      }
    }
    Ok(Tokens::new(out_tokens))
  });
});
