// The binding for lookupstring_of_tokens_red.tex: \lxset stores its argument (Tokens), \lxget reads it back.
DefPrimitive("\\lxset{}", |stomach, v| { AssignValue("lx:k", v, "global"); });
DefMacro("\\lxget", |gullet| LookupString("lx:k"));
