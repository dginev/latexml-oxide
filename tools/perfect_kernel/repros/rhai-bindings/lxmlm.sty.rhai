// A math ligature matcher that defines another at its first call (the repro math_ligature_matcher_defines_one.tex).
DefMathLigature(|node| {
  if LookupValue("lxmlm_done") != true { AssignValue("lxmlm_done", true, "global"); DefMathLigature(":=", "≔", #{ role: "RELOP", name: "assign" }); }
  ()
});
