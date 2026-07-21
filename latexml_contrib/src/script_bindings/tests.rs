//! Unit tests for the script-binding surface (real State, no Document).

use latexml_core::{
  gullet,
  state::{State, StateOptions, set_state},
};

use super::*;

/// Bootstrap enough engine to validate prototypes (`{}` etc. need the base
/// parameter-type registry). In a real conversion this is already loaded.
fn fresh_state() {
  set_state(State::new(StateOptions::default()));
  latexml_core::stomach::initialize_stomach();
  latexml_engine::base::load_definitions().expect("bootstrap base parameter types");
}

/// The Wave-A pool surface (state, Let/RawTeX, counters, token helpers):
/// every registration must round-trip through a real script execution.
#[test]
fn pool_surface_state_counters_tokens() {
  fresh_state();
  load_script(
    r##"
      AssignValue("ws:k", "v1");
      assign_global("ws:str", LookupString("ws:k"));

      RawTeX("\\def\\wsfoo{FOO}");
      Let("\\wsalias", "\\wsfoo");
      assign_global("ws:def", if IsDefined("\\wsfoo") { "yes" } else { "no" });
      assign_global("ws:alias", if IsDefined("\\wsalias") { "yes" } else { "no" });
      assign_global("ws:xeq", if XEquals("\\wsalias", "\\wsfoo") { "eq" } else { "ne" });
      assign_global("ws:expand", UnTeX(Expand(TokenizeInternal("\\wsfoo"))));

      NewCounter("wsctr");
      StepCounter("wsctr");
      StepCounter("wsctr");
      AddToCounter("wsctr", 3);
      assign_global("ws:cv", CounterValue("wsctr").to_string());
      let refmap = RefStepCounter("wsctr");
      assign_global("ws:ref", if ("tags" in refmap) && ("id" in refmap) { "has" } else { "none" });
      ResetCounter("wsctr");
      assign_global("ws:cv0", CounterValue("wsctr").to_string());

      assign_global("ws:digest", ToString(DigestText("ab")));

      assign_global("ws:cat", LookupCatcode("a").to_string());
      AssignCatcode("~", 12);
      assign_global("ws:cat2", LookupCatcode("~").to_string());
      assign_global("ws:meaning", if LookupMeaning("\\wsfoo") == "" { "none" } else { "some" });
      let idmap = RefStepID("wsctr");
      assign_global("ws:refid", if "id" in idmap { "has" } else { "none" });

      AssignMapping("wsmap", "alpha", "A1");
      assign_global("ws:map", LookupMapping("wsmap", "alpha"));
      assign_global("ws:kv1", GetKeyVal("lang=rust, size={1, 2}", "size"));
      let kvm = GetKeyVals("lang=rust, size={1, 2}");
      assign_global("ws:kv2", kvm.lang);
    "##,
  )
  .expect("wave-A surface script should load cleanly");
  assert_eq!(lookup_str("ws:str"), "v1", "AssignValue/LookupString");
  assert_eq!(lookup_str("ws:def"), "yes", "RawTeX \\def + IsDefined");
  assert_eq!(lookup_str("ws:alias"), "yes", "Let installs the alias");
  assert_eq!(lookup_str("ws:xeq"), "eq", "XEquals alias == \\wsfoo");
  assert_eq!(lookup_str("ws:expand"), "FOO", "Expand through the gullet");
  assert_eq!(lookup_str("ws:cv"), "5", "2 steps + 3 = 5");
  assert_eq!(
    lookup_str("ws:ref"),
    "has",
    "RefStepCounter returns tags+id"
  );
  assert_eq!(lookup_str("ws:cv0"), "0", "ResetCounter zeroes");
  assert_eq!(
    lookup_str("ws:digest"),
    "ab",
    "DigestText -> Digested handle"
  );
  assert_eq!(lookup_str("ws:cat"), "11", "letter catcode reads as 11");
  assert_eq!(lookup_str("ws:cat2"), "12", "AssignCatcode ~ -> OTHER");
  assert_eq!(
    lookup_str("ws:meaning"),
    "some",
    "LookupMeaning sees \\wsfoo"
  );
  assert_eq!(lookup_str("ws:refid"), "has", "RefStepID returns id");
  assert_eq!(lookup_str("ws:map"), "A1", "AssignMapping/LookupMapping");
  assert_eq!(lookup_str("ws:kv1"), "1, 2", "GetKeyVal brace-aware value");
  assert_eq!(lookup_str("ws:kv2"), "rust", "GetKeyVals map access");
}

