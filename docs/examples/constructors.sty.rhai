// Constructors (Perl's DefConstructor) build XML. Two forms, as in Perl.

// (a) A template: #1.. are the arguments, attributes may interpolate them. Inline text starts the paragraph, so
// the space after it is kept (Perl's `enterHorizontal => 1`, as its \emph).
DefConstructor("\\term{}", "<ltx:text class=\"term\">#1</ltx:text>", #{ enterHorizontal: true });

// (b) A closure: the document handle, then each argument, as Perl's sub { my ($document, $x) = @_; ... }.
DefConstructor("\\aside{}", |document, x| {
  document.openElement("ltx:text", #{ class: "aside" });
  document.absorb(x);
  document.closeElement("ltx:text");
}, #{ enterHorizontal: true });

// Properties computed at digestion are named in the template as #name: the closure is Perl's
// properties => sub { my ($stomach, $x) = @_; ... }, each argument as whatsit.getArg gives it.
DefConstructor("\\tagged{}", "<ltx:text class=\"#kind\">#1</ltx:text>", #{
  properties: |stomach, x| #{ kind: "k-" + ToString(x) },
  enterHorizontal: true
});

// An environment: #body is its digested content.
DefEnvironment("{boxed}", "<ltx:block class=\"boxed\">#body</ltx:block>");
