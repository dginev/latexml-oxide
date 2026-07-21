// Example runtime script binding (Rhai) for latexml-oxide.
// See docs/parity/script_bindings_plan.md. The `runtime-bindings` feature is ON by
// default: name this file `<pkg>.sty.rhai` next to your document (or on the
// search path) and `\usepackage{<pkg>}` discovers and loads it — no
// recompilation. This file doubles as the surface tour and is load-tested in
// CI (`shipped_example_loads`).
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

// ── Environments (DefEnvironment) — template (#body) and imperative forms ──
DefEnvironment("{rquote}", "<ltx:quote class=\"rhai\">#body</ltx:quote>", #{
  mode: "internal_vertical"
});
// Env argument -> attribute goes via `properties` (an env's #n does NOT
// interpolate at attribute position — faithful Perl semantics):
DefEnvironment("{biop}{}",
  "<ltx:note role=\"biography\" class=\"#pname\">#body</ltx:note>", #{
  properties: |a| #{ pname: a }
});

// ── Option bags: hooks, fonts, reversion, sizer ──
// All Def* forms accept a trailing #{...} of Perl's key => value options.
DefConstructor("\\fnote{}{}",
  "^<ltx:note role=\"footnote\" ?#mark(mark=\"#mark\")()>#2</ltx:note>", #{
  mode: "internal_vertical",
  beforeDigest: || neutralize_font(),
  afterDigest: || {
    let m = whatsit().argString(1);
    if m != "" { whatsit().setProperty("mark", m); }
  },
  reversion: |mark, body| "\\fnote{" + mark + "}{" + body + "}",
  sizer: || "10pt;8pt;2pt"
});

// ── Counters & cross-references ──
NewCounter("exhibit");
DefConstructor("\\exhibit{}", "<ltx:text class=\"exhibit\">#tags #1</ltx:text>", #{
  properties: |x| RefStepCounter("exhibit")
});

// ── KeyVals ──
DefKeyVal("EX", "lang", "");
DefMacro("\\exkv OptionalKeyVals:EX {}", |kv, body| {
  "(" + GetKeyVal(kv, "lang") + ")" + body
});

// ── Math: DefMath, ligatures (data + matcher-closure), accents ──
DefMath("\\exsum", "∑", #{ role: "SUMOP", meaning: "sum" });
DefMathLigature(":=", "≔", #{ role: "ASSIGN", name: "assign" });
DefAccent("\\exacc", "́", "´");

// ── Document rewrites (run at finalization) ──
DefRewrite(#{ xpath: "descendant-or-self::ltx:text[@class='exhibit']",
              attributes: #{ lang: "exhibit" } });

// ── Gullet reads from a macro body ──
DefMacro("\\exread", || {
  let opt = ReadOptional();
  let arg = ReadArg();
  "R(" + opt + ":" + arg + ")"
});

// ── State & pool helpers (same names as the native macros) ──
DefPrimitive("\\exset{}", |v| {
  AssignValue("ex:k", v, "global");
  StepCounter("exhibit");
});
DefMacro("\\exget", || LookupString("ex:k") + "/" + CounterValue("exhibit").to_string());

// ── LookupDefinition: fetch an already-installed definition and PUSH hooks onto
//    it (Perl BookML's `my $d = LookupDefinition(T_CS('\rule')); push(@{ $$d{afterConstruct} }, sub{…})`).
//    Each `push<H>` appends, each `unshift<H>` prepends. Digest hooks
//    (before/afterDigest) work on any DefConstructor/DefPrimitive/DefMath;
//    construct + body hooks (before/afterConstruct, afterDigestBody) are
//    DefConstructor-only. `LookupDefinition` returns `()` for an undefined CS.
DefConstructor("\\exhook{}", "<ltx:text class=\"hooked\">#1</ltx:text>");
let exdef = LookupDefinition("\\exhook");
exdef.pushAfterDigest(|| { AssignValue("ex:digested", "yes", "global"); });
exdef.pushBeforeConstruct(|document| { document.setAttribute("data-pre", "1"); });
exdef.pushAfterConstruct(|document| { document.setAttribute("data-post", "1"); });
