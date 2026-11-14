// The binding for gullet_only_on_one_arg_macro_red.tex: a Perl-form body naming only the gullet of a 1-argument macro.
DefMacro("\\lxg{}", |gullet| Tokens(T_CS("\\textbf"), T_BEGIN(), gullet.readArg(), T_END()));
