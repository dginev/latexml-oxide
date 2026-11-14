// Constructors (Perl's DefConstructor) build XML. Two forms, as in Perl.

// (a) A template: #1.. are the arguments, attributes may interpolate them.
DefConstructor("\\term{}", "<ltx:text class=\"term\">#1</ltx:text>");

// (b) A closure: the document handle, then each argument, as Perl's sub { my ($document, $x) = @_; ... }.
DefConstructor("\\aside{}", |document, x| {
  document.openElement("ltx:text", #{ class: "aside" });
  document.absorb(x);
  document.closeElement("ltx:text");
});

// Properties computed at digestion are named in the template as #name (Perl's properties => sub {...}); the
// properties closure receives the arguments only (Perl passes the stomach first).
DefConstructor("\\tagged{}", "<ltx:text class=\"#kind\">#1</ltx:text>", #{
  properties: |x| #{ kind: "k-" + ToString(x) }
});

// An environment: #body is its digested content.
DefEnvironment("{boxed}", "<ltx:block class=\"boxed\">#body</ltx:block>");
