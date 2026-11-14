DefMacroI(Tokenize("\\mE"), (), "ME");
DefMacroI("\\csnamemF\\endcsname", (), "MF");
DefConditional("\\ifrh{}", |gullet, x| ToString(x) == "a");
DefPrimitive("\\sets", || { SetCondition("\\relax", true); SetCondition("\\nosuch", true); });
DefMacro("\\rows", || {
  let r = [];
  r.push(if IfCondition("\\ifx") { "T" } else { "F" });
  r.push(if IfCondition("\\ifx", [T_CS("\\relax"), T_CS("\\relax")]) { "T" } else { "F" });
  r.push(if IfCondition("\\ifx", [T_CS("\\relax"), T_LETTER("a")]) { "T" } else { "F" });
  r.push(if IfCondition("\\ifnum", [Number(1), T_OTHER("<"), Number(2)]) { "T" } else { "F" });
  r.push(if IfCondition("\\ifrh", [Tokenize("a")]) { "T" } else { "F" });
  r.push(if IfCondition("\\ifrh") { "T" } else { "F" });
  r.push(if IsDefined("\\csnamemF\\endcsname") { "D" } else { "U" });
  r.push(if IsDefined("\\mF") { "D" } else { "U" });
  r.push(if LookupMeaning("\\nosuch").is_empty() { "E" } else { "N" });
  r.push(LookupMeaning("\\relax").len());
  r.push(LookupMeaning(T_LETTER("q")).len());
  r.reduce(|s, v| if s == "" { `${v}` } else { s + " " + v }, "")
});
