use crate::prelude::*;

static OPTS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*").unwrap());

LoadDefinitions!({
  // Apparently LaTeX does NOT define \magnification,
  // and babel uses that to determine whether we're runing LaTeX!!!
  Let!("\\magnification", "\\@undefined");
  Let!("\\@empty", "\\lx@empty");
  Let!("\\@ifundefined", "\\lx@ifundefined");
  //**********************************************************************
  // Basic \documentclass & \documentstyle

  DefConditional!("\\if@compatibility", { lookup_bool("2.09_COMPATIBILITY") });
  DefMacro!("\\@compatibilitytrue", "");
  DefMacro!("\\@compatibilityfalse", "");

  Let!("\\@currentlabel", "\\@empty");
  DefMacro!("\\@currdir", "./");

  // Let's try just starting with this set (since we've loaded LaTeX)
  AssignValue!("inPreamble", true); // \begin{document} will clear this.

  DefConstructor!("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []",
                  "<?latexml class='#2' ?#1(options='#1')?>",
    after_digest => sub[whatsit] {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let class_opts = match options {
        Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(ToString::to_string).collect(),
        None => Vec::new(),
      };
      load_class(&(whatsit.get_arg(2).unwrap().to_string()),
                class_opts,
                Tokens!(T_CS!("\\AtBeginDocument"), T_CS!("\\warn@unusedclassoptions")))
  });

  AssignValue!("@unusedoptionlist", Stored::Strings(Rc::new([])));
  DefPrimitive!("\\warn@unusedclassoptions", {
    if let Some(Stored::Strings(unused)) = lookup_value("@unusedoptionlist") {
      if !unused.is_empty() {
        Info!(
          "unexpected",
          "options",
          "Unused global options: {}",
          arena::with_many(&unused, |u| u.join(","))
        );
        state::assign_value("@unusedoptionlist", Stored::Strings(Rc::new([])), None);
      }
    }
  });

  // \documentstyle is defined in tex_job.rs (part of TeX.pool, before LaTeX.pool)

  // sub onlyPreamble {
  //   my ($cs) = @_;
  //   Error('unexpected', $cs, $state->getStomach,
  //     "The current command '" . ToString($cs) . "' can only appear in the preamble")
  //     unless LookupValue("inPreamble");
  //   return; }
});
