// Two register getters: one calls Fatal(), the other catches one (the repro register_getter_fatal_printed_once.tex).
DefRegister("\\lxtwa", Dimension("0pt"), #{ getter: || { Fatal("unexpected", "Internal", "lxtwa gives up"); Dimension("1pt") } });
DefRegister("\\lxtwb", Dimension("0pt"), #{ getter: || { try { Fatal("unexpected", "Internal", "lxtwb caught one"); } catch (e) { } Dimension("2pt") } });
