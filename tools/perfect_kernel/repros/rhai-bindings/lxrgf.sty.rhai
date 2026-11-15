// The binding for register_getter_fatal_continues_red.tex: a register getter that calls Fatal().
DefRegister("\\lxrgf", Dimension("0pt"), #{ getter: || { Fatal("unexpected", "Internal", "lxrgf gives up"); Dimension("1pt") } });
