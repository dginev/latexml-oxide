// A macro whose body calls Fatal() (digest_fatal_keeps_text_before.tex).
DefMacro("\\lxmbf", || { Fatal("unexpected", "Internal", "lxmbf gives up"); "X" });