/// Wave-B definition forms: DefRegister (count + dimen), DefConditional
/// (Rhai test driven from real TeX), DefKeyVal, DefLigature, DefMath.
#[test]
fn pool_surface_definition_forms() {
  fresh_state();
  load_script(
    r##"
      DefRegister("\\wbcount", 42);
      DefRegister("\\wbdimen", "5pt");
      DefKeyVal("WB", "color", "");
      DefLigature("ff", "F");
      DefMath("\\wbsum", "∑", #{ role: "SUMOP", meaning: "sum" });
      DefConditional("\\ifwb{}", |x| x == "on");
      DefMacro("\\wbprobe{}", |x| "\\ifwb{" + x + "}YES\\else NO\\fi");
    "##,
  )
  .expect("wave-B surface script should load cleanly");
  // Registers installed and readable through the native register store.
  assert!(
    latexml_core::state::lookup_definition(&latexml_core::T_CS!("\\wbcount"))
      .expect("lookup")
      .is_some(),
    "\\wbcount register installed"
  );
  assert!(
    latexml_core::state::lookup_definition(&latexml_core::T_CS!("\\wbdimen"))
      .expect("lookup")
      .is_some(),
    "\\wbdimen register installed"
  );
  assert!(
    latexml_core::state::lookup_definition(&latexml_core::T_CS!("\\wbsum"))
      .expect("lookup")
      .is_some(),
    "DefMath \\wbsum installed"
  );
  // The conditional drives real expansion: \ifwb{on} -> YES, \ifwb{off} -> NO.
  let on = gullet::do_expand(mouth::tokenize_internal("\\wbprobe{on}")).expect("expand on");
  assert_eq!(on.to_string().trim(), "YES", "conditional true branch");
  let off = gullet::do_expand(mouth::tokenize_internal("\\wbprobe{off}")).expect("expand off");
  assert_eq!(off.to_string().trim(), "NO", "conditional false branch");
}

fn lookup_str(key: &str) -> String {
  match latexml_core::state::lookup_value(key) {
    Some(Stored::String(s)) => arena::to_string(s),
    _ => String::new(),
  }
}

/// Conformance: the *same* `afterDigest` constructor defined two ways —
/// macro-style (calling `ConstructorBuilder` directly, as `DefConstructor!`
/// lowers) and via Rhai (which now also routes through `ConstructorBuilder`) —
/// produces identical behaviour. This is the anti-drift guard between
/// `setup_binding_language.rs` and the Rhai layer.
#[test]
fn builder_conformance_macro_style_vs_rhai_afterdigest() {
  use latexml_core::binding::def::builder::{ConstructorBuilder, OptionValue};

  fresh_state();

  // (1) Macro-style: build \mfoo via ConstructorBuilder; afterDigest is a
  // native Rust closure reading the whatsit's first arg.
  let after: DigestionClosure = Rc::new(|w: &mut Whatsit| -> Result<Vec<Digested>> {
    let s = match w.get_arg(1) {
      Some(d) => d.untex()?,
      None => String::new(),
    };
    latexml_core::state::assign_value("conf:m", s, Some(Scope::Global));
    Ok(Vec::new())
  });
  ConstructorBuilder::new("\\mfoo{}")
    .expect("builder")
    .replacement(template_replacement("<ltx:text>#1</ltx:text>").expect("template"))
    .set_option("mode", OptionValue::Str("text".to_string()))
    .expect("set_option")
    .after_digest(after)
    .install()
    .expect("install");
  latexml_core::stomach::digest(mouth::tokenize_internal(r"\mfoo{ZED}")).expect("digest mfoo");

  // (2) Rhai: the equivalent \rfoo — same builder under the hood; afterDigest
  // reads the whatsit via whatsit().
  load_script(
    r#"DefConstructor("\\rfoo{}", "<ltx:text>#1</ltx:text>", #{
           mode: "text",
           afterDigest: || { assign_global("conf:r", whatsit().argString(1)); }
         });"#,
  )
  .expect("load");
  latexml_core::stomach::digest(mouth::tokenize_internal(r"\rfoo{ZED}")).expect("digest rfoo");

  let m = lookup_str("conf:m");
  let r = lookup_str("conf:r");
  assert_eq!(m, "ZED", "macro-style afterDigest did not capture the arg");
  assert_eq!(
    m, r,
    "macro-style and Rhai afterDigest diverged: {m:?} vs {r:?}"
  );
  latexml_core::reset_thread_engine();
}

