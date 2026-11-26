// A macro whose body defines a macro with an unbalanced expansion (unbalanced_definition_keeps_text_before.tex).
DefMacro("\\lxubd", || { DefMacro("\\lxubdx", "{"); "X" });
