// Registers (Perl's DefRegister / LookupRegister / AssignRegister) with typed values: Number, Dimension, Glue
// and MuGlue handles, built as Perl's constructors build them.

DefRegister("\\gap", Dimension("3pt"));
DefRegister("\\stretchy", Glue("1pt plus 2fil"));
DefRegister("\\tally", Number(0));

// LookupRegister returns the typed handle; arithmetic and comparison are Perl's (Number.pm add, multiply, ...).
DefMacro("\\showgap", |gullet| {
  let g = LookupRegister("\\gap");
  let doubled = g * 2;
  Explode(ToString(doubled) + (if doubled > Dimension("5pt") { " wide" } else { " narrow" }))
});

// AssignRegister at digestion time (a primitive), here globally.
DefPrimitive("\\bump", |stomach| {
  AssignRegister("\\tally", LookupRegister("\\tally") + Number(1), "global");
});