#[test]
fn m1_script_macro_expands_through_real_gullet() {
  fresh_state();
  let n = load_script(
    r#"
        DefMacro("\\twice{}", |x| x + x);
        DefMacro("\\greet{}", |name| "Hello, " + name + "!");
      "#,
  )
  .expect("load_script");
  assert_eq!(n, 2);

  let out = gullet::do_expand(mouth::tokenize_internal(r"\twice{ab}")).expect("expand twice");
  assert_eq!(out.to_string(), "abab");

  let out = gullet::do_expand(mouth::tokenize_internal(r"\greet{World}")).expect("expand greet");
  assert_eq!(out.to_string(), "Hello, World!");

  latexml_core::reset_thread_engine();
}

#[test]
fn m1_expansion_to_control_sequence_is_faithful() {
  fresh_state();
  load_script(r#"DefMacro("\\emphx{}", |x| "\\textit{" + x + "}");"#).expect("load");
  let out = gullet::do_expand(mouth::tokenize_internal(r"\emphx{hi}")).expect("expand");
  assert_eq!(out.to_string(), r"\textit{hi}");
  latexml_core::reset_thread_engine();
}

#[test]
fn cache_reuses_compiled_script_and_still_wires() {
  fresh_state();
  let src = r#"DefMacro("\\dup{}", |x| x + x);"#;
  assert_eq!(load_script(src).expect("first load"), 1);
  // Second load is a cache hit (no recompile) but still installs the binding.
  assert_eq!(load_script(src).expect("second load"), 1);
  let out = gullet::do_expand(mouth::tokenize_internal(r"\dup{yo}")).expect("expand");
  assert_eq!(out.to_string(), "yoyo");
  latexml_core::reset_thread_engine();
}

#[test]
fn load_file_reads_and_installs() {
  fresh_state();
  let path = std::env::temp_dir().join("lx_script_bindings_load_file_test.sty.rhai");
  std::fs::write(&path, r#"DefMacro("\\trip{}", |x| x + x + x);"#).expect("write temp");
  let n = load_file(path.to_str().unwrap()).expect("load_file");
  assert_eq!(n, 1);
  let out = gullet::do_expand(mouth::tokenize_internal(r"\trip{ab}")).expect("expand");
  assert_eq!(out.to_string(), "ababab");
  let _ = std::fs::remove_file(&path);
  latexml_core::reset_thread_engine();
}

/// Translation of the Perl doc example:
///   DeclareOption('opt', sub { Digest(Tokenize('\relax')); });
/// We add a marker assignment so the test can observe the body ran.
#[test]
fn declare_option_registers_and_runs() {
  fresh_state();
  load_script(
    r#"DeclareOption("opt", || {
           Digest(Tokenize("\\relax"));
           assign_global("script:opt_ran", "yes");
         });"#,
  )
  .expect("load");
  // Invoke the option by digesting its generated \ds@opt primitive.
  latexml_core::stomach::digest(mouth::tokenize_internal(r"\ds@opt")).expect("digest \\ds@opt");
  let ran = match latexml_core::state::lookup_value("script:opt_ran") {
    Some(Stored::String(s)) => arena::to_string(s),
    _ => String::new(),
  };
  assert_eq!(
    ran, "yes",
    "DeclareOption body (Tokenize+Digest) did not run"
  );
  latexml_core::reset_thread_engine();
}

