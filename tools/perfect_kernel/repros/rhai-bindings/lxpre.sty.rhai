// The binding for the three Package.pm-call repros (trimmedcommalist_trailing_comma_red.tex,
// stringify_dimension_red.tex, fontdecode_encoding_ignores_family_map_red.tex).
DefMacro("\\lxtcl", || `${TrimmedCommaList("a,b,").len()}`);
DefMacro("\\lxstd", || Stringify(Dimension("1pt")));
DeclareFontMap("GENC", ["x", "y", "z"]);
DeclareFontMap("GENC", ["X", "Y", "Z"], #{family: "serif"});
DefMacro("\\lxfdf", || FontDecode(1, "GENC"));
