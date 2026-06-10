//! End-to-end validation of runtime Rhai script bindings through a real
//! conversion (docs/script_bindings_plan.md, milestone M4).
//!
//! A sample binding (one `DefMacro`, one `DefConstructor`) is authored in Rhai
//! and loaded at runtime via the *extra* binding dispatcher when the document
//! `\usepackage{lxrhaitest}`s it — exactly the path real contrib packages use.
//! We then assert on the produced XML: the macro must expand and the constructor
//! must emit its element.
#![cfg(feature = "runtime-bindings")]

use std::rc::Rc;

use latexml::core_interface::DigestionAPI;
use latexml_core::common::error::Result;
use latexml_core::state;
use latexml_core::{Core, CoreOptions};

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

  // amsmath-style numbered construct: RefStepCounter feeding #tags, plus a
  // string reversion (the `\@@multline` option shape, scaled down).
  NewCounter("rqeq");
  DefConstructor("\\numbered{}", "<ltx:text class=\"eq\">#tags #1</ltx:text>", #{
    properties: |x| RefStepCounter("rqeq"),
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

  // DefRewrite (data form): stamp every biography note at finalization.
  DefRewrite(#{ xpath: "descendant-or-self::ltx:note[@role='biography'][not(@class)]",
                attributes: #{ class: "rw-stamp" } });
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
    "\\begin{document}\\twicex{ab} \\myemph{hi} \\mytext{zz} \\wrap{\\myemph{deep}} \\note{N} \\rot{xx}{yy}{zz2} \\cif{Y}\\cif{} ",
    "body\\fnote{*}{Marked}more\\fnote{}{Plain} \\pnote{dyn} \\snote{st} \\mypi{d1} ",
    "\\begin{rquote}Quotable\\end{rquote} \\begin{bio}{Ada}Pioneer\\end{bio} ",
    "\\begin{biop}{Ada}Idiom\\end{biop} \\begin{rbox}Boxed\\end{rbox} ",
    "\\begin{rproof}QED-body\\end{rproof} \\numbered{NUM} \\rcite*[pre][post]{k1,k2} ",
    "\\gsbox{2}{3}{SCL} \\kvprobe[lang=rust]{KVB} \\sized{SZ} \\racc{o} $a := b$ $c!!$ \\gread[x]{y} ",
    "\\endreferences \\setx{hello}\\end{document}"
  );
  let doc = latexml
    .convert_file(tex.to_string())
    .expect("conversion with a script binding should succeed");
  let xml = doc.serialize_to_string();

  // NB: the serializer emits the LaTeXML namespace as the default (no `ltx:`
  // prefix), so elements appear unprefixed.
  assert!(xml.contains("abab"), "macro \\twicex did not expand; xml=\n{xml}");
  assert!(
    xml.contains("<emph>hi</emph>"),
    "imperative constructor \\myemph did not emit its element; xml=\n{xml}"
  );
  assert!(
    xml.contains("class=\"rhai\"") && xml.contains("zz"),
    "template constructor \\mytext did not emit; xml=\n{xml}"
  );
  // Re-entrancy (GATE-1): the nested script constructor ran inside another
  // script constructor's body without a borrow panic.
  assert!(
    xml.contains("<emph>deep</emph>"),
    "re-entrant nested script constructor failed; xml=\n{xml}"
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
  assert!(unmarked, "footnote port: unmarked note should have no mark attribute; xml=\n{xml}");
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
  let opt = match latexml_core::state::lookup_value("rh:opt") {
    Some(latexml_core::common::store::Stored::String(s)) => latexml_core::common::arena::to_string(s),
    _ => String::from("<unset>"),
  };
  assert_eq!(opt, "draft-on", "DeclareOption+ProcessOptions did not fire for [draft]");
  // IEEEproof port: properties closure digests the title; #title/#font holes.
  assert!(
    xml.contains("<proof") && xml.contains("Proof:") && xml.contains("QED-body"),
    "IEEEproof-style ({{rproof}}) properties-digestion failed; xml=\n{xml}"
  );
  // amsmath-style: RefStepCounter feeds #tags (the tag renders as "1").
  assert!(
    xml.contains("class=\"eq\"") && xml.contains("NUM"),
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
  // DefRewrite: the xpath+attributes rule fired at finalization.
  assert!(
    xml.contains("rw-stamp"),
    "DefRewrite xpath/attributes rule did not fire; xml=\n{xml}"
  );

  // Primitive seam: the digestion-time side-effect persisted into State.
  let stored = latexml_core::state::lookup_value("script:x");
  let val = match stored {
    Some(latexml_core::common::store::Stored::String(s)) => Some(latexml_core::common::arena::to_string(s)),
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

/// Default `.rhai` FILE discovery (no embedder dispatcher): a
/// `<name>.sty.rhai` next to the document is found via the searchpath
/// machinery and loaded on `\usepackage{<name>}` — the downstream
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
  let cfg = Config { format: OutputFormat::XML, ..Config::default() };
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
