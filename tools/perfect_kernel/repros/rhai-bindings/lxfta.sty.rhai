// A ligature whose fontTest assigns a value (the repro fonttest_assigns_a_value.tex).
DefLigature("qq", "Q", #{ fontTest: |font| { AssignValue("lxfta_tested", true, "global"); true } });
