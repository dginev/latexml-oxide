// A macro whose body calls Fatal() (the RED digest_fatal_keeps_text_before_red.tex).
DefMacro("\\lxmbf", || { Fatal("unexpected", "Internal", "lxmbf gives up"); "X" });
