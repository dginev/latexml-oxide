// Example runtime script binding (Rhai) for latexml-oxide.
// See docs/script_bindings_plan.md. Load via the `script-bindings` feature.
//
// A binding file is a Rhai script that calls the registration functions below.
// NOTE: backslashes in TeX control sequences must be doubled in Rhai strings:
// write "\\foo", not "\foo".

// ── Expandable macros (DefMacro) ──
// Body receives each argument as its TeX-source string and returns a string of
// TeX, which is re-tokenized faithfully (an expansion to \textit yields a real
// control sequence, not letters).

DefMacro("\\twicex{}", |x| x + x);                       // \twicex{ab} -> abab
DefMacro("\\greet{}",  |name| "Hello, " + name + "!");   // \greet{World} -> Hello, World!
DefMacro("\\emphx{}",  |x| "\\textit{" + x + "}");       // expands to a real \textit

// ── Constructors (DefConstructor) — two forms ──

// (a) Template form (the common dialect): an XML string with #1..#9 for args.
//     Attributes may interpolate #n. Elements use the ltx: namespace.
DefConstructor("\\mytext{}", "<ltx:text class=\"rhai\">#1</ltx:text>");
DefConstructor("\\boxed{}",  "<ltx:text framed=\"rectangle\">#1</ltx:text>");

// (b) Imperative form: the body receives `document` (Perl's $_[0]) plus each
//     argument, and reads almost verbatim like the Perl original:
//
//     # Perl:  DefConstructor('\myemph{}', sub {
//     #          $_[0]->openElement('ltx:emph'); $_[0]->absorb($_[1]);
//     #          $_[0]->closeElement('ltx:emph'); });
DefConstructor("\\myemph{}", |document, x| {
  document.openElement("ltx:emph");
  document.absorb(x);
  document.closeElement("ltx:emph");
});

// Faithful translation of the documentation example — a no-arg constructor that
// just closes elements:
//   Perl: DefConstructor('\endreferences', sub {
//           $_[0]->maybeCloseElement('ltx:biblist');
//           $_[0]->maybeCloseElement('ltx:bibliography'); });
DefConstructor("\\endreferences", |document| {
  document.maybeCloseElement("ltx:biblist");
  document.maybeCloseElement("ltx:bibliography");
});

// Imperative constructor with an attribute and literal text:
//   \note{N} -> <ltx:text class="note">[N]</ltx:text>
DefConstructor("\\note{}", |document, x| {
  document.openElement("ltx:text");
  document.setAttribute("class", "note");
  document.absorbString("[");
  document.absorb(x);
  document.absorbString("]");
  document.closeElement("ltx:text");
});

// ── Class/package options (DeclareOption) ──
// Translates verbatim from:
//   Perl: DeclareOption('eqsecnum', sub { Digest(Tokenize('\AtEndOfClass{\eqsecnum}')); });
// `Tokenize` builds a token list, `Digest` runs it (here scheduling \eqsecnum at
// end of class). DeclareOption records the name and defines the \ds@<opt> hook.
DeclareOption("eqsecnum", || {
  Digest(Tokenize("\\AtEndOfClass{\\eqsecnum}"));
});

// ── Primitives (DefPrimitive) — digestion-time side-effects ──
// State is shared with assign_value (group-local) / assign_global / lookup_value.
DefPrimitive("\\setx{}", |v| { assign_global("script:x", v); });
DefMacro("\\getx", || lookup_value("script:x"));
