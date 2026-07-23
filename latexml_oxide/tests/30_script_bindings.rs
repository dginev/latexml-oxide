//! End-to-end validation of runtime Rhai script bindings through a real
//! conversion (docs/parity/script_bindings_plan.md, milestone M4).
//!
//! A sample binding (one `DefMacro`, one `DefConstructor`) is authored in Rhai
//! and loaded at runtime via the *extra* binding dispatcher when the document
//! `\usepackage{lxrhaitest}`s it — exactly the path real contrib packages use.
//! We then assert on the produced XML: the macro must expand and the constructor
//! must emit its element.
#![cfg(feature = "runtime-bindings")]

use std::rc::Rc;

use latexml::core_interface::DigestionAPI;
use latexml_core::{Core, CoreOptions, common::error::Result, state};

/// A sample contrib binding, authored in Rhai (no Rust toolchain, no recompile).
const SAMPLE: &str = r##"
  // Expandable macro: \twicex{X} -> XX
  DefMacro("\\twicex{}", |x| x + x);

  // Constructor (imperative, proxy syntax close to Perl's $document->method):
  // \myemph{X} -> <ltx:emph>X</ltx:emph>.
  DefConstructor("\\myemph{}", |document, x| {
    document.openElement("ltx:emph");
    document.absorb(x);
    document.closeElement("ltx:emph");
  });

  // The documentation example, translated 1:1 from Perl. A no-arg constructor
  // that just (maybe-)closes elements; safe no-op where they aren't open.
  DefConstructor("\\endreferences", |document| {
    document.maybeCloseElement("ltx:biblist");
    document.maybeCloseElement("ltx:bibliography");
  });

  // Constructor (template form, the dominant dialect): \mytext{X} ->
  // <ltx:text class="rhai">X</ltx:text>, executed by the runtime template
  // interpreter (no Rhai per invocation).
  DefConstructor("\\mytext{}", "<ltx:text class=\"rhai\">#1</ltx:text>");

  // Re-entrancy: \wrap absorbs its (already-digested) argument, so
  // \wrap{\myemph{..}} makes one script constructor's body trigger another
  // script constructor's construction while \wrap's active-context is live.
  DefConstructor("\\wrap{}", |document, x| {
    document.openElement("ltx:text");
    document.absorb(x);
    document.closeElement("ltx:text");
  });

  // Imperative constructor exercising attributes + literal text:
  // \note{N} -> <ltx:text class="note">[N]</ltx:text>
  DefConstructor("\\note{}", |document, x| {
    document.openElement("ltx:text");
    document.setAttribute("class", "note");
    document.absorbString("[");
    document.absorb(x);
    document.absorbString("]");
    document.closeElement("ltx:text");
  });

  // Variable argument order + omission: 3 required args, template uses #3 then
  // #1 (reordered) and never references #2 (omitted) — exactly as the Rust
  // DefConstructor! macro / Perl template would.
  DefConstructor("\\rot{}{}{}", "<ltx:text>#3#1</ltx:text>");

  // Primitive: digestion-time side-effect into State (global, so it survives the
  // document group for the post-conversion assertion).
  DefPrimitive("\\setx{}", |v| { assign_global("script:x", v); });

  // Corpus-shaped template constructor exercising a top-level CONDITIONAL with
  // `#n` rendering through the real Document pipeline (the #171 template AST):
  // \cif{X} -> <ltx:emph>X</ltx:emph> when the arg is truthy, else the else-branch.
  DefConstructor("\\cif{}", "?#1(<ltx:emph>#1</ltx:emph>)(<ltx:text class=\"empty\">none</ltx:text>)");

  // A 1:1 port of plain TeX's \footnote constructor (plain_constructs.rs L292,
  // Perl TeX.pool `\footnote`) — the richest real template in the corpus: `^`
  // float prefix, conditional attribute pair on a `#mark` property hole,
  // conditional `#prenote` content hole, plus mode + an afterDigest hook that
  // routes the digested mark arg into the property the template consumes.
  DefConstructor("\\fnote{}{}",
    "^<ltx:note role=\"footnote\" ?#mark(mark=\"#mark\")()>?#prenote(#prenote )()#2</ltx:note>", #{
    mode: "internal_vertical",
    beforeDigest: || neutralize_font(),
    afterDigest: || {
      let m = whatsit().argString(1);
      if m != "" { whatsit().setProperty("mark", m); }
    }
  });

  // `properties` as a CLOSURE (Perl `properties => sub {…}`): computes the
  // whatsit's property map from the digested args; the template reads it
  // through a `#cls` hole at attribute position.
  DefConstructor("\\pnote{}", "<ltx:text class=\"#cls\">#1</ltx:text>", #{
    properties: |x| #{ cls: "from-" + x }
  });

  // `properties` as a STATIC MAP (Perl `properties => { key => value }`).
  DefConstructor("\\snote{}", "<ltx:text class=\"#cls\">#1</ltx:text>", #{
    properties: #{ cls: "static-props" }
  });

  // Processing-instruction template (the class/package PI dialect shape).
  DefConstructor("\\mypi{}", "<?mypi data=\"#1\"?>");

  // Environment, template form — a 1:1 port of latex_base's {quote}
  // (latex_constructs.rs L5019): the template's #body hole receives the
  // digested environment body.
  DefEnvironment("{rquote}", "<ltx:quote class=\"rhai\">#body</ltx:quote>", #{
    mode: "internal_vertical"
  });

  // Environment with a required argument — a 1:1 port of the cas-dc contrib
  // class's {bio}{} (cas_dc_cls.rs L108): #1 at attribute position + #body.
  // NB: an environment's `#n` at ATTRIBUTE position renders EMPTY — verified
  // identical in Perl LaTeXML, Rust native, and this runtime path (all three
  // emit <note role="biography"> with no name=). That's faithful semantics,
  // not a bug; the working idiom is `properties` (next specimen).
  DefEnvironment("{bio}{}",
    "<ltx:note role=\"biography\" name=\"#1\">#body</ltx:note>", #{
    mode: "internal_vertical"
  });

  // The Perl-idiomatic way to get an environment argument into an attribute:
  // route it through `properties` and use a `#prop` hole. (The attribute must
  // be schema-allowed: ltx:note has no @name, and BOTH Perl and Rust silently
  // drop schema-disallowed attributes — verified identical with a literal
  // name='LIT' probe. `class` is universally allowed.)
  DefEnvironment("{biop}{}",
    "<ltx:note role=\"biography\" class=\"#pname\">#body</ltx:note>", #{
    properties: |a| #{ pname: a }
  });

  // Environment, IMPERATIVE form — the {center}-style native shape
  // (sub[document, _args, props]): the body reaches #body through
  // document.absorbProperty.
  DefEnvironment("{rbox}", |document| {
    document.openElement("ltx:text");
    document.setAttribute("class", "rbox");
    document.absorbProperty("body");
    document.closeElement("ltx:text");
  });

  // ── Wave C/E: package-option machinery through a real \usepackage[draft] ──
  // (ieeetran/article classes' DeclareOption + ProcessOptions shape.)
  DeclareOption("draft", || assign_global("rh:opt", "draft-on"));
  ProcessOptions();

  // ieeetran {IEEEproof}-style: the `properties` closure DIGESTS content
  // up-front (title + its font), the template consumes #title/#font holes.
  DefEnvironment("{rproof}[]",
    "<ltx:proof class=\"ltx_proof\"><ltx:title font=\"#font\">#title</ltx:title>#body</ltx:proof>", #{
    mode: "internal_vertical",
    properties: |_a| {
      let title = DigestText("{\\bfseries Proof:}");
      #{ title: title, font: title }
    }
  });

  // amsmath-style numbered construct: RefStepCounter as the properties
  // closure, plus a string reversion (the `\@@multline` option shape, scaled
  // down). NB `#tags` (an `ltx:tags`) is BLOCK-level and cannot be rendered
  // inside an inline `ltx:text` — doing so makes the document validly
  // auto-close the text to host the block, leaving the template's
  // `</ltx:text>` with nothing to close (a benign-but-logged malformed close).
  // So we render the refnum as a STRING property instead; RefStepCounter is
  // still exercised (it steps the counter and returns its map).
  NewCounter("rqeq");
  DefConstructor("\\numbered{}", "<ltx:text class=\"eq\">(#refnum) #1</ltx:text>", #{
    properties: |x| { let m = RefStepCounter("rqeq"); m.refnum = CounterValue("rqeq").to_string(); m },
    reversion: "\\numbered{#1}"
  });

  // natbib \cite-style parameter types: star match + two optionals +
  // Semiverbatim keys — the prototype dialect is shared with the macros.
  DefMacro("\\rcite OptionalMatch:* [][] Semiverbatim", |star, pre, post, keys| {
    "[" + pre + ";" + post + ";" + keys + ";" + star + "]"
  });

  // graphics \Gscale@box-style: {Float} params + a properties closure that
  // COMPUTES from the numeric args (the dual properties/afterDigest pattern).
  DefConstructor("\\gsbox {Float}{Float} {}",
    "<ltx:inline-block class=\"gs\" xscale=\"#xscale\" yscale=\"#yscale\">#3</ltx:inline-block>", #{
    properties: |xs, ys, _box| #{ xscale: xs, yscale: ys }
  });

  // listings/siunitx-style OptionalKeyVals: the keyval dict arrives as the
  // macro's first argument (its TeX-source form).
  DefKeyVal("RH", "lang", "");
  DefKeyVal("RH", "size", "");
  DefMacro("\\kvprobe OptionalKeyVals:RH {}", |kv, body| {
    "(" + kv + ")" + body
  });

  // sizer + closure-form reversion (read-only whatsit context in both).
  DefConstructor("\\sized{}", "<ltx:text class=\"sz\">#1</ltx:text>", #{
    sizer: || "10pt;8pt;2pt",
    reversion: |x| "\\sized{" + x + "}"
  });

  // DefAccent: \racc{o} routes through \lx@applyaccent (combining acute).
  DefAccent("\\racc", "́", "´");

  // DefMathLigature: ":=" collapses to one ASSIGN token (data-form matcher).
  DefMathLigature(":=", "≔", #{ role: "ASSIGN", name: "assign" });

  // DefMathLigature, matcher-CLOSURE form: a Rhai body walking the node
  // chain (read-only Node proxy), merging "!!" into a single double-bang.
  DefMathLigature(|node| {
    if node.qname() != "ltx:XMTok" || node.content() != "!" { return (); }
    let prev = node.prevSibling();
    if prev == () { return (); }
    if prev.qname() == "ltx:XMTok" && prev.content() == "!" {
      #{ n: 2, replacement: "‼", role: "POSTFIX" }
    } else { () }
  });

  // Gullet seams from a macro body (Perl $gullet->readOptional/readArg):
  // \gread[x]{y} -> G(x:y) — mid-expansion stream reads from Rhai.
  DefMacro("\\gread", || {
    let opt = ReadOptional();
    let arg = ReadArg();
    "G(" + opt + ":" + arg + ")"
  });

  // DefRewrite, replace-closure form (replace-by-reinsertion, the native
  // sub[document,nodes] shape): detach the victim, insert a replacement.
  DefConstructor("\\rwvictim{}", "<ltx:text class=\"victim\">#1</ltx:text>");
  DefRewrite(#{ select: "descendant-or-self::ltx:text[@class='victim']" }, |document, nodes| {
    document.openElement("ltx:text");
    document.setAttribute("class", "replaced");
    document.absorbString("REPL");
    document.closeElement("ltx:text");
  });

  // DefRewrite (data form): stamp every biography note at finalization.
  DefRewrite(#{ xpath: "descendant-or-self::ltx:note[@role='biography'][not(@class)]",
                attributes: #{ class: "rw-stamp" } });

  // XML-parser exposure (#350): parse a small (X)HTML snippet and splice the
  // parsed SUBTREE into the tree at the current point — the Rhai analog of Perl
  // BookML's \bmlRawHTML (XML::LibXML->parse_string + $document->appendTree). The
  // snippet arrives as MARKUP (not TeX-escaped text), so the assertion proves it
  // became structured element/attribute/text nodes, not an escaped `&lt;p&gt;`
  // string. `<ltx:rawhtml>` is the schema's xhtml-markup container (Misc class).
  //
  // Namespaces go through the SAME registry the Perl bindings use — these are
  // the Rhai-exposed `RegisterNamespace`/`RegisterDocumentNamespace` helpers
  // (Package.pm:2049-2057). The snippet declares xhtml as its DEFAULT namespace
  // (empty prefix), so re-creating it correctly depends on resolving the URI
  // through this registry; the assertion pins that it lands as xhtml, not ltx.
  // (Core pre-registers xhtml, so these calls are idempotent — they document the
  // flow a binding needs for a namespace core does not already know.)
  RegisterNamespace("xhtml", "http://www.w3.org/1999/xhtml");
  RegisterDocumentNamespace("xhtml", "http://www.w3.org/1999/xhtml");

  DefConstructor("\\rhrawhtml", |document| {
    document.openElement("ltx:rawhtml");
    document.insertXML("<p xmlns=\"http://www.w3.org/1999/xhtml\" class=\"lead\">hi <b>bold</b> x</p>");
    document.closeElement("ltx:rawhtml");
  });

  // The XML-manipulation foundation, exercised as a script author would: parse a
  // FRAGMENT (two sibling roots — rejected by Perl's single-node parseChunk, an
  // intentional divergence), walk and EDIT the parsed nodes while they are still
  // detached, then insert them. Proves the parsed nodes survive being held across
  // statements (they own their document), that node methods work on them exactly
  // as on in-tree nodes, and that edits made before insertion reach the document.
  DefConstructor("\\rhfragment", |document| {
    let nodes = ParseXML("<p xmlns=\"http://www.w3.org/1999/xhtml\">one</p><p xmlns=\"http://www.w3.org/1999/xhtml\">two</p>");
    for n in nodes {
      n.setAttribute("class", "frag-" + n.firstChild().content());
    }
    // Walking UP from a top-level parsed node must find nothing: above it lies
    // only the throwaway `_lxfragment` wrapper a multi-root chunk is parsed
    // inside. Leaking it would let a binding splice `<_lxfragment>` into the page.
    nodes[0].setAttribute("data-top",
      if type_of(nodes[0].parent()) == "()" { "detached" } else { "LEAKED-WRAPPER" });
    document.openElement("ltx:rawhtml");
    document.insertXML(nodes);
    document.closeElement("ltx:rawhtml");
  });
"##;

/// Extra dispatcher: load the sample script when `lxrhaitest` is requested.
fn script_dispatch(request: &str) -> Option<Result<()>> {
  let base = request.split('.').next().unwrap_or(request);
  if base == "lxrhaitest" {
    Some(latexml_contrib::script_bindings::load_script(SAMPLE).map(|_| ()))
  } else {
    None
  }
}

#[test]
fn script_binding_macro_and_constructor_convert() {
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  state::set_extra_bindings_dispatch(Rc::new(script_dispatch));

  let tex = concat!(
    "literal:\\documentclass{article}\\usepackage[draft]{lxrhaitest}",
    "\\begin{document}\\twicex{ab} \\myemph{hi} \\mytext{zz} \\wrap{\\myemph{deep}} \\wrap{\\wrap{\\myemph{deeper}}} \\note{N} \\rot{xx}{yy}{zz2} \\cif{Y}\\cif{} ",
    "body\\fnote{*}{Marked}more\\fnote{}{Plain} \\pnote{dyn} \\snote{st} \\mypi{d1} ",
    "\\begin{rquote}Quotable\\end{rquote} \\begin{bio}{Ada}Pioneer\\end{bio} ",
    "\\begin{biop}{Ada}Idiom\\end{biop} \\begin{rbox}Boxed\\end{rbox} ",
    "\\begin{rproof}QED-body\\end{rproof} \\numbered{NUM} \\rcite*[pre][post]{k1,k2} ",
    "\\gsbox{2}{3}{SCL} \\kvprobe[lang=rust]{KVB} \\sized{SZ} \\racc{o} $a := b$ $c!!$ \\gread[x]{y} \\rwvictim{OLD} ",
    "\\rhrawhtml \\rhfragment ",
    "\\endreferences \\setx{hello}\\end{document}"
  );
  let doc = latexml
    .convert_file(tex.to_string())
    .expect("conversion with a script binding should succeed");
  let xml = doc.serialize_to_string();

  // No spurious `Error:`/`Fatal:` from any specimen — pins the malformed-close
  // class (a block hole rendered in inline text would auto-close the text and
  // leave the template's `</…>` dangling). Uses the O(1) status counter
  // (`convert_file` reset it at conversion start), not a log-string scan —
  // order-independent and allocation-free.
  use latexml_core::common::error::{LogStatus, get_status};
  assert!(
    get_status(LogStatus::Error) == 0 && get_status(LogStatus::Fatal) == 0,
    "conversion logged errors: {}",
    latexml_core::common::error::get_status_message()
  );

  // NB: the serializer emits the LaTeXML namespace as the default (no `ltx:`
  // prefix), so elements appear unprefixed.
  assert!(
    xml.contains("abab"),
    "macro \\twicex did not expand; xml=\n{xml}"
  );
  assert!(
    xml.contains("<emph>hi</emph>"),
    "imperative constructor \\myemph did not emit its element; xml=\n{xml}"
  );
  assert!(
    xml.contains("class=\"rhai\"") && xml.contains("zz"),
    "template constructor \\mytext did not emit; xml=\n{xml}"
  );
  // Re-entrancy: the nested script constructor (\myemph) ran inside \wrap's
  // body — one script constructor's body triggering another's construction
  // while \wrap's active-context is live. This exercises the raw-pointer
  // re-mint in `script_bindings/mod.rs::with_doc` on the REAL libxml2-backed
  // path; the borrow-aliasing soundness of the pattern is proven separately by
  // the Miri model `latexml_core::runtime_bindings_reentrancy_model` (PR #248 B1).
  assert!(
    xml.contains("<emph>deep</emph>"),
    "re-entrant nested script constructor failed; xml=\n{xml}"
  );
  // Deeper 3-level re-entrancy `\wrap{\wrap{\myemph{deeper}}}` — two outer
  // `absorb`s parked while the innermost body mutates (the worst-case live
  // reborrow depth; mirrors the model's `three_levels` Miri case). `\wrap`'s
  // attribute-less `<text>` wrappers collapse (LaTeXML merges them, as with the
  // single-`\wrap` case above), so the proof is that the innermost content
  // survived all three re-entrant `absorb` levels intact.
  assert!(
    xml.contains("<emph>deeper</emph>"),
    "deep (3-level) re-entrant script construction failed; xml=\n{xml}"
  );
  // Imperative attributes + text (el_attr/el_text).
  assert!(
    xml.contains("class=\"note\""),
    "imperative el_attr/el_text constructor \\note failed; xml=\n{xml}"
  );
  // Variable argument order + omission: \rot{xx}{yy}{zz2} with template
  // "#3#1" must yield "zz2xx" (order #3 then #1) and never "yy" (arg #2 omitted).
  assert!(
    xml.contains("zz2xx") && !xml.contains("yy"),
    "variable-order/omitted-arg handling failed; xml=\n{xml}"
  );
  // Template CONDITIONAL through the real pipeline (#171 AST): \cif{Y} takes the
  // truthy then-branch (<emph>Y</emph>); \cif{} takes the else-branch.
  assert!(
    xml.contains("<emph>Y</emph>"),
    "template conditional then-branch (\\cif{{Y}}) failed; xml=\n{xml}"
  );
  assert!(
    xml.contains("class=\"empty\"") && xml.contains("none"),
    "template conditional else-branch (\\cif{{}}) failed; xml=\n{xml}"
  );
  // \footnote port: `^` float + `?#mark(mark="#mark")()` conditional attribute
  // fed by the afterDigest setProperty hook. \fnote{*}{Marked} must carry the
  // mark; \fnote{}{Plain} must not (the hook skips the empty mark).
  assert!(
    xml.contains("role=\"footnote\"") && xml.contains("mark=\"*\"") && xml.contains("Marked"),
    "footnote port: marked note missing/incomplete; xml=\n{xml}"
  );
  let unmarked = xml
    .split("<note")
    .skip(1)
    .any(|n| n.contains("Plain") && !n.split('>').next().unwrap_or("").contains("mark="));
  assert!(
    unmarked,
    "footnote port: unmarked note should have no mark attribute; xml=\n{xml}"
  );
  // properties-as-closure: #cls hole filled from the computed map.
  assert!(
    xml.contains("class=\"from-dyn\""),
    "properties closure (\\pnote) did not populate #cls; xml=\n{xml}"
  );
  // properties-as-static-map.
  assert!(
    xml.contains("class=\"static-props\""),
    "static properties map (\\snote) did not populate #cls; xml=\n{xml}"
  );
  // PI template through the runtime interpreter.
  assert!(
    xml.contains("<?mypi") && xml.contains("data=\"d1\""),
    "PI template (\\mypi) did not emit; xml=\n{xml}"
  );
  // DefEnvironment, template form ({quote} port): #body lands inside the element.
  assert!(
    xml.contains("class=\"rhai\"") && xml.contains("Quotable"),
    "environment template ({{rquote}}) did not wrap its body; xml=\n{xml}"
  );
  // DefEnvironment with a required argument ({bio}{} port from cas-dc):
  // faithful-Perl behavior is role+body present and `name=` ABSENT (an env's
  // `#n` at attribute position renders empty in Perl, Rust native, and here —
  // all three verified identical; "Ada" is consumed by the begin's arg read).
  let bio_note = xml
    .split("<note")
    .skip(1)
    .find(|n| n.contains("Pioneer"))
    .expect("bio note missing");
  assert!(
    bio_note.contains("role=\"biography\"")
      && !bio_note.split('>').next().unwrap_or("").contains("name="),
    "environment with arg ({{bio}}{{Ada}}) diverged from faithful-Perl output; xml=\n{xml}"
  );
  // The working idiom ({biop}): env arg → properties closure → #pname hole at
  // attribute position, on a schema-allowed attribute.
  assert!(
    xml.contains("class=\"Ada\"") && xml.contains("Idiom"),
    "environment properties idiom ({{biop}}) failed; xml=\n{xml}"
  );
  // DefEnvironment, imperative form: absorbProperty(\"body\").
  assert!(
    xml.contains("class=\"rbox\"") && xml.contains("Boxed"),
    "imperative environment ({{rbox}}) / absorbProperty failed; xml=\n{xml}"
  );
  // Wave C: \usepackage[draft] → DeclareOption body ran via ProcessOptions.
  let opt = match state::lookup_value("rh:opt") {
    Some(latexml_core::common::store::Stored::String(s)) => {
      latexml_core::common::arena::to_string(s)
    },
    _ => String::from("<unset>"),
  };
  assert_eq!(
    opt, "draft-on",
    "DeclareOption+ProcessOptions did not fire for [draft]"
  );
  // IEEEproof port: properties closure digests the title; #title/#font holes.
  assert!(
    xml.contains("<proof") && xml.contains("Proof:") && xml.contains("QED-body"),
    "IEEEproof-style ({{rproof}}) properties-digestion failed; xml=\n{xml}"
  );
  // amsmath-style: RefStepCounter (as properties) steps the counter and the
  // refnum (1 after the first step) renders inline alongside the arg.
  assert!(
    xml.contains("class=\"eq\"") && xml.contains("(1)") && xml.contains("NUM"),
    "RefStepCounter-properties constructor (\\numbered) failed; xml=\n{xml}"
  );
  // natbib-style parameter types: star + optionals + semiverbatim keys.
  assert!(
    xml.contains("[pre;post;k1,k2;*]"),
    "natbib-style \\rcite param marshaling failed; xml=\n{xml}"
  );
  // graphics-style {Float} params -> properties closure -> attribute holes.
  assert!(
    xml.contains("class=\"gs\"")
      && xml.contains("xscale=\"2")
      && xml.contains("yscale=\"3")
      && xml.contains("SCL"),
    "Gscale-style (\\gsbox) Float-args properties failed; xml=\n{xml}"
  );
  // OptionalKeyVals macro arg: the dict reaches the body as TeX source.
  assert!(
    xml.contains("rust") && xml.contains("KVB"),
    "OptionalKeyVals (\\kvprobe) marshaling failed; xml=\n{xml}"
  );
  // sizer + closure reversion wire without breaking construction.
  assert!(
    xml.contains("class=\"sz\"") && xml.contains("SZ"),
    "sizer/closure-reversion constructor (\\sized) failed; xml=\n{xml}"
  );
  // DefAccent through \lx@applyaccent: o + combining acute.
  assert!(
    xml.contains("ó") || xml.contains("o\u{0301}"),
    "DefAccent (\\racc) failed; xml=\n{xml}"
  );
  // DefMathLigature: := merged to a single ≔ token with the ASSIGN role.
  assert!(
    xml.contains("≔"),
    "DefMathLigature (:=) did not merge; xml=\n{xml}"
  );
  // Matcher-closure DefMathLigature: !! merged via the Node proxy walk.
  assert!(
    xml.contains("‼"),
    "matcher-closure DefMathLigature (!!) did not merge; xml=\n{xml}"
  );
  // Gullet reads from a Rhai macro body.
  assert!(
    xml.contains("G(x:y)"),
    "gullet seams (\\gread ReadOptional/ReadArg) failed; xml=\n{xml}"
  );
  // Replace-closure rewrite: victim detached, replacement inserted.
  assert!(
    xml.contains("REPL") && xml.contains("class=\"replaced\"") && !xml.contains("OLD"),
    "DefRewrite replace-closure (reinsertion) failed; xml=\n{xml}"
  );
  // DefRewrite: the xpath+attributes rule fired at finalization.
  assert!(
    xml.contains("rw-stamp"),
    "DefRewrite xpath/attributes rule did not fire; xml=\n{xml}"
  );
  // XML-parser exposure (#350): \rhrawhtml parsed the (X)HTML snippet into a
  // STRUCTURED subtree (element + attribute + text) inside <ltx:rawhtml>, NOT an
  // escaped `&lt;p&gt;` text blob. `class="lead"` surviving as a real attribute
  // (real quotes) is the structured signal; the `&lt;p` guard rules out the
  // escaped-text failure mode. Proves `document.insertXML` → native
  // `Document::insert_xml` → `append_tree` (Perl's parse_string + appendTree).
  assert!(
    xml.contains("rawhtml")
      && xml.contains("class=\"lead\"")
      && xml.contains("bold")
      && !xml.contains("&lt;p"),
    "insertXML did not splice a parsed XML subtree; xml=\n{xml}"
  );
  // ...and the absorbed subtree kept its OWN namespace. The snippet declares
  // xhtml as a DEFAULT namespace (empty libxml prefix), so this only holds if the
  // node re-creation resolves the namespace URI through the registered
  // code-namespace map (`RegisterNamespace`) instead of assuming an empty prefix
  // means ltx. Mislabelling these `ltx:p`/`ltx:b` would strip exactly what the
  // XHTML post-processor keys on (`copy-foreign` matches `xhtml:*`), silently
  // dropping the raw HTML from the final output.
  assert!(
    xml.contains("http://www.w3.org/1999/xhtml") || xml.contains("xhtml:p"),
    "insertXML lost the snippet's xhtml namespace (mislabelled as ltx?); xml=\n{xml}"
  );
  // The XML-manipulation foundation (#350): `\rhfragment` parsed a two-root
  // FRAGMENT — which Perl's single-node parseChunk rejects — held the parsed
  // nodes across statements (they own their document), EDITED each one while it
  // was still detached, and inserted them. Both `class="frag-*"` values prove all
  // three: the fragment survived whole (two nodes, not one), the nodes were still
  // valid when written to, and the pre-insertion edits reached the document.
  assert!(
    xml.contains("frag-one") && xml.contains("frag-two"),
    "ParseXML fragment round-trip failed (both edited siblings should be present); xml=\n{xml}"
  );
  // ...and walking up from a top-level parsed node found nothing, so the
  // throwaway `_lxfragment` wrapper a multi-root chunk is parsed inside stays
  // invisible to scripts — it must never be reachable, let alone insertable.
  assert!(
    xml.contains("data-top=\"detached\""),
    "parent() of a top-level ParseXML node leaked the fragment wrapper; xml=\n{xml}"
  );
  assert!(
    !xml.contains("_lxfragment"),
    "the internal fragment wrapper reached the document; xml=\n{xml}"
  );

  // Primitive seam: the digestion-time side-effect persisted into State.
  let stored = state::lookup_value("script:x");
  let val = match stored {
    Some(latexml_core::common::store::Stored::String(s)) => {
      Some(latexml_core::common::arena::to_string(s))
    },
    _ => None,
  };
  assert_eq!(
    val.as_deref(),
    Some("hello"),
    "primitive \\setx side-effect not observed in State"
  );

  drop(latexml);
  latexml_core::reset_thread_engine();
}

/// #321: `LookupDefinition(cs).pushBeforeConstruct/pushAfterConstruct` — the
/// BookML shape (`push(@{ $$def{afterConstruct} }, sub{…})`) driven end-to-end
/// through a real conversion, so the construct hooks fire with a LIVE Document.
/// A binding defines `\bmlwrap{X}` → `<text class="w">X</text>`, then patches it
/// to absorb an ordered marker BEFORE construction (into the enclosing node) and
/// another AFTER — proving both construct-hook families run, in order.
const HOOK_SAMPLE: &str = r##"
  DefConstructor("\\bmlwrap{}", "<ltx:text class=\"w\">#1</ltx:text>");
  let d = LookupDefinition("\\bmlwrap");
  d.pushBeforeConstruct(|document| { document.absorbString("BMLB"); });
  d.pushAfterConstruct(|document| { document.absorbString("BMLA"); });

  // The actual BookML shape: patch a REAL kernel constructor (\rule, a locked-or-not
  // engine DefConstructor) — proving LookupDefinition resolves an already-installed
  // kernel def AND the push installs despite the binding-time lock (bindings run
  // UNLOCKED, Perl Package.pm:loadLTXML `local $UNLOCKED = 1`).
  LookupDefinition("\\rule").pushAfterConstruct(|document| { document.absorbString("RULEHOOK"); });
"##;

fn hook_dispatch(request: &str) -> Option<Result<()>> {
  let base = request.split('.').next().unwrap_or(request);
  if base == "lxhooktest" {
    Some(latexml_contrib::script_bindings::load_script(HOOK_SAMPLE).map(|_| ()))
  } else {
    None
  }
}

#[test]
fn lookup_definition_pushes_construct_hooks_end_to_end() {
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  state::set_extra_bindings_dispatch(Rc::new(hook_dispatch));

  let tex = concat!(
    "literal:\\documentclass{article}\\usepackage{lxhooktest}",
    "\\begin{document}\\bmlwrap{MID}\\rule{1pt}{2pt}\\end{document}"
  );
  let doc = latexml
    .convert_file(tex.to_string())
    .expect("conversion with pushed construct hooks should succeed");
  let xml = doc.serialize_to_string();

  use latexml_core::common::error::{LogStatus, get_status};
  assert!(
    get_status(LogStatus::Error) == 0 && get_status(LogStatus::Fatal) == 0,
    "conversion logged errors: {}",
    latexml_core::common::error::get_status_message()
  );

  // Both pushed construct hooks fired, and in order: beforeConstruct's marker
  // precedes the construct's own content, afterConstruct's follows it.
  let (b, m, a) = (xml.find("BMLB"), xml.find("MID"), xml.find("BMLA"));
  assert!(
    b.is_some() && m.is_some() && a.is_some(),
    "a pushed construct hook did not fire; xml=\n{xml}"
  );
  assert!(
    b < m && m < a,
    "construct hooks ran out of order (before < content < after expected); xml=\n{xml}"
  );

  // The BookML case: the afterConstruct pushed onto the REAL kernel `\rule`
  // fired — proving the push installed over a kernel definition (lock bypassed
  // during binding load) and ran during the real conversion.
  assert!(
    xml.contains("<rule") && xml.contains("RULEHOOK"),
    "pushAfterConstruct onto the kernel \\rule did not fire; xml=\n{xml}"
  );

  drop(latexml);
  latexml_core::reset_thread_engine();
}

/// Option-bag parity, end-to-end: `DefPrimitive`/`DefMath` accept
/// `beforeDigest`/`afterDigest` closures and `DefEnvironment` accepts
/// `afterDigestBody` — the same flexary, unordered options the compile-time
/// `Def*!` macros take. Driven through a real conversion (afterDigestBody needs a
/// captured environment body). Each hook records a state side-effect we read back.
/// The Document XML surface a COMPILE-TIME binding (`_sty.rs`/`_cls.rs`) uses,
/// exposed to the runtime layer so the two do not diverge (issue #350: "expose it
/// in rhai in full generality, so that the interfaces are comparable in the
/// runtime and compile-time layers"). Every method below is named after its Perl
/// `Core/Document.pm` original.
const XMLAPI_SAMPLE: &str = r##"
  // A tagged block to query and mutate:
  //   <ltx:text class="#1"><ltx:emph>#1-inner</ltx:emph></ltx:text>
  DefConstructor("\\xqbuild{}", |document, cls| {
    let c = ToString(cls);
    document.openElement("ltx:text");
    document.setAttribute("class", c);
    document.openElement("ltx:emph");
    document.absorbString(c + "-inner");
    document.closeElement("ltx:emph");
    document.closeElement("ltx:text");
  });

  // insertElement (Perl `insertElement`) — the element counterpart that
  // `insertXML` is named after — plus addClass, generateID, getNode, getElement.
  DefConstructor("\\xqinsert", |document| {
    let n = document.insertElement("ltx:text", #{ class: "ins-elem" });
    document.addClass(n, "ins-extra");
    document.generateID(n, "xq");
    assign_global("xq:insqname", n.qname());
    assign_global("xq:insid", n.getAttribute("xml:id"));
    assign_global("xq:cur", document.getNode().qname());
    assign_global("xq:elem", document.getElement().qname());
  });

  // findnodes / findnode (Perl `findnodes`/`findnode`), whole-document and
  // scoped to a node — the query half a script had no way to reach before.
  DefConstructor("\\xqfind", |document| {
    let hits = document.findnodes("//ltx:text[@class]");
    assign_global("xq:count", "" + hits.len());
    let one = document.findnode("//ltx:text[contains(@class,'ins-elem')]");
    assign_global("xq:one", one.getAttribute("class"));
    let keep = document.findnode("//ltx:text[@class='keep']");
    assign_global("xq:scoped", "" + document.findnodes(".//ltx:emph", keep).len());
    assign_global("xq:missing",
      if type_of(document.findnode("//ltx:nosuch")) == "()" { "unit" } else { "BAD" });
  });

  // Structural manipulation: rename / remove / unwrap / wrap / appendClone /
  // replaceNode (Perl renameNode, removeNode, unwrapNodes, wrapNodes,
  // appendClone, replaceNode).
  DefConstructor("\\xqmutate", |document| {
    document.renameNode(document.findnode("//ltx:text[@class='rename']"), "ltx:emph");
    document.removeNode(document.findnode("//ltx:text[@class='remove']"));
    document.unwrapNodes(document.findnode("//ltx:text[@class='unwrap']"));
    let w = document.wrapNodes("ltx:text", [document.findnode("//ltx:text[@class='wrap']")]);
    document.addClass(w, "wrapper");
    document.appendClone(document.findnode("//ltx:text[@class='keep']"),
                         [document.findnode("//ltx:text[@class='clone']")]);
    document.replaceNode(document.findnode("//ltx:text[@class='replace']"),
                         [document.findnode("//ltx:text[@class='mover']")]);
  });

  // openElementAt / closeElementAt (Perl `openElementAt`/`closeElementAt`):
  // build at an explicit node instead of the current insertion point.
  DefConstructor("\\xqat", |document| {
    let target = document.findnode("//ltx:text[@class='at']");
    let n = document.openElementAt(target, "ltx:emph", #{ class: "at-child" });
    document.closeElementAt(n);
  });

  // A PARSED node may only enter the document through insertXML, which
  // re-creates it. Handing one to a node-splicing method would move a node
  // still owned by the throwaway parse document into ours — a use-after-free
  // once the script drops the handle. Must be a clean error, not a segfault.
  DefConstructor("\\xqforeign", |document| {
    let parsed = ParseXML("<b>x</b>");
    let target = document.findnode("//ltx:text[@class='keep']");
    try {
      document.replaceNode(target, parsed);
      assign_global("xq:foreign", "ACCEPTED-A-PARSED-NODE");
    } catch (e) {
      assign_global("xq:foreign", "rejected");
      assign_global("xq:foreignmsg", "" + e);
    }
  });

  // renameNode on an ORPHAN: `Document::rename_node` panics on a node with no
  // parent, which from an untrusted script would abort the conversion (and with
  // panic=abort, the process). Must be a clean error instead.
  DefConstructor("\\xqorphan", |document| {
    let orphan = document.findnode("//ltx:text[@class='orphan']");
    document.removeNode(orphan);
    try {
      document.renameNode(orphan, "ltx:emph");
      assign_global("xq:orphan", "RENAMED-AN-ORPHAN");
    } catch (e) {
      assign_global("xq:orphan", "rejected");
      assign_global("xq:orphanmsg", "" + e);
    }
  });
"##;

fn xmlapi_dispatch(request: &str) -> Option<Result<()>> {
  let base = request.split('.').next().unwrap_or(request);
  if base == "lxxmlapitest" {
    Some(latexml_contrib::script_bindings::load_script(XMLAPI_SAMPLE).map(|_| ()))
  } else {
    None
  }
}

/// The runtime layer reaches the same Document XML surface the compile-time
/// layer does — query (`findnodes`/`findnode`), element insertion
/// (`insertElement`), and structural edits — and the two hazards that surface
/// creates for an untrusted script are contained rather than fatal.
#[test]
fn document_xml_api_matches_the_compile_time_surface() {
  use latexml_core::common::error::{LogStatus, get_status};
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  state::set_extra_bindings_dispatch(Rc::new(xmlapi_dispatch));

  let tex = concat!(
    "literal:\\documentclass{article}\\usepackage{lxxmlapitest}\\begin{document}",
    "\\xqinsert ",
    "\\xqbuild{keep}\\xqbuild{rename}\\xqbuild{remove}\\xqbuild{unwrap}",
    "\\xqbuild{wrap}\\xqbuild{clone}\\xqbuild{replace}\\xqbuild{mover}",
    "\\xqbuild{at}\\xqbuild{orphan}",
    "\\xqfind \\xqmutate \\xqat \\xqforeign \\xqorphan ",
    "\\end{document}"
  );
  let doc = latexml
    .convert_file(tex.to_string())
    .expect("the Document XML surface must not abort the conversion");
  let xml = doc.serialize_to_string();

  let g = |k: &str| match state::lookup_value(k) {
    Some(latexml_core::common::store::Stored::String(s)) => {
      latexml_core::common::arena::to_string(s)
    },
    _ => String::new(),
  };

  // ── insertElement / addClass / generateID / getNode / getElement ──
  assert_eq!(
    g("xq:insqname"),
    "ltx:text",
    "insertElement returned the new node"
  );
  assert!(
    g("xq:insid").contains("xq"),
    "generateID did not stamp the requested prefix, or getAttribute could not \
     read the namespaced xml:id back; got {:?}",
    g("xq:insid")
  );
  assert!(
    xml.contains("ins-elem") && xml.contains("ins-extra"),
    "insertElement/addClass did not reach the document; xml=\n{xml}"
  );
  assert!(
    !g("xq:cur").is_empty() && !g("xq:elem").is_empty(),
    "getNode/getElement returned nothing: cur={:?} elem={:?}",
    g("xq:cur"),
    g("xq:elem")
  );

  // ── findnodes / findnode ──
  assert!(
    g("xq:count").parse::<usize>().unwrap_or(0) >= 10,
    "findnodes did not see the built blocks; count={:?}",
    g("xq:count")
  );
  assert!(
    g("xq:one").contains("ins-elem"),
    "findnode returned the wrong node; got {:?}",
    g("xq:one")
  );
  assert_eq!(
    g("xq:scoped"),
    "1",
    "findnodes scoped to a node over-matched"
  );
  assert_eq!(
    g("xq:missing"),
    "unit",
    "findnode with no match must be (), not an error"
  );

  // ── structural manipulation ──
  assert!(
    xml.contains("<ltx:emph class=\"rename\"") || xml.contains("class=\"rename\""),
    "renameNode did not retag the node; xml=\n{xml}"
  );
  assert!(
    !xml.contains("remove-inner"),
    "removeNode left its subtree behind; xml=\n{xml}"
  );
  assert!(
    xml.contains("unwrap-inner") && !xml.contains("class=\"unwrap\""),
    "unwrapNodes must drop the wrapper but keep its children; xml=\n{xml}"
  );
  assert!(
    xml.contains("wrapper"),
    "wrapNodes did not create the wrapping element; xml=\n{xml}"
  );
  assert_eq!(
    xml.matches("clone-inner").count(),
    2,
    "appendClone should have produced a second copy; xml=\n{xml}"
  );
  assert!(
    xml.contains("mover-inner"),
    "replaceNode lost the replacement node; xml=\n{xml}"
  );

  // ── openElementAt / closeElementAt ──
  assert!(
    xml.contains("at-child"),
    "openElementAt/closeElementAt did not build at the target node; xml=\n{xml}"
  );

  // ── the two hazards are contained ──
  assert_eq!(
    g("xq:foreign"),
    "rejected",
    "a PARSED node was spliced in directly — that is a use-after-free once the \
     script drops its handle; it must be routed through insertXML"
  );
  // ...and it was OUR guard that refused it, not some unrelated failure.
  assert!(
    g("xq:foreignmsg").contains("insertXML") && g("xq:foreignmsg").contains("ParseXML"),
    "replaceNode failed for the wrong reason: {:?}",
    g("xq:foreignmsg")
  );
  assert_eq!(
    g("xq:orphan"),
    "rejected",
    "renameNode on an orphan reached Document::rename_node's panic"
  );
  assert!(
    g("xq:orphanmsg").contains("no parent"),
    "renameNode failed for the wrong reason: {:?}",
    g("xq:orphanmsg")
  );
  assert!(
    get_status(LogStatus::Fatal) == 0,
    "the XML surface escalated to Fatal: {}",
    latexml_core::common::error::get_status_message()
  );

  drop(latexml);
  latexml_core::reset_thread_engine();
}

const OPTBAG_SAMPLE: &str = r##"
  DefPrimitive("\\optbagprim", || { AssignValue("optbag:body", "X", "global"); }, #{
    beforeDigest: || { AssignValue("optbag:before", "B", "global"); },
    afterDigest:  || { AssignValue("optbag:after", "A", "global"); }
  });
  DefEnvironment("{optbagenv}", "<ltx:text class=\"optbagenv\">#body</ltx:text>", #{
    mode: "text",
    afterDigestBody: || { AssignValue("optbag:afterbody", "yes", "global"); }
  });
"##;

fn optbag_dispatch(request: &str) -> Option<Result<()>> {
  let base = request.split('.').next().unwrap_or(request);
  if base == "lxoptbagtest" {
    Some(latexml_contrib::script_bindings::load_script(OPTBAG_SAMPLE).map(|_| ()))
  } else {
    None
  }
}

#[test]
fn option_bag_digest_hooks_end_to_end() {
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  state::set_extra_bindings_dispatch(Rc::new(optbag_dispatch));

  let tex = concat!(
    "literal:\\documentclass{article}\\usepackage{lxoptbagtest}",
    "\\begin{document}\\optbagprim\\begin{optbagenv}Z\\end{optbagenv}\\end{document}"
  );
  let doc = latexml
    .convert_file(tex.to_string())
    .expect("conversion with option-bag digest hooks should succeed");
  let _ = doc.serialize_to_string();

  let g = |k: &str| match state::lookup_value(k) {
    Some(latexml_core::common::store::Stored::String(s)) => {
      latexml_core::common::arena::to_string(s)
    },
    _ => String::new(),
  };
  // DefPrimitive option-bag beforeDigest + body + afterDigest all fired.
  assert_eq!(
    g("optbag:before"),
    "B",
    "DefPrimitive beforeDigest option did not run"
  );
  assert_eq!(g("optbag:body"), "X", "DefPrimitive body did not run");
  assert_eq!(
    g("optbag:after"),
    "A",
    "DefPrimitive afterDigest option did not run"
  );
  // DefEnvironment afterDigestBody fired after the captured body digested.
  assert_eq!(
    g("optbag:afterbody"),
    "yes",
    "DefEnvironment afterDigestBody option did not run"
  );

  drop(latexml);
  latexml_core::reset_thread_engine();
}

/// Default `.rhai` FILE discovery (no embedder dispatcher): a
/// `<name>.sty.rhai` next to the document is found via the searchpath
/// machinery and loaded on `\usepackage{<name>}` — the downstream
const BADXML_SAMPLE: &str = r##"
  // Two ways to get malformed markup into the document, each of which MUST
  // degrade only itself: parse-and-insert in one call, and the standalone parser.
  DefConstructor("\\rhbadxml", |document| {
    document.insertXML("<p>unclosed");
  });
  DefConstructor("\\rhbadparse", |document| {
    let nodes = ParseXML("<p>a & b</p>");
    document.insertXML(nodes);
  });
"##;

fn badxml_dispatch(request: &str) -> Option<Result<()>> {
  let base = request.split('.').next().unwrap_or(request);
  if base == "lxbadxmltest" {
    Some(latexml_contrib::script_bindings::load_script(BADXML_SAMPLE).map(|_| ()))
  } else {
    None
  }
}

/// Malformed markup DEGRADES ONE BINDING, it does not abort the conversion and
/// it does not silently vanish.
///
/// This is the contract that makes rejecting malformed input (rather than
/// letting libxml "recover" it, which silently DESTROYS content — see
/// `common::xml::parse_chunk`) safe to impose: the author gets a loud `Error:`
/// naming the snippet, everything around it still converts.
#[test]
fn malformed_insert_xml_degrades_the_binding_not_the_conversion() {
  use latexml_core::common::error::{LogStatus, get_status};
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  state::set_extra_bindings_dispatch(Rc::new(badxml_dispatch));

  let tex = concat!(
    "literal:\\documentclass{article}\\usepackage{lxbadxmltest}",
    "\\begin{document}BEFORE \\rhbadxml MIDDLE \\rhbadparse AFTER\\end{document}"
  );
  let doc = latexml
    .convert_file(tex.to_string())
    .expect("a malformed chunk must not abort the conversion");
  let xml = doc.serialize_to_string();

  // The surrounding document is intact — the failure was contained.
  assert!(
    xml.contains("BEFORE") && xml.contains("MIDDLE") && xml.contains("AFTER"),
    "a malformed chunk swallowed neighbouring content; xml=\n{xml}"
  );
  // Nothing was salvaged into the tree: recovery mode would have inserted a
  // repaired `<p>` here, which is precisely the silent content mangling we
  // refuse. `insertXML` inserts all of the markup or none of it.
  assert!(
    !xml.contains("unclosed"),
    "malformed markup was salvaged into the document instead of rejected; xml=\n{xml}"
  );
  // And it was LOUD: reported as an error, never a silent skip. (`\rhbadparse`
  // raises its own script error from ParseXML, so both paths are covered.)
  assert!(
    get_status(LogStatus::Error) > 0,
    "malformed markup was dropped silently — no Error: was logged"
  );
  assert!(
    get_status(LogStatus::Fatal) == 0,
    "a malformed chunk escalated to Fatal: {}",
    latexml_core::common::error::get_status_message()
  );

  drop(latexml);
  latexml_core::reset_thread_engine();
}

/// customize-without-recompiling story for the single-file binary.
#[test]
fn script_binding_discovered_from_file() {
  use latexml::converter::Converter;
  use latexml_core::common::{Config, OutputFormat};

  let dir = std::env::temp_dir().join("lx_rhai_discovery");
  std::fs::create_dir_all(&dir).expect("tempdir");
  std::fs::write(
    dir.join("lxdisc.sty.rhai"),
    r#"DefMacro("\\discmark", || "DISCOVERED");"#,
  )
  .expect("write rhai");
  let tex = dir.join("disc.tex");
  std::fs::write(
    &tex,
    "\\documentclass{article}\\usepackage{lxdisc}\\begin{document}\\discmark\\end{document}",
  )
  .expect("write tex");

  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::XML,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(tex.to_string_lossy().to_string());
  let xml = r.result.expect("conversion should produce a document");
  assert!(
    xml.contains("DISCOVERED"),
    "discovered .rhai binding did not load/expand; xml=\n{xml}"
  );
  latexml_core::reset_thread_engine();
}
