// Faithful port of Perl t/daemon/testlocks.sty.ltxml. Exercises `locked`:
// a locked macro can be re-defined by ANOTHER binding load (loads run UNLOCKED),
// but not by raw-TeX or document source.
DefMacro("\\fooA", "testlocks FOO-A", #{ locked: true });
DefMacro("\\fooB", "testlocks FOO-B", #{ locked: true });
DefMacro("\\fooC", "testlocks FOO-C", #{ locked: true });
DefMacro("\\fooD", "testlocks FOO-D", #{ locked: true });

InputDefinitions("testlocks-a", #{ type: "sty", noltxml: true });

// Defined in testlocks-b (loaded from raw testlocks-a.sty); re-defined here,
// back in this binding's UNLOCKED load, so \fooE ends as "testlocks FOO-E".
DefMacro("\\fooE", "testlocks FOO-E", #{ locked: true });