/// The `DefConstructor` option-bag form: a trailing Rhai map `#{ … }` with
/// named options (any order, omittable) including a closure-valued
/// `afterDigest` — the analog of the macro's `key => value` options.
#[test]
fn constructor_options_map_runs_afterdigest() {
  fresh_state();
  // The parameterless afterDigest body reaches the in-flight whatsit via
  // whatsit() — referencing context only when needed ("omit as implied").
  load_script(
    r#"DefConstructor("\\opt{}", "<ltx:text>#1</ltx:text>", #{
           mode: "text",
           afterDigest: || { assign_global("script:cad", whatsit().argString(1)); }
         });"#,
  )
  .expect("load");
  latexml_core::stomach::digest(mouth::tokenize_internal(r"\opt{HELLO}")).expect("digest \\opt");
  let ran = match latexml_core::state::lookup_value("script:cad") {
    Some(Stored::String(s)) => arena::to_string(s),
    _ => String::new(),
  };
  assert_eq!(
    ran, "HELLO",
    "afterDigest body did not read the whatsit arg via whatsit()"
  );
  latexml_core::reset_thread_engine();
}

/// Regression for #314: `LookupTokens("class_options")` panicked with
/// "RefCell already borrowed". `class_options` is a `Stored::VecDequeStored`,
/// whose branch in `state::lookup_tokens` reverts each item to Tokens via
/// `mouth::tokenize_internal` — which takes a *mutable* STATE borrow — while
/// the outer immutable `state!()` borrow was still held. The fix drops the
/// borrow before the conversion (mirroring the `Stored::String` branch).
#[test]
fn lookup_tokens_on_vecdeque_value_does_not_panic() {
  fresh_state();
  // Populate class_options exactly as the class-loader does: a queue of
  // option strings (Stored::VecDequeStored of Stored::String).
  latexml_core::state::push_value("class_options", "a4paper").expect("push a4paper");
  latexml_core::state::push_value("class_options", "12pt").expect("push 12pt");
  load_script(r#"assign_global("ct:opts", UnTeX(LookupTokens("class_options")));"#)
    .expect("LookupTokens on a VecDequeStored value must not panic");
  assert_eq!(
    lookup_str("ct:opts"),
    "a4paper12pt",
    "LookupTokens should revert the queued option strings to their tokens"
  );
  latexml_core::reset_thread_engine();
}

/// #315: `LookupString` is scalar-only. On a list value (`class_options` is a
/// `Stored::VecDequeStored`) it must return "" — never the internal
/// `VecDequeStored[…]` Debug repr it used to leak. Structural access is via
/// `LookupValue` (below), mirroring Perl's `LookupValue` returning the arrayref.
#[test]
fn lookup_string_on_list_value_is_empty_not_leaked() {
  fresh_state();
  latexml_core::state::push_value("class_options", "a4paper").expect("push a4paper");
  latexml_core::state::push_value("class_options", "12pt").expect("push 12pt");
  load_script(r#"assign_global("ct:s", LookupString("class_options"));"#)
    .expect("LookupString on a list value must not panic or leak");
  let got = lookup_str("ct:s");
  assert!(
    !got.contains("VecDequeStored") && !got.contains("Stored["),
    "LookupString must not leak the internal enum representation, got {got:?}"
  );
  assert_eq!(
    got, "",
    "LookupString is scalar-only; a list value yields \"\""
  );
  latexml_core::reset_thread_engine();
}

/// #315: `LookupValue` exposes a list value AS a Rhai array (mirroring Perl's
/// `LookupValue` returning the arrayref), so the caller reads/iterates/joins it
/// structurally — no invented separator baked into the reversion. Here the
/// script indexes it and joins with its own `|`.
#[test]
fn lookup_value_on_list_returns_rhai_array() {
  fresh_state();
  latexml_core::state::push_value("class_options", "a4paper").expect("push a4paper");
  latexml_core::state::push_value("class_options", "12pt").expect("push 12pt");
  load_script(
    r#"
    let opts = LookupValue("class_options");
    assign_global("ct:0", opts[0]);
    assign_global("ct:1", opts[1]);
    let joined = "";
    for o in opts { if joined != "" { joined += "|"; } joined += o; }
    assign_global("ct:joined", joined);
    "#,
  )
  .expect("LookupValue on a list value must return an indexable Rhai array");
  assert_eq!(lookup_str("ct:0"), "a4paper", "array element 0");
  assert_eq!(lookup_str("ct:1"), "12pt", "array element 1");
  assert_eq!(
    lookup_str("ct:joined"),
    "a4paper|12pt",
    "the caller joins the array with its own separator — structural, not a baked-in comma"
  );
  latexml_core::reset_thread_engine();
}

/// #319: the diagnostics surface beyond `Warn`/`Error` — `Info`, the
/// `Note`/`Progress` family (side-effecting, must load and run cleanly), and
/// `Fatal` (must abort the script, mirroring Perl `Fatal` which dies).
#[test]
fn diagnostics_surface_info_notes_progress_and_fatal() {
  fresh_state();
  load_script(
    r#"
    Info("test", "obj", "an info message");
    NoteSTDERR("a stderr note");
    NoteLog("a log note");
    ProgressSpinup("stage");
    ProgressStep("working");
    ProgressSpindown("stage");
    "#,
  )
  .expect("non-fatal diagnostics must load and run cleanly");
  let fatal = load_script(r#"Fatal("internal", "obj", "boom");"#);
  assert!(fatal.is_err(), "Fatal must abort the script, got {fatal:?}");
  // Clear the run tally so the raised fatal doesn't leak into sibling tests.
  latexml_core::common::error::initialize_report();
  latexml_core::reset_thread_engine();
}

/// #317: `RequireResource($resource, %options)` — the option-map form carries
/// `type` (mime), `media`, and `content`. Perl's option name `type` maps to the
/// `Resource`'s `mimetype`. The single-arg form still infers a missing mime from
/// the extension.
#[test]
fn require_resource_option_map_sets_type_media_content() {
  fresh_state();
  latexml_core::state::reset_pending_resources();
  load_script(
    r#"RequireResource("custom.css", #{ type: "text/css", media: "print", content: "body{}" });"#,
  )
  .expect("RequireResource with an option map must load cleanly");
  let pending = latexml_core::state::take_pending_resources();
  assert_eq!(pending.len(), 1, "one resource pushed");
  assert_eq!(pending[0].name, "custom.css");
  assert_eq!(
    pending[0].mimetype, "text/css",
    "Perl `type` maps to mimetype"
  );
  assert_eq!(pending[0].media, "print");
  assert_eq!(pending[0].content, "body{}");

  // The single-arg form still infers the mime from the extension.
  load_script(r#"RequireResource("plain.js");"#).expect("single-arg RequireResource");
  let inferred = latexml_core::state::take_pending_resources();
  assert_eq!(
    inferred[0].mimetype, "text/javascript",
    "js extension infers its mime"
  );
  latexml_core::reset_thread_engine();
}

#[test]
fn m1_errors_are_clean() {
  fresh_state();
  assert!(load_script("DefMacro(\"\\\\x{}\", |a| a +").is_err());

  fresh_state();
  load_script(r#"DefMacro("\\boom{}", |x| { throw "kaboom"; });"#).expect("load");
  let r = gullet::do_expand(mouth::tokenize_internal(r"\boom{x}"));
  assert!(r.is_err(), "throwing body should error, got {r:?}");
  latexml_core::reset_thread_engine();
}

/// The SHIPPED example binding must always load cleanly — pins
/// `docs/examples/sample.sty.rhai` against surface drift.
#[test]
fn shipped_example_loads() {
  fresh_state();
  let src = std::fs::read_to_string(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../docs/examples/sample.sty.rhai"
  ))
  .expect("read shipped example");
  let n = load_script(&src).expect("shipped example must load");
  assert!(
    n >= 15,
    "expected the full surface tour to install (got {n})"
  );
  latexml_core::reset_thread_engine();
}
