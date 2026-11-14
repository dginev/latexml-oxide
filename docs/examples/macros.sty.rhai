// Expandable macros (Perl's DefMacro). A closure names the gullet handle and every argument, as Perl's
// sub { my ($gullet, $a, $b) = @_; ... }; each argument arrives as Tokens and the result is Tokens.

// Swap two arguments: Tokens(...) concatenates Token and Tokens values, up to four, or an array of any length
// (Tokenize("...") turns TeX source into Tokens; as in TeX, it drops a trailing space).
DefMacro("\\swap{}{}", |gullet, a, b| Tokens(b, a));

// Build the expansion token by token: T_CS names a control sequence, T_BEGIN/T_END are the braces.
DefMacro("\\bold{}", |gullet, x| Tokens(T_CS("\\textbf"), T_BEGIN(), x, T_END()));

// An optional argument is () (Perl's undef) when absent: test it with defined(v), never v == "".
DefMacro("\\hello[]{}", |gullet, greeting, name| {
  let g = if defined(greeting) { greeting } else { Tokenize("Hello") };
  Tokens([g, T_OTHER(","), T_SPACE(), name, T_OTHER("!")])
});

// Read from the gullet yourself, as Perl's $gullet->readArg: a body with no declared parameters.
DefMacro("\\pair", |gullet| {
  let first = gullet.readArg();
  let second = gullet.readArg();
  Tokens([T_OTHER("("), first, T_OTHER(","), second, T_OTHER(")")])
});

// Perl's truthiness is !!v: (), "", "0" and 0 are false; any Token or Tokens (even empty) is true, so test a string.
// A closure may also return a string of TeX, which is tokenized.
DefMacro("\\truthy{}", |gullet, x| if !!ToString(x) { "yes" } else { "no" });
