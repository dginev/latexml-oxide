// Faithful port of Perl t/daemon/testlocks-b.sty.ltxml.
// RE-definition of \fooD allowed (binding load is UNLOCKED) → "testlocks-b FOO-D".
DefMacro("\\fooD", "testlocks-b FOO-D", #{ locked: true });
// \fooE first definition here; later re-defined by testlocks.sty.
DefMacro("\\fooE", "testlocks-b FOO-E", #{ locked: true });
