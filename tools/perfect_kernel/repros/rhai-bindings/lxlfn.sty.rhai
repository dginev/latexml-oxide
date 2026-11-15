// A ligature whose fontTest dies on a bold font (the RED fonttest_fatal_closed_by_set_node_red.tex).
DefLigature("qq", "Q", #{ fontTest: |font| { if font.getSeries() == "bold" { Fatal("unexpected", "Internal", "lxlfn gives up"); } true } });
