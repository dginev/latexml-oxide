// The binding for lookupmeaning_is_a_string_red.tex: \lxlm gives \lxr the meaning LookupMeaning returns.
DefPrimitive("\\lxlm", || AssignMeaning(T_CS("\\lxr"), LookupMeaning("\\relax")));
