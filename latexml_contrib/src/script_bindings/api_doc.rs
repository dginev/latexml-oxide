//! Generator for [`API.md`](../../../script_bindings/API.md) — the rendered
//! reference of every function a `.rhai` binding can call.
//!
//! The list is not hand-maintained: it is read back OUT of the live engine via
//! Rhai's reflections API (`Engine::gen_fn_signatures`, the mechanism the Rhai
//! book points at for exactly this), so it cannot drift from what is actually
//! registered. `api_reference_is_up_to_date` in `tests.rs` regenerates and
//! compares on every test run; adding a `register_fn` without regenerating fails
//! CI.
//!
//! Test-only. `gen_fn_signatures` needs Rhai's `metadata` feature, which pulls in
//! serde + serde_json — that must never reach the shipped binary, so `metadata`
//! is enabled on a **dev-dependency** only. Under `resolver = "2"` a
//! dev-dependency's features are not unified into a normal build, so
//! `cargo build --release` stays serde-free.

use rhai::Engine;

/// The receiver groups, in presentation order: free functions first (that is
/// what a binding file calls at top level), then the proxies a body reaches
/// through. `None` is the free-function group.
const GROUPS: &[(Option<&str>, &str, &str)] = &[
  (
    None,
    "Global functions",
    "Called at the top level of a binding file, or from inside any body.",
  ),
  (
    Some("Document"),
    "`document` methods",
    "Reached through the `document` handle a constructor body receives as its \
     first argument — Perl's `$document->method`.",
  ),
  (
    Some("Node"),
    "`Node` methods",
    "An XML node: one returned by `ParseXML`, or one handed to a rewrite / \
     ligature-matcher body.",
  ),
  (
    Some("Whatsit"),
    "`whatsit` methods",
    "The whatsit a hook receives positionally, as Perl passes `$whatsit` (Whatsit.pm).",
  ),
  (
    Some("Definition"),
    "`LookupDefinition()` methods",
    "An already-installed definition, for pushing hooks onto it.",
  ),
  (
    Some("Gullet"),
    "`Gullet` methods",
    "Perl's `$gullet->read…`: the gullet handle a macro body or a parameter-type reader receives first.",
  ),
  (
    Some("Stomach"),
    "`Stomach` methods",
    "Perl's `$stomach`: the handle a primitive body receives first (Primitive.pm:78).",
  ),
  (
    Some("Font"),
    "`Font` methods",
    "Perl's Common/Font.pm getters, on the font a `DefLigature` `fontTest` closure receives.",
  ),
  (
    Some("Number"),
    "`Number` methods",
    "A Number value handle (Common/Number.pm); its shared arithmetic is under Global functions.",
  ),
  (
    Some("Dimension"),
    "`Dimension` methods",
    "A Dimension value handle (Common/Dimension.pm); its shared arithmetic is under Global functions.",
  ),
  (
    Some("Glue"),
    "`Glue` methods",
    "A Glue value handle (Common/Glue.pm); its shared arithmetic is under Global functions.",
  ),
  (
    Some("MuGlue"),
    "`MuGlue` methods",
    "A MuGlue value handle (Core/MuGlue.pm); its shared arithmetic is under Global functions.",
  ),
  (
    Some("Command"),
    "`Command` methods",
    "The `std::process::Command` mirror, for a binding that shells out.",
  ),
];

/// Functions whose registration returns `Dynamic` but whose ACTUAL result is a
/// known handle. Rhai can only report `Dynamic`, which renders as `?` and tells a
/// binding author nothing — and these are exactly the node-walking and
/// node-returning calls that are hardest to infer. `Node?` reads "a Node, or `()`
/// if there is none".
///
/// The calls whose `Dynamic` parameters are token arguments (`token_arg`: a CS name or a `Token`),
/// rendered `string|Token` rather than `?`.
const TOKEN_ARG_FNS: &[&str] = &[
  "AddToMacro",
  "AssignMeaning",
  "DigestIf",
  "IsDefined",
  "Let",
  "LookupDefinition",
  "LookupMeaning",
  "LookupRegister",
  "LookupRegisterOrDefault",
  "SetCondition",
  "XEquals",
];

/// Hand-written, but not drift-prone in the direction that matters:
/// [`overrides_all_exist`] fails if a name here is no longer registered, and any
/// function NOT listed keeps its honest `?`.
const RETURN_OVERRIDES: &[(&str, &str)] = &[
  ("LookupDefinition", "Definition?"),
  ("children", "array"),
  ("findnode", "Node?"),
  ("floatToElement", "Node?"),
  ("firstChild", "Node?"),
  ("getElement", "Node?"),
  ("getNode", "Node"),
  ("insertElement", "Node"),
  ("nextSibling", "Node?"),
  ("closeElement", "Node?"),
  ("maybeCloseElement", "Node?"),
  ("openElement", "Node"),
  ("openElementAt", "Node"),
  ("parent", "Node?"),
  ("prevSibling", "Node?"),
  ("renameNode", "Node"),
  ("wrapNodes", "Node?"),
];

/// Every [`RETURN_OVERRIDES`] key must still be a registered function — so a
/// renamed or deleted call cannot leave a lie behind in the reference.
pub(super) fn overrides_all_exist(engine: &Engine) -> Vec<&'static str> {
  let registered: Vec<String> = engine
    .gen_fn_signatures(false)
    .iter()
    .map(|s| s.split_once('(').map_or(s.clone(), |(n, _)| n.to_string()))
    .collect();
  RETURN_OVERRIDES
    .iter()
    .map(|(name, _)| *name)
    .filter(|name| !registered.iter().any(|r| r == name))
    .collect()
}

/// Where the in-depth documentation for one `.rhai` call lives.
enum Doc {
  /// A short explanation, plus the documented Rust item that implements the
  /// call: `Rust(summary, path)`.
  ///
  /// BOTH, deliberately. The link is what keeps the reference from rotting —
  /// it goes to the real semantics, and being a rustdoc INTRA-DOC link it is
  /// validated by rustdoc itself, so a renamed target is a `cargo doc` warning
  /// naming the line in `API.md` rather than a dead anchor nobody notices. But
  /// a link alone makes the reader click 145 times to find out what anything
  /// does, so every row also says it in one line. The summary is what the call
  /// means TO A BINDING AUTHOR; the link carries the exact semantics.
  Rust(&'static str, &'static str),
  /// No single Rust item to point at — a Rhai-only helper, or a shim whose
  /// whole behaviour lives in the registration. Written out here, because "no
  /// counterpart" must not mean "no documentation". Rendered as Markdown, so a
  /// note may still LINK the parts that do have a Rust home (`parent` below
  /// describes its guard and links the traversal it wraps); rustdoc validates
  /// those links exactly as it does a [`Doc::Rust`] one.
  Note(&'static str),
}

/// A call's counterparts, for the generated three-column cheat sheet ([`generate_cheatsheet`]): Perl's spelling
/// (Package.pm, or the Perl method or expression), then the compile-time macro that lowers to the same core call;
/// `""` where there is none.
struct Mirror(&'static str, &'static str);

/// Perl calls a binding author meets (Package.pm's exports and the value constructors) that the dialect does not
/// register yet, with their macro: the cheat sheet's empty Rhai cells. [`stale_gaps`] fails when one gets registered.
const PERL_GAPS: &[(&str, &str)] = &[
  ("AfterAssignment", ""),
  ("Box", ""),
  ("CheckOptions", ""),
  ("CleanBibKey", ""),
  ("CleanClassName", ""),
  ("CleanIndexKey", ""),
  ("CleanURL", ""),
  ("Color", ""),
  ("ComposeURL", ""),
  ("Debug", "Debug!"),
  ("DefAutoload", ""),
  ("DefColor", ""),
  ("DefColorModel", ""),
  ("DefConditionalI", ""),
  ("DefConstructorI", ""),
  ("DefEnvironmentI", ""),
  ("DefExpandable", ""),
  ("DefMacroI", ""),
  ("DefMathI", ""),
  ("DefPrimitiveI", ""),
  ("DefRegisterI", ""),
  ("Deprecated", ""),
  ("DigestLiteral", ""),
  ("DisableKeyVal", ""),
  ("DocType", ""),
  ("Float", ""),
  ("FontDecode", ""),
  ("FontDecodeString", ""),
  ("GenerateID", ""),
  ("HasKeyVal", ""),
  ("IfCondition", ""),
  ("InputContent", ""),
  ("InstallDefinition", "InstallDefinition!"),
  ("List", ""),
  ("LoadFontMap", ""),
  ("LoadFormat", ""),
  ("LookupColor", "LookupColor!"),
  ("MaybeNoteLabel", ""),
  ("MaybePeekLabel", ""),
  ("MuDimension", ""),
  ("NormalizeBibKey", ""),
  ("Pair", ""),
  ("PairList", ""),
  ("ProcessPendingResources", ""),
  ("ReadAlignmentTemplate", ""),
  ("ReadParameters", ""),
  ("Stringify", ""),
  ("TokensI", ""),
  ("TrimmedCommaList", ""),
  ("convertLaTeXArgs", ""),
  ("createXMRefs", ""),
  ("decodeMathChar", ""),
  ("dualize_arglist", ""),
];

/// Every registered call, by name. Overloads share one entry: the signatures
/// differ, what the call MEANS does not. Names registered on more than one
/// handle are the exception — they live in [`HANDLE_DOCS`] instead.
///
/// Three tests keep this honest — [`undocumented_names`] fails when a
/// `register_fn` has no entry (so a new helper cannot ship undocumented),
/// [`stale_names`] fails when an entry names a call that no longer exists, and
/// [`ambiguous_names`] fails when one entry would be shared by two handles.
const DOCS: &[(&str, Mirror, Doc)] = &[
  (
    "!",
    Mirror("!$v", ""),
    Doc::Note(
      "Perl's truthiness, for `!!v` (`if ($v)`) and `!v` (`unless ($v)`): `()`, `\"\"`, `\"0\"`, `0` and `0.0` are \
       false, a bool is itself, every Token, Tokens (even empty), box, whatsit, array and map is true \
       (Perl's objects overload no bool).",
    ),
  ),
  (
    "!=",
    Mirror("$a ne $b", ""),
    Doc::Note(
      "**Deprecated** (Warns once per binding load): a Tokens, Token, box or value handle against a string compares \
       its string form (use `ToString(v) != s`); `()` against a string, either way round, is `\"\"`, as Perl's `undef ne ''` \
       (use `defined(v)` or `!!v`).",
    ),
  ),
  (
    "==",
    Mirror("$a eq $b", ""),
    Doc::Note(
      "**Deprecated** (Warns once per binding load): a Tokens, Token, box or value handle against a string compares \
       its string form (use `ToString(v) == s`); `()` against a string, either way round, is `\"\"`, as Perl's `undef eq ''` \
       (use `defined(v)` or `!v`). Rhai's `switch` does not use this rule: it matches by value and type, so a `\"\" =>` \
       case never matches `()` — test `if defined(v)` or `!v` instead.",
    ),
  ),
  (
    "*",
    Mirror("$v->multiply($n)", ""),
    Doc::Note(
      "A Number, Dimension, Glue or MuGlue times an int or float (Number.pm `multiply`, truncated; Glue scales its stretch and shrink).",
    ),
  ),
  (
    "+",
    Mirror("$v->add($w)", ""),
    Doc::Note(
      "The sum of two value handles of one type (Number.pm `add`); Glue + Glue and MuGlue + MuGlue as TeX's `\\advance` \
       (the higher fill order wins: `MuGlue(\"3mu plus 1mu\") + MuGlue(\"1mu plus 1fil\")` is `4.0mu plus 1.0fil`), \
       Glue + a Number or Dimension adds to the natural size. Perl's Glue.pm and MuGlue.pm differ (OXIDIZED_DESIGN_DIVERGENCES #1030).",
    ),
  ),
  (
    "-",
    Mirror("$v->subtract($w)", ""),
    Doc::Note(
      "The difference of two value handles (Number.pm `subtract`), or the negation of one (`negate`); a Glue as TeX's \
       `\\advance` by the negation: Glue − Dimension keeps the stretch, `Glue(\"1pt plus 1fil\") - Glue(\"1pt plus 2fil\")` \
       is `0.0pt plus -1.0fil` (Perl drops the stretch; OXIDIZED_DESIGN_DIVERGENCES #1030).",
    ),
  ),
  (
    "/",
    Mirror("$v->divide($n)", ""),
    Doc::Note("A value handle divided by an int, truncated (Number.pm `divide`)."),
  ),
  (
    "<",
    Mirror("$v->valueOf < $w->valueOf", ""),
    Doc::Note("Compares two value handles of one type by `valueOf`."),
  ),
  (
    "<=",
    Mirror("$v->valueOf <= $w->valueOf", ""),
    Doc::Note("Compares two value handles of one type by `valueOf`."),
  ),
  (
    ">",
    Mirror("$v->valueOf > $w->valueOf", ""),
    Doc::Note("Compares two value handles of one type by `valueOf`."),
  ),
  (
    ">=",
    Mirror("$v->valueOf >= $w->valueOf", ""),
    Doc::Note("Compares two value handles of one type by `valueOf`."),
  ),
  (
    "AddToCounter",
    Mirror("AddToCounter", "AddToCounter!"),
    Doc::Rust(
      "Add a number to a counter.",
      "latexml_core::binding::counter::dialect::add_to_counter",
    ),
  ),
  (
    "AppendSearchPath",
    Mirror("", ""),
    Doc::Rust(
      "Append a directory to the input search path (searched after existing paths).",
      "latexml_core::state::add_search_path",
    ),
  ),
  (
    "AssignBool",
    Mirror("AssignValue", ""),
    Doc::Rust(
      "Bind a key to a value, as `AssignValue`.",
      "latexml_core::state::assign_value",
    ),
  ),
  (
    "AssignCatcode",
    Mirror("AssignCatcode", "AssignCatcode!"),
    Doc::Rust(
      "Set a character's category code, with an optional scope.",
      "latexml_core::state::assign_catcode",
    ),
  ),
  (
    "AssignDelcode",
    Mirror("AssignDelcode", ""),
    Doc::Rust(
      "Set a character's delimiter code, with an optional scope.",
      "latexml_core::state::assign_delcode",
    ),
  ),
  (
    "AssignFloat",
    Mirror("AssignValue", ""),
    Doc::Rust(
      "Bind a key to a fractional (float) value, with a scope.",
      "latexml_core::state::assign_value",
    ),
  ),
  (
    "AssignLCcode",
    Mirror("AssignLCcode", ""),
    Doc::Rust(
      "Set a character's lowercase code, with an optional scope.",
      "latexml_core::state::assign_lccode",
    ),
  ),
  (
    "AssignMapping",
    Mirror("AssignMapping", "AssignMapping!"),
    Doc::Rust(
      "Bind one key inside a named mapping to any value, as `AssignValue`; `()` removes the key.",
      "latexml_core::state::assign_mapping",
    ),
  ),
  (
    "AssignMathcode",
    Mirror("AssignMathcode", ""),
    Doc::Rust(
      "Set a character's math code, with an optional scope.",
      "latexml_core::state::assign_mathcode",
    ),
  ),
  (
    "AssignMeaning",
    Mirror("Let", "AssignMeaning!"),
    Doc::Rust(
      "Give a token (CS name or `Token`) the current meaning of another, as `Let`.",
      "latexml_core::state::let_definition",
    ),
  ),
  (
    "AssignNumber",
    Mirror("AssignValue", ""),
    Doc::Rust(
      "Bind a key to a value, as `AssignValue`.",
      "latexml_core::state::assign_value",
    ),
  ),
  (
    "AssignRegister",
    Mirror("AssignRegister", "AssignRegister!"),
    Doc::Rust(
      "Perl's `AssignRegister(cs, value[, scope[, params]])`: a typed value, or a string read for the register's type; a Warning when `cs` is no register.",
      "latexml_core::state::assign_register_token",
    ),
  ),
  (
    "AssignSFcode",
    Mirror("AssignSFcode", ""),
    Doc::Rust(
      "Set a character's space-factor code, with an optional scope.",
      "latexml_core::state::assign_sfcode",
    ),
  ),
  (
    "AssignString",
    Mirror("AssignValue", ""),
    Doc::Rust(
      "Bind a key to a value, as `AssignValue`.",
      "latexml_core::state::assign_value",
    ),
  ),
  (
    "AssignUCcode",
    Mirror("AssignUCcode", ""),
    Doc::Rust(
      "Set a character's uppercase code, with an optional scope.",
      "latexml_core::state::assign_uccode",
    ),
  ),
  (
    "AssignValue",
    Mirror("AssignValue", "AssignValue!"),
    Doc::Rust(
      "Bind a key in the value table to any value (string, number, bool, `Token`, `Tokens`, handle), with a scope; `()` stores undef.",
      "latexml_core::state::assign_value",
    ),
  ),
  (
    "AtBeginDocument",
    Mirror("AtBeginDocument", ""),
    Doc::Rust(
      "Queue TeX source, a `Tokens`/`Token` value, or a closure (installed as a macro) to run at `\\begin{document}`.",
      "latexml_core::state::push_value",
    ),
  ),
  (
    "AtEndDocument",
    Mirror("AtEndDocument", ""),
    Doc::Rust(
      "Queue TeX source, a `Tokens`/`Token` value, or a closure (installed as a macro) to run at `\\end{document}`.",
      "latexml_core::state::push_value",
    ),
  ),
  (
    "CC_ACTIVE",
    Mirror("CC_ACTIVE", ""),
    Doc::Note("The catcode constant `CC_ACTIVE` (13), for `Token(string, catcode)`."),
  ),
  (
    "CC_ALIGN",
    Mirror("CC_ALIGN", ""),
    Doc::Note("The catcode constant `CC_ALIGN` (4), for `Token(string, catcode)`."),
  ),
  (
    "CC_ARG",
    Mirror("CC_ARG", ""),
    Doc::Note("The catcode constant `CC_ARG` (18), for `Token(string, catcode)`."),
  ),
  (
    "CC_BEGIN",
    Mirror("CC_BEGIN", ""),
    Doc::Note("The catcode constant `CC_BEGIN` (1), for `Token(string, catcode)`."),
  ),
  (
    "CC_COMMENT",
    Mirror("CC_COMMENT", ""),
    Doc::Note("The catcode constant `CC_COMMENT` (14), for `Token(string, catcode)`."),
  ),
  (
    "CC_CS",
    Mirror("CC_CS", ""),
    Doc::Note("The catcode constant `CC_CS` (16), for `Token(string, catcode)`."),
  ),
  (
    "CC_END",
    Mirror("CC_END", ""),
    Doc::Note("The catcode constant `CC_END` (2), for `Token(string, catcode)`."),
  ),
  (
    "CC_EOL",
    Mirror("CC_EOL", ""),
    Doc::Note("The catcode constant `CC_EOL` (5), for `Token(string, catcode)`."),
  ),
  (
    "CC_ESCAPE",
    Mirror("CC_ESCAPE", ""),
    Doc::Note("The catcode constant `CC_ESCAPE` (0), for `Token(string, catcode)`."),
  ),
  (
    "CC_IGNORE",
    Mirror("CC_IGNORE", ""),
    Doc::Note("The catcode constant `CC_IGNORE` (9), for `Token(string, catcode)`."),
  ),
  (
    "CC_INVALID",
    Mirror("CC_INVALID", ""),
    Doc::Note("The catcode constant `CC_INVALID` (15), for `Token(string, catcode)`."),
  ),
  (
    "CC_LETTER",
    Mirror("CC_LETTER", ""),
    Doc::Note("The catcode constant `CC_LETTER` (11), for `Token(string, catcode)`."),
  ),
  (
    "CC_MARKER",
    Mirror("CC_MARKER", ""),
    Doc::Note("The catcode constant `CC_MARKER` (17), for `Token(string, catcode)`."),
  ),
  (
    "CC_MATH",
    Mirror("CC_MATH", ""),
    Doc::Note("The catcode constant `CC_MATH` (3), for `Token(string, catcode)`."),
  ),
  (
    "CC_OTHER",
    Mirror("CC_OTHER", ""),
    Doc::Note("The catcode constant `CC_OTHER` (12), for `Token(string, catcode)`."),
  ),
  (
    "CC_PARAM",
    Mirror("CC_PARAM", ""),
    Doc::Note("The catcode constant `CC_PARAM` (6), for `Token(string, catcode)`."),
  ),
  (
    "CC_SPACE",
    Mirror("CC_SPACE", ""),
    Doc::Note("The catcode constant `CC_SPACE` (10), for `Token(string, catcode)`."),
  ),
  (
    "CC_SUB",
    Mirror("CC_SUB", ""),
    Doc::Note("The catcode constant `CC_SUB` (8), for `Token(string, catcode)`."),
  ),
  (
    "CC_SUPER",
    Mirror("CC_SUPER", ""),
    Doc::Note("The catcode constant `CC_SUPER` (7), for `Token(string, catcode)`."),
  ),
  (
    "Command",
    Mirror("", ""),
    Doc::Note(
      "Start a `std::process::Command` builder. A Rhai-only shim; nothing runs until `output()`.",
    ),
  ),
  (
    "CounterValue",
    Mirror("CounterValue", "CounterValue!"),
    Doc::Rust(
      "The current value of a counter.",
      "latexml_core::binding::counter::dialect::counter_value",
    ),
  ),
  (
    "DeclareOption",
    Mirror("DeclareOption", "DeclareOption!"),
    Doc::Note(
      "Declare a class/package option: named with a body, or the bare-closure form for the default handler (Perl `DeclareOption(undef, sub {...})`).",
    ),
  ),
  (
    "DefAccent",
    Mirror("DefAccent", "DefAccent!"),
    Doc::Note(
      "Define an accent command in both its combining and standalone forms; Perl's options map `#{below: true}`.",
    ),
  ),
  (
    "DefColumnType",
    Mirror("DefColumnType", "DefColumnType!"),
    Doc::Rust(
      "Define a tabular column type, as its rewrite expansion.",
      "latexml_core::binding::def::dialect::def_macro",
    ),
  ),
  (
    "DefParameterType",
    Mirror("DefParameterType", "DefParameterType!"),
    Doc::Rust(
      "Define a parameter type: the reader `|gullet, extra…|` returns its value (Tokens, a Token, `()` for none), \
       with Perl's options `optional`, `novalue`, `semiverbatim`, `undigested`, `reversion`, `beforeDigest`, \
       `afterDigest`, and `nargs` accepted unread, as Perl's.",
      "latexml_core::parameter::Parameter",
    ),
  ),
  (
    "DefConditional",
    Mirror("DefConditional", "DefConditional!"),
    Doc::Rust(
      "Define a conditional control sequence.",
      "latexml_core::binding::def::dialect::def_conditional",
    ),
  ),
  (
    "DefConstructor",
    Mirror("DefConstructor", "DefConstructor!"),
    Doc::Rust(
      "Define a control sequence that constructs XML.",
      "latexml_core::binding::def::dialect::def_constructor",
    ),
  ),
  (
    "DefEnvironment",
    Mirror("DefEnvironment", "DefEnvironment!"),
    Doc::Rust(
      "Define an environment that constructs XML.",
      "latexml_core::binding::def::dialect::def_environment",
    ),
  ),
  (
    "DefKeyVal",
    Mirror("DefKeyVal", "DefKeyVal!"),
    Doc::Rust(
      "Declare one key of a keyval family.",
      "latexml_core::keyval::define",
    ),
  ),
  (
    "DefLigature",
    Mirror("DefLigature", "DefLigature!"),
    Doc::Note(
      "Register a text ligature pattern; `#{fontTest: |font| …}` limits it to the fonts the closure accepts.",
    ),
  ),
  (
    "DefMacro",
    Mirror("DefMacro", "DefMacro!"),
    Doc::Rust(
      "Define a macro's expansion; a closure body is Perl's `|gullet, args|`, each argument Tokens (or a `KeyVals`). \
       A Perl-form closure declares its handle and every argument, `|gullet, x|`; fewer parameters is the old \
       form, with a Warning naming the fix.",
      "latexml_core::binding::def::dialect::def_macro",
    ),
  ),
  (
    "DefMath",
    Mirror("DefMath", "DefMath!"),
    Doc::Rust(
      "Define a mathematical symbol or function.",
      "latexml_core::binding::def::dialect::def_math",
    ),
  ),
  (
    "DefMathLigature",
    Mirror("DefMathLigature", "DefMathLigature!"),
    Doc::Note("Register a math ligature, as a pattern or a matcher closure."),
  ),
  (
    "DefMathRewrite",
    Mirror("DefMathRewrite", "DefMathRewrite!"),
    Doc::Note("As `DefRewrite`, scoped to math."),
  ),
  (
    "DefPrimitive",
    Mirror("DefPrimitive", "DefPrimitive!"),
    Doc::Rust(
      "Define a primitive: it runs at digestion, after expansion; a closure body is Perl's `|stomach, args|`. A \
       Perl-form closure declares its handle and every argument, `|stomach, x|`; fewer parameters is the old \
       form, with a Warning naming the fix.",
      "latexml_core::binding::def::dialect::def_primitive",
    ),
  ),
  (
    "DefRegister",
    Mirror("DefRegister", "DefRegister!"),
    Doc::Rust(
      "Perl's `DefRegister(cs, value[, #{readonly, getter, setter, address, allocate}])`: the value's type (Number, \
       Dimension, Glue, MuGlue, Tokens) is the register's; a string is read for the type `cs` already has, else by \
       its form (a `mu` unit MuGlue, `plus`/`minus` Glue, another unit Dimension; no unit is an Error and 0). `getter` is `|params…|`, `setter` \
       `|value, scope, params…|`.",
      "latexml_core::binding::def::dialect::def_register",
    ),
  ),
  (
    "DefRewrite",
    Mirror("DefRewrite", "DefRewrite!"),
    Doc::Note("Register a document rewrite rule (data form, or a replace closure)."),
  ),
  (
    "Dimension",
    Mirror("Dimension", "Dimension!"),
    Doc::Note(
      "Perl's `Dimension(spec)`: `\"3pt\"` (a number and a unit), or an int of sp; a spec that is not a number and a unit is an Error and 0 (Perl reads `\"3 furlongs\"` as 3sp, \
       OXIDIZED_DESIGN_DIVERGENCES #1031).",
    ),
  ),
  (
    "Glue",
    Mirror("Glue", "Glue!"),
    Doc::Note(
      "Perl's `Glue(spec)`: `\"1pt plus 2fil minus 1pt\"`, an int of sp, or `Glue(sp, plus, pfill, minus, mfill)` with fill orders as names or 0-3; an unreadable spec is an Error and 0.",
    ),
  ),
  (
    "LookupDimension",
    Mirror("LookupDimension", ""),
    Doc::Rust(
      "Perl's `LookupDimension(x[, noerror])`: a spec or a register's name as a Dimension; `Dimension(0)` with a Warning otherwise.",
      "latexml_core::state::lookup_dimension_cs",
    ),
  ),
  (
    "LookupRegister",
    Mirror("LookupRegister", "LookupRegister!"),
    Doc::Rust(
      "Perl's `LookupRegister(cs[, params])`: the register's typed value (Number, Dimension, Glue, MuGlue, Tokens); `()` with a Warning when `cs` is no register, undefined included.",
      "latexml_core::state::lookup_register_definition",
    ),
  ),
  (
    "MuGlue",
    Mirror("MuGlue", "MuGlue!"),
    Doc::Note(
      "Perl's `MuGlue(spec)`: `\"3mu plus 1mu\"`, an int of the mu value in sp, or the five-argument form; an unreadable spec is an Error and 0.",
    ),
  ),
  (
    "Number",
    Mirror("Number", "Number!"),
    Doc::Note(
      "Perl's `Number(n)`: an int, a float (truncated) or a numeric string; any other string is an Error and 0.",
    ),
  ),
  (
    "absolute",
    Mirror("$v->absolute", ""),
    Doc::Note("A value handle's absolute value (Number.pm)."),
  ),
  (
    "add",
    Mirror("$v->add", ""),
    Doc::Note("Perl's `$a->add($b)`: as `+`."),
  ),
  (
    "defined",
    Mirror("defined $v", ""),
    Doc::Note(
      "Perl's `defined($v)`: true for every value but `()` (`\"\"`, `0` and empty Tokens are defined); `v == ()` \
       is not the test for a string, which compares equal to `()` when empty.",
    ),
  ),
  (
    "Digest",
    Mirror("Digest", "Digest!"),
    Doc::Rust(
      "Digest TeX source (a string, TokenizeInternal'd), a `Token` or `Tokens`, returning the digested handle.",
      "latexml_core::stomach::digest",
    ),
  ),
  (
    "DigestText",
    Mirror("DigestText", "DigestText!"),
    Doc::Rust(
      "Digest TeX source, a `Token` or `Tokens` in restricted horizontal mode, returning the digested handle.",
      "latexml_core::binding::content::digest_text",
    ),
  ),
  (
    "EndSemiverbatim",
    Mirror("EndSemiverbatim", ""),
    Doc::Rust(
      "Close a `StartSemiverbatim` region, restoring the saved catcodes.",
      "latexml_core::state::end_semiverbatim",
    ),
  ),
  (
    "Error",
    Mirror("Error", "Error!"),
    Doc::Note(
      "Log an `Error:`, with Perl's trailing details (a string or an array). Past `MAX_ERRORS` this escalates to `Fatal` and ends the conversion.",
    ),
  ),
  (
    "ExecuteOptions",
    Mirror("ExecuteOptions", ""),
    Doc::Rust(
      "Run the handlers for a list of class/package options.",
      "latexml_core::binding::content::execute_options",
    ),
  ),
  (
    "Expand",
    Mirror("Expand", "Expand!"),
    Doc::Rust(
      "Fully expand tokens (a `Tokens`, or one `Token`), without digesting them.",
      "latexml_core::gullet::do_expand",
    ),
  ),
  (
    "ExpandPartially",
    Mirror("ExpandPartially", ""),
    Doc::Rust(
      "Expand tokens only up to the first unexpandable one.",
      "latexml_core::gullet::do_expand_partially",
    ),
  ),
  (
    "Fatal",
    Mirror("Fatal", "Fatal!"),
    Doc::Note("End the conversion with `Fatal:<cat>:<obj>`, plus optional details."),
  ),
  (
    "GetKeyVal",
    Mirror("GetKeyVal", "GetKeyVal!"),
    Doc::Note(
      "One value of a keyval set, as Perl's: Tokens as read, a box once digested, `()` when missing; a repeated \
       key's last value. Test it as Perl's `if ($v)` with `!!v`, or `defined(v)`.",
    ),
  ),
  (
    "GetKeyVals",
    Mirror("GetKeyVals", "GetKeyVals!"),
    Doc::Note(
      "A whole keyval set as a map of `GetKeyVal`'s values, a repeated key's as an array. Accepts a `KeyVals` \
       argument (read, or digested in a constructor) or its `k=v,…` source string.",
    ),
  ),
  (
    "Info",
    Mirror("Info", "Info!"),
    Doc::Note("Log an `Info:` line, plus optional details."),
  ),
  (
    "InputDefinitions",
    Mirror("InputDefinitions", "InputDefinitions!"),
    Doc::Rust(
      "Find and load the definitions for a package or class.",
      "latexml_core::binding::content::input_definitions",
    ),
  ),
  (
    "IsDefined",
    Mirror("IsDefined", "IsDefined!"),
    Doc::Rust(
      "Whether a token (CS name or `Token`) is defined, and not `\\let` to `\\relax`.",
      "latexml_core::binding::def::dialect::is_defined_token",
    ),
  ),
  (
    "LaTeXMLVersion",
    Mirror("$LaTeXML::VERSION", ""),
    Doc::Note(
      "The engine version string — the `LATEXML_VERSION` value ([`lookup_string`](latexml_core::state::lookup_string)).",
    ),
  ),
  (
    "Let",
    Mirror("Let", "Let!"),
    Doc::Rust(
      "TeX's `\\let`: copy one token's meaning onto another, with an optional scope (`\"global\"` is `\\global\\let`). Each token is a CS name string or a `Token`.",
      "latexml_core::state::let_definition",
    ),
  ),
  (
    "LoadClass",
    Mirror("LoadClass", "LoadClass!"),
    Doc::Rust(
      "Load a document class, falling back to a prefix match then OmniBus.",
      "latexml_core::binding::content::load_class",
    ),
  ),
  (
    "LookupBool",
    Mirror("LookupValue", "LookupBool!"),
    Doc::Rust(
      "Read a value as a boolean.",
      "latexml_core::state::lookup_bool",
    ),
  ),
  (
    "LookupCatcode",
    Mirror("LookupCatcode", "LookupCatcode!"),
    Doc::Rust(
      "The category code currently in force for a character; `()` when the table holds none.",
      "latexml_core::state::lookup_catcode",
    ),
  ),
  (
    "LookupDefinition",
    Mirror("LookupDefinition", "LookupDefinition!"),
    Doc::Note(
      "Fetch the installed definition of a CS name or `Token` so hooks can be pushed onto it; `()` when undefined or `\\let` to a character.",
    ),
  ),
  (
    "LookupDelcode",
    Mirror("LookupDelcode", ""),
    Doc::Rust(
      "A character's delimiter code; `()` when the table holds none.",
      "latexml_core::state::lookup_delcode",
    ),
  ),
  (
    "LookupFloat",
    Mirror("LookupValue", ""),
    Doc::Rust(
      "Read a value as a fractional (float) number, keeping the fraction `LookupNumber` would truncate.",
      "latexml_core::state::lookup_float",
    ),
  ),
  (
    "LookupLCcode",
    Mirror("LookupLCcode", ""),
    Doc::Rust(
      "A character's lowercase code; `()` when the table holds none.",
      "latexml_core::state::lookup_lccode",
    ),
  ),
  (
    "LookupMapping",
    Mirror("LookupMapping", ""),
    Doc::Rust(
      "Read one key out of a named mapping, with the type it was stored as; `()` when absent.",
      "latexml_core::state::lookup_mapping",
    ),
  ),
  (
    "LookupMappingKeys",
    Mirror("LookupMappingKeys", ""),
    Doc::Rust(
      "The keys of a named mapping, sorted.",
      "latexml_core::state::with_mapping_keys",
    ),
  ),
  (
    "LookupMathcode",
    Mirror("LookupMathcode", ""),
    Doc::Rust(
      "A character's math code; `()` when the table holds none.",
      "latexml_core::state::lookup_mathcode",
    ),
  ),
  (
    "LookupMeaning",
    Mirror("LookupMeaning", "LookupMeaning!"),
    Doc::Rust(
      "What a token means right now: its definition, or itself.",
      "latexml_core::state::lookup_meaning",
    ),
  ),
  (
    "LookupNumber",
    Mirror("LookupValue", "LookupNumber!"),
    Doc::Rust(
      "Read a value as a number.",
      "latexml_core::state::lookup_number",
    ),
  ),
  (
    "LookupSFcode",
    Mirror("LookupSFcode", ""),
    Doc::Rust(
      "A character's space-factor code; `()` when the table holds none.",
      "latexml_core::state::lookup_sfcode",
    ),
  ),
  (
    "LookupString",
    Mirror("LookupValue", "LookupString!"),
    Doc::Rust(
      "Read a value as a string; empty when unset.",
      "latexml_core::state::lookup_string",
    ),
  ),
  (
    "LookupTokens",
    Mirror("LookupValue", "LookupTokens!"),
    Doc::Rust(
      "Read a value as tokens.",
      "latexml_core::state::lookup_tokens",
    ),
  ),
  (
    "LookupUCcode",
    Mirror("LookupUCcode", ""),
    Doc::Rust(
      "A character's uppercase code; `()` when the table holds none.",
      "latexml_core::state::lookup_uccode",
    ),
  ),
  (
    "LookupValue",
    Mirror("LookupValue", ""),
    Doc::Rust(
      "Read a value with the type it was stored as (a list as an array); `()` when unset.",
      "latexml_core::state::lookup_value",
    ),
  ),
  (
    "MergeFont",
    Mirror("MergeFont", "MergeFont!"),
    Doc::Rust(
      "Merge font attributes into the current font, group-locally.",
      "latexml_core::binding::content::merge_font",
    ),
  ),
  (
    "NewCounter",
    Mirror("NewCounter", "NewCounter!"),
    Doc::Rust(
      "Declare a new counter.",
      "latexml_core::binding::counter::dialect::new_counter",
    ),
  ),
  (
    "Note",
    Mirror("Note", "Note!"),
    Doc::Note("Write a progress note to both the conversion log and stderr."),
  ),
  (
    "NoteLog",
    Mirror("NoteLog", "NoteLog!"),
    Doc::Note("Write a progress note to the conversion log only."),
  ),
  (
    "NoteSTDERR",
    Mirror("NoteSTDERR", "NoteSTDERR!"),
    Doc::Note("Write a progress note to stderr only."),
  ),
  (
    "ParseXML",
    Mirror("", ""),
    Doc::Rust(
      "Parse a markup chunk into nodes; a bare fragment is fine.",
      "latexml_core::common::xml::parse_fragment",
    ),
  ),
  (
    "PassOptions",
    Mirror("PassOptions", ""),
    Doc::Rust(
      "Forward options to a package or class not yet loaded.",
      "latexml_core::binding::content::pass_options",
    ),
  ),
  (
    "PopValue",
    Mirror("PopValue", ""),
    Doc::Rust(
      "Pop and return the last value of a value-table list, type preserved (`()` if empty).",
      "latexml_core::state::pop_value",
    ),
  ),
  (
    "PrependSearchPath",
    Mirror("", ""),
    Doc::Rust(
      "Prepend a directory to the input search path (searched first).",
      "latexml_core::state::search_paths_push_front",
    ),
  ),
  (
    "ProcessOptions",
    Mirror("ProcessOptions", "ProcessOptions!"),
    Doc::Rust(
      "Execute the declared options — in declaration order, or (`*`) in the order passed.",
      "latexml_core::binding::content::process_options",
    ),
  ),
  (
    "ProgressSpindown",
    Mirror("ProgressSpindown", ""),
    Doc::Rust(
      "Close a named progress stage in the log.",
      "latexml_core::common::error::note_end",
    ),
  ),
  (
    "ProgressSpinup",
    Mirror("ProgressSpinup", ""),
    Doc::Rust(
      "Open a named progress stage in the log.",
      "latexml_core::common::error::note_begin",
    ),
  ),
  (
    "ProgressStep",
    Mirror("ProgressStep", ""),
    Doc::Rust(
      "Advance the progress indicator; a no-op in this port.",
      "latexml_core::common::error::progress_step",
    ),
  ),
  (
    "PushValue",
    Mirror("PushValue", ""),
    Doc::Rust(
      "Push a value (string/int/float/bool/Tokens) onto a value-table list, creating it (global) if absent.",
      "latexml_core::state::push_value",
    ),
  ),
  (
    "RawTeX",
    Mirror("RawTeX", "RawTeX!"),
    Doc::Rust(
      "Process a chunk of literal TeX as definitions.",
      "latexml_core::stomach::raw_tex",
    ),
  ),
  (
    "ReadArg",
    Mirror("$gullet->readArg", ""),
    Doc::Rust(
      "Read one TeX argument, a token or a braced group, as `Tokens`.",
      "latexml_core::gullet::read_arg",
    ),
  ),
  (
    "ReadOptional",
    Mirror("$gullet->readOptional", ""),
    Doc::Rust(
      "Read a LaTeX optional `[…]` argument as `Tokens`; `()` when there is none.",
      "latexml_core::gullet::read_optional",
    ),
  ),
  (
    "ReadToken",
    Mirror("$gullet->readToken", ""),
    Doc::Rust(
      "Read the next token, unexpanded; `()` at the end of input.",
      "latexml_core::gullet::read_token",
    ),
  ),
  (
    "ReadUntil",
    Mirror("$gullet->readUntil", ""),
    Doc::Rust(
      "Read a balanced token sequence up to a delimiter (TeX source, `Token` or `Tokens`), as `Tokens`.",
      "latexml_core::gullet::read_until",
    ),
  ),
  (
    "RefCurrentID",
    Mirror("RefCurrentID", "RefCurrentID!"),
    Doc::Rust(
      "Reuse the last id without stepping, when its box was pruned.",
      "latexml_core::binding::counter::dialect::ref_current_id",
    ),
  ),
  (
    "RefStepCounter",
    Mirror("RefStepCounter", "RefStepCounter!"),
    Doc::Rust(
      "Step a counter through `\\refstepcounter`'s meaning (`noreset`: directly) and return its `refnum` and `id`.",
      "latexml_core::binding::counter::dialect::ref_step_counter_by_meaning",
    ),
  ),
  (
    "RefStepID",
    Mirror("RefStepID", "RefStepID!"),
    Doc::Rust(
      "Step only the uncounter, for an UN-numbered unit; returns the id.",
      "latexml_core::binding::counter::dialect::ref_step_id",
    ),
  ),
  (
    "RegisterDocumentNamespace",
    Mirror("RegisterDocumentNamespace", "RegisterDocumentNamespace!"),
    Doc::Rust(
      "Bind an OUTPUT-document prefix to a namespace URI.",
      "latexml_core::common::model::register_document_namespace",
    ),
  ),
  (
    "RegisterNamespace",
    Mirror("RegisterNamespace", "RegisterNamespace!"),
    Doc::Rust(
      "Bind a CODE prefix to a namespace URI.",
      "latexml_core::common::model::register_namespace",
    ),
  ),
  (
    "RelaxNGSchema",
    Mirror("RelaxNGSchema", "RelaxNGSchema!"),
    Doc::Rust(
      "Select the RelaxNG schema defining the output language.",
      "latexml_core::binding::content::select_relaxng_schema",
    ),
  ),
  (
    "RequirePackage",
    Mirror("RequirePackage", "RequirePackage!"),
    Doc::Rust(
      "Load a package (options: Perl's `options`, `type`, `notex`, `after`, …): its binding, or the raw `.sty` when `INCLUDE_STYLES` allows.",
      "latexml_core::binding::content::require_package",
    ),
  ),
  (
    "RequireResource",
    Mirror("RequireResource", "RequireResource!"),
    Doc::Rust(
      "Attach a CSS or JavaScript resource to the document.",
      "latexml_core::binding::content::require_resource",
    ),
  ),
  (
    "ResetCounter",
    Mirror("ResetCounter", "ResetCounter!"),
    Doc::Rust(
      "Reset a counter to zero.",
      "latexml_core::binding::counter::dialect::reset_counter",
    ),
  ),
  (
    "Revert",
    Mirror("Revert", "Revert!"),
    Doc::Rust(
      "A digested value back to the source tokens that made it.",
      "latexml_core::digested::Digested::revert",
    ),
  ),
  (
    "SetCondition",
    Mirror("SetCondition", ""),
    Doc::Rust(
      "Set a test-less conditional (`\\newif` style) to true or false.",
      "latexml_core::binding::content::set_condition",
    ),
  ),
  (
    "ShiftValue",
    Mirror("ShiftValue", ""),
    Doc::Rust(
      "Remove and return the first value of a value-table list, type preserved (`()` if empty).",
      "latexml_core::state::shift_value",
    ),
  ),
  (
    "SkipSpaces",
    Mirror("$gullet->skipSpaces", ""),
    Doc::Rust(
      "Discard any run of spaces at the head of the input.",
      "latexml_core::gullet::skip_spaces",
    ),
  ),
  (
    "StartSemiverbatim",
    Mirror("StartSemiverbatim", ""),
    Doc::Rust(
      "Begin a SEMI-verbatim region: neutralize the math/special catcodes \
       (`^ _ ~ & $ # '`) to literal `OTHER`, but keep `\\ { }` special so control \
       sequences and `\\end{env}` still parse. This is NOT full verbatim — a backslash \
       command is still expanded (e.g. the Mathematica `\\[Gamma]` fires `\\[`, which \
       opens display math), and because `{` `}` stay group-markers a literal brace in \
       the source is consumed as grouping, not printed: `{}` yields nothing and `{x}` \
       yields `x`. Only the `{verbatim}` / fancyvrb environments read a body \
       as raw lines where even `\\ { }` are literal. Pass extra single-char strings to \
       neutralize them too — but not `\\`, since `\\end{env}` needs it. Pair with \
       `EndSemiverbatim` in a `DefEnvironment`'s `beforeDigest`/`beforeDigestEnd`.",
      "latexml_core::state::begin_semiverbatim",
    ),
  ),
  (
    "StepCounter",
    Mirror("StepCounter", "StepCounter!"),
    Doc::Rust(
      "Step a counter; usually you want `RefStepCounter` instead.",
      "latexml_core::binding::counter::dialect::step_counter",
    ),
  ),
  (
    "T_ACTIVE",
    Mirror("T_ACTIVE", "T_ACTIVE!"),
    Doc::Note(
      "One active-character `Token` (Perl `T_ACTIVE`), e.g. for `Let(T_ACTIVE(\"~\"), \"\\\\nobreakspace\")`.",
    ),
  ),
  (
    "T_ALIGN",
    Mirror("T_ALIGN", "T_ALIGN!"),
    Doc::Note("The `Token` `&` under `CC_ALIGN` (a Perl constant; call it as `T_ALIGN()`)."),
  ),
  (
    "T_BEGIN",
    Mirror("T_BEGIN", "T_BEGIN!"),
    Doc::Note("The `Token` `{` under `CC_BEGIN` (a Perl constant; call it as `T_BEGIN()`)."),
  ),
  (
    "T_CR",
    Mirror("T_CR", "T_CR!"),
    Doc::Note("The `Token` a newline under `CC_SPACE` (a Perl constant; call it as `T_CR()`)."),
  ),
  (
    "T_ARG",
    Mirror("T_ARG", "T_ARG!"),
    Doc::Note(
      "A parameter `Token` `#n` under `CC_ARG`, from an int 1-9 or a digit `Token` (Perl `T_ARG`).",
    ),
  ),
  (
    "T_COMMENT",
    Mirror("T_COMMENT", "T_COMMENT!"),
    Doc::Note(
      "A comment `Token`: `%` followed by the text, under `CC_COMMENT` (Perl `T_COMMENT`).",
    ),
  ),
  (
    "T_CS",
    Mirror("T_CS", "T_CS!"),
    Doc::Note(
      "One control-sequence `Token`; `Digest`, `Expand`, `UnTeX`, `absorb` take it as a one-token list.",
    ),
  ),
  (
    "T_END",
    Mirror("T_END", "T_END!"),
    Doc::Note("The `Token` `}` under `CC_END` (a Perl constant; call it as `T_END()`)."),
  ),
  (
    "T_LETTER",
    Mirror("T_LETTER", "T_LETTER!"),
    Doc::Note("One letter `Token` (catcode 11)."),
  ),
  (
    "T_MARKER",
    Mirror("T_MARKER", "T_MARKER!"),
    Doc::Note(
      "A marker `Token` under `CC_MARKER` (Perl `T_MARKER`, marked there \"don't use unless you know\").",
    ),
  ),
  (
    "T_MATH",
    Mirror("T_MATH", "T_MATH!"),
    Doc::Note("The `Token` `$` under `CC_MATH` (a Perl constant; call it as `T_MATH()`)."),
  ),
  (
    "T_OTHER",
    Mirror("T_OTHER", "T_OTHER!"),
    Doc::Note("One other-character `Token` (catcode 12)."),
  ),
  (
    "T_PARAM",
    Mirror("T_PARAM", "T_PARAM!"),
    Doc::Note("The `Token` `#` under `CC_PARAM` (a Perl constant; call it as `T_PARAM()`)."),
  ),
  (
    "T_SPACE",
    Mirror("T_SPACE", "T_SPACE!"),
    Doc::Note("The `Token` a space under `CC_SPACE` (a Perl constant; call it as `T_SPACE()`)."),
  ),
  (
    "T_SUB",
    Mirror("T_SUB", "T_SUB!"),
    Doc::Note("The `Token` `_` under `CC_SUB` (a Perl constant; call it as `T_SUB()`)."),
  ),
  (
    "T_SUPER",
    Mirror("T_SUPER", "T_SUPER!"),
    Doc::Note("The `Token` `^` under `CC_SUPER` (a Perl constant; call it as `T_SUPER()`)."),
  ),
  (
    "Tag",
    Mirror("Tag", "Tag!"),
    Doc::Rust(
      "Declare document-model properties for one element tag: `autoOpen`, `autoClose`, the `afterOpen`/`afterClose` hooks.",
      "latexml_core::binding::content::install_tag",
    ),
  ),
  (
    "TeX",
    Mirror("Digest(Tokenize($tex))", "TeX!"),
    Doc::Rust(
      "Tokenize a TeX source string (style catcodes) and digest it.",
      "latexml_core::stomach::digest",
    ),
  ),
  (
    "ToAttribute",
    Mirror("$box->toAttribute", ""),
    Doc::Rust(
      "The digested value as a string fit for an XML attribute.",
      "latexml_core::digested::Digested::to_attribute",
    ),
  ),
  (
    "ToString",
    Mirror("ToString", ""),
    Doc::Note(
      "Perl's `ToString`: a box, Tokens or Token as plain text, a whatsit as its reversion, `()` as `\"\"`.",
    ),
  ),
  (
    "Today",
    Mirror("", "Today!"),
    Doc::Rust(
      "Today's date, as `\\today` renders it.",
      "latexml_engine::base_utilities::today",
    ),
  ),
  (
    "Token",
    Mirror("Token", "Token!"),
    Doc::Note(
      "Perl's `Token(string, catcode)`: one `Token`; the catcode is a `CC_*` value, `CC_OTHER()` if omitted.",
    ),
  ),
  (
    "Tokenize",
    Mirror("Tokenize", "Tokenize!"),
    Doc::Rust(
      "Tokenize a string under the standard catcode table.",
      "latexml_core::mouth::tokenize",
    ),
  ),
  (
    "TokenizeInternal",
    Mirror("TokenizeInternal", "TokenizeInternal!"),
    Doc::Rust(
      "Tokenize a string under the style-file table, where `@` is a letter.",
      "latexml_core::mouth::tokenize_internal",
    ),
  ),
  (
    "Tokens",
    Mirror("Tokens", "Tokens!"),
    Doc::Note(
      "Perl's `Tokens(@tokens)`: a token list from `Token`s and `Tokens` (flattened), given as up to four arguments or as an array. Where a call takes a `Token` (`Let`, `XEquals`, …), a `Tokens` is an error.",
    ),
  ),
  (
    "UnTeX",
    Mirror("UnTeX", ""),
    Doc::Rust(
      "Tokens (or one `Token`) back to the TeX source that could have produced them.",
      "latexml_core::tokens::Tokens::untex",
    ),
  ),
  (
    "UnshiftValue",
    Mirror("UnshiftValue", ""),
    Doc::Rust(
      "Prepend a value (string/int/float/bool/Tokens/Token) to a value-table list, creating it (global) if absent.",
      "latexml_core::state::unshift_value",
    ),
  ),
  (
    "Warn",
    Mirror("Warn", "Warn!"),
    Doc::Note("Log a `Warning:` with the given category and object, plus optional details."),
  ),
  (
    "XEquals",
    Mirror("XEquals", "XEquals!"),
    Doc::Rust(
      "Whether two tokens (CS name strings or `Token`s) have the same meaning.",
      "latexml_core::state::x_equals",
    ),
  ),
  (
    "absorb",
    Mirror("$document->absorb", ""),
    Doc::Rust(
      "Absorb a digested value at the current insertion point.",
      "latexml_core::document::Document::absorb",
    ),
  ),
  (
    "absorbProperty",
    Mirror("", ""),
    Doc::Rust(
      "Absorb one of the whatsit's properties, by name.",
      "latexml_core::document::Document::absorb",
    ),
  ),
  (
    "absorbString",
    Mirror("$document->absorb", ""),
    Doc::Rust(
      "Absorb a plain string at the current insertion point.",
      "latexml_core::document::Document::absorb_string",
    ),
  ),
  (
    "addClass",
    Mirror("$document->addClass", ""),
    Doc::Rust(
      "Add CSS classes to a node, keeping those it already has.",
      "latexml_core::document::Document::add_class",
    ),
  ),
  (
    "appendClone",
    Mirror("$document->appendClone", ""),
    Doc::Rust(
      "Append COPIES of nodes, with fresh ids.",
      "latexml_core::document::Document::append_clone",
    ),
  ),
  (
    "arg",
    Mirror("", ""),
    Doc::Note("Append one argument to the command."),
  ),
  (
    "argString",
    Mirror("$whatsit->getArg", ""),
    Doc::Note("The whatsit's nth argument, reverted to source text."),
  ),
  (
    "args",
    Mirror("", ""),
    Doc::Note("Append several arguments at once, from an array."),
  ),
  (
    "assign_global",
    Mirror("AssignValue", ""),
    Doc::Note(
      "[`assign_value`](latexml_core::state::assign_value) with [`Scope::Global`](latexml_core::state::Scope::Global): the binding survives the enclosing TeX group.",
    ),
  ),
  (
    "assign_value",
    Mirror("AssignValue", ""),
    Doc::Note(
      "[`assign_value`](latexml_core::state::assign_value) with [`Scope::Local`](latexml_core::state::Scope::Local), TeX's default: the binding expires with the enclosing group.",
    ),
  ),
  (
    "children",
    Mirror("$node->childNodes", ""),
    Doc::Rust(
      "The node's child nodes.",
      "libxml::tree::Node::get_child_nodes",
    ),
  ),
  (
    "canContain",
    Mirror("$document->canContain", ""),
    Doc::Rust(
      "May an element (a node or a qname) contain a child qname? — the schema \
       query before opening an element.",
      "latexml_core::document::can_contain",
    ),
  ),
  (
    "closeElement",
    Mirror("$document->closeElement", ""),
    Doc::Rust(
      "Close the deepest open element of that name and return it (`()` when it is not open).",
      "latexml_core::document::Document::close_element",
    ),
  ),
  (
    "closeElementAt",
    Mirror("$document->closeElementAt", ""),
    Doc::Rust(
      "Close an element that was opened with `openElementAt`.",
      "latexml_core::document::Document::close_element_at",
    ),
  ),
  (
    "content",
    Mirror("$node->textContent", ""),
    Doc::Rust(
      "The node's text content.",
      "libxml::tree::Node::get_content",
    ),
  ),
  (
    "current_dir",
    Mirror("", ""),
    Doc::Note("Run the command in this working directory."),
  ),
  (
    "divide",
    Mirror("$v->divide", ""),
    Doc::Note("Perl's `$a->divide(n)`: as `/`; also by a Number."),
  ),
  (
    "divideround",
    Mirror("$v->divideround", ""),
    Doc::Note("Perl's `$a->divideround(n)`: the quotient rounded, as eTeX's."),
  ),
  (
    "env",
    Mirror("", ""),
    Doc::Note("Set one environment variable for the command."),
  ),
  (
    "findnode",
    Mirror("$document->findnode", ""),
    Doc::Rust(
      "The first node matching an XPath.",
      "latexml_core::document::Document::findnode",
    ),
  ),
  (
    "findnodes",
    Mirror("$document->findnodes", ""),
    Doc::Rust(
      "Every node matching an XPath.",
      "latexml_core::document::Document::findnodes",
    ),
  ),
  (
    "firstChild",
    Mirror("$node->firstChild", ""),
    Doc::Rust(
      "The node's first child, or `()`.",
      "libxml::tree::Node::get_first_child",
    ),
  ),
  (
    "floatToElement",
    Mirror("$document->floatToElement", ""),
    Doc::Rust(
      "Move the insertion point up to the nearest ancestor that can contain a \
       qname; returns the previous point to `setNode` back to (or `()`).",
      "latexml_core::document::Document::float_to_element",
    ),
  ),
  (
    "generateID",
    Mirror("", ""),
    Doc::Rust(
      "Give a node an `xml:id`, if it has none.",
      "latexml_core::document::Document::generate_id",
    ),
  ),
  (
    "getAttribute",
    Mirror("$node->getAttribute", ""),
    Doc::Rust(
      "An attribute's value, namespace-aware so `xml:id` is found.",
      "latexml_core::common::model::get_node_attribute",
    ),
  ),
  (
    "getElement",
    Mirror("$document->getElement", ""),
    Doc::Rust(
      "The element at, or containing, the insertion point.",
      "latexml_core::document::Document::get_element",
    ),
  ),
  (
    "getNode",
    Mirror("$document->getNode", ""),
    Doc::Rust(
      "The current insertion point.",
      "latexml_core::document::Document::get_node",
    ),
  ),
  (
    "getArg",
    Mirror("$whatsit->getArg", ""),
    Doc::Rust(
      "The nth argument (from 1), as a constructor body receives it: a `Digested` handle, `Tokens` for an undigested one, `()` when absent.",
      "latexml_core::whatsit::Whatsit::get_arg",
    ),
  ),
  (
    "getShrink",
    Mirror("$v->getShrink", ""),
    Doc::Note("A Glue's shrink as a Dimension (Glue.pm)."),
  ),
  (
    "getShrinkOrder",
    Mirror("$v->getShrinkOrder", ""),
    Doc::Note("A Glue's shrink fill order as a Number, 0 (finite) to 3 (filll)."),
  ),
  (
    "getStretch",
    Mirror("$v->getStretch", ""),
    Doc::Note("A Glue's stretch as a Dimension (Glue.pm)."),
  ),
  (
    "getStretchOrder",
    Mirror("$v->getStretchOrder", ""),
    Doc::Note("A Glue's stretch fill order as a Number, 0 (finite) to 3 (filll)."),
  ),
  (
    "hasAttribute",
    Mirror("$node->hasAttribute", ""),
    Doc::Rust(
      "Whether the node carries that attribute.",
      "latexml_core::common::model::get_node_attribute",
    ),
  ),
  (
    "insertElement",
    Mirror("$document->insertElement", ""),
    Doc::Rust(
      "Open, absorb and close in one step; returns the new element.",
      "latexml_core::document::Document::insert_element",
    ),
  ),
  (
    "insertPI",
    Mirror("$document->insertPI", ""),
    Doc::Rust(
      "Insert a processing instruction: a target, with optional attributes.",
      "latexml_core::document::Document::insert_pi",
    ),
  ),
  (
    "insertXML",
    Mirror("", ""),
    Doc::Rust(
      "Splice ALREADY-PARSED nodes in at the insertion point.",
      "latexml_core::document::Document::insert_nodes",
    ),
  ),
  (
    "larger",
    Mirror("$v->larger", ""),
    Doc::Note("The larger of two value handles of one type, by `valueOf`."),
  ),
  (
    "lookup_value",
    Mirror("LookupValue", ""),
    Doc::Note(
      "[`lookup_value`](latexml_core::state::lookup_value) coerced to a string; empty when unset, so an unbound key and an empty one are indistinguishable here.",
    ),
  ),
  (
    "maybeCloseElement",
    Mirror("$document->maybeCloseElement", ""),
    Doc::Rust(
      "Close an element if it is open and closeable and return it; otherwise `()`.",
      "latexml_core::document::Document::maybe_close_element",
    ),
  ),
  (
    "multiply",
    Mirror("$v->multiply", ""),
    Doc::Note("Perl's `$a->multiply(n)`: as `*`; also by a Number."),
  ),
  (
    "name",
    Mirror("$node->localname", ""),
    Doc::Rust("The node's local name.", "libxml::tree::Node::get_name"),
  ),
  (
    "negate",
    Mirror("$v->negate", ""),
    Doc::Note("A value handle negated (Glue: its stretch and shrink too)."),
  ),
  (
    "neutralize_font",
    Mirror("", ""),
    Doc::Rust(
      "Reset the text and math fonts to their defaults, group-locally.",
      "latexml_engine::base_utilities::neutralize_font",
    ),
  ),
  (
    "nextSibling",
    Mirror("$node->nextSibling", ""),
    Doc::Rust(
      "The node's next sibling, or `()`.",
      "libxml::tree::Node::get_next_sibling",
    ),
  ),
  (
    "openElement",
    Mirror("$document->openElement", ""),
    Doc::Rust(
      "Open an element (with an optional attribute map), make it the insertion point and return it.",
      "latexml_core::document::Document::open_element",
    ),
  ),
  (
    "openElementAt",
    Mirror("$document->openElementAt", ""),
    Doc::Rust(
      "Open an element at a given node rather than the insertion point.",
      "latexml_core::document::Document::open_element_at",
    ),
  ),
  (
    "output",
    Mirror("", ""),
    Doc::Note(
      "Run the command and return `#{ status, stdout, stderr }`. Refused when `LATEXML_DISABLE_SHELL_ESCAPE` is set (see SAFETY.md).",
    ),
  ),
  (
    "parent",
    Mirror("$node->parentNode", ""),
    Doc::Note(
      "The parent node ([`get_parent`](libxml::tree::Node::get_parent)), or `()` when there is none. A node from `ParseXML` is a TOP-LEVEL node of its chunk, so it reports NO parent: everything above it is a parse artifact ([`is_parse_artifact`](latexml_core::common::xml::is_parse_artifact)), never markup the script wrote.",
    ),
  ),
  (
    "prevSibling",
    Mirror("$node->previousSibling", ""),
    Doc::Rust(
      "The node's previous sibling, or `()`.",
      "libxml::tree::Node::get_prev_sibling",
    ),
  ),
  (
    "propertyString",
    Mirror("$whatsit->getProperty", ""),
    Doc::Note("One property of the whatsit, as a string."),
  ),
  (
    "ptValue",
    Mirror("$v->ptValue", ""),
    Doc::Note(
      "A Dimension's or Glue's natural size in points, rounded to `prec` decimals (2 by default; Dimension.pm).",
    ),
  ),
  (
    "pushAfterConstruct",
    Mirror("push(@{$$defn{afterConstruct}}, …)", ""),
    Doc::Note("Append a hook to that definition's `afterConstruct` list."),
  ),
  (
    "pushAfterDigest",
    Mirror("push(@{$$defn{afterDigest}}, …)", ""),
    Doc::Note("Append a hook to that definition's `afterDigest` list."),
  ),
  (
    "pushAfterDigestBody",
    Mirror("push(@{$$defn{afterDigestBody}}, …)", ""),
    Doc::Note("Append a hook to that definition's `afterDigestBody` list."),
  ),
  (
    "pushBeforeConstruct",
    Mirror("push(@{$$defn{beforeConstruct}}, …)", ""),
    Doc::Note("Append a hook to that definition's `beforeConstruct` list."),
  ),
  (
    "pushBeforeDigest",
    Mirror("push(@{$$defn{beforeDigest}}, …)", ""),
    Doc::Note("Append a hook to that definition's `beforeDigest` list."),
  ),
  (
    "pxValue",
    Mirror("$v->pxValue", ""),
    Doc::Note("A Dimension's or Glue's natural size in pixels at the `DPI` value (Dimension.pm)."),
  ),
  (
    "qname",
    Mirror("$document->getNodeQName($node)", ""),
    Doc::Rust(
      "The node's qualified name (`ltx:section`, `#PCDATA`, …).",
      "latexml_core::common::model::with_node_qname",
    ),
  ),
  (
    "readArg",
    Mirror("$gullet->readArg", ""),
    Doc::Rust(
      "`gullet.readArg()`: `ReadArg` on a `Gullet` handle.",
      "latexml_core::gullet::read_arg",
    ),
  ),
  (
    "readOptional",
    Mirror("$gullet->readOptional", ""),
    Doc::Rust(
      "`gullet.readOptional()`: `ReadOptional` on a `Gullet` handle.",
      "latexml_core::gullet::read_optional",
    ),
  ),
  (
    "readToken",
    Mirror("$gullet->readToken", ""),
    Doc::Rust(
      "`gullet.readToken()`: `ReadToken` on a `Gullet` handle.",
      "latexml_core::gullet::read_token",
    ),
  ),
  (
    "readUntil",
    Mirror("$gullet->readUntil", ""),
    Doc::Rust(
      "`gullet.readUntil(delim)`: `ReadUntil` on a `Gullet` handle.",
      "latexml_core::gullet::read_until",
    ),
  ),
  (
    "removeAttribute",
    Mirror("$node->removeAttribute", ""),
    Doc::Rust(
      "Remove a possibly-prefixed attribute.",
      "latexml_core::common::model::remove_node_attribute",
    ),
  ),
  (
    "removeNode",
    Mirror("$document->removeNode", ""),
    Doc::Rust(
      "Remove a node, releasing the ids under it.",
      "latexml_core::document::Document::remove_node",
    ),
  ),
  (
    "renameNode",
    Mirror("$document->renameNode", ""),
    Doc::Rust(
      "Rename an element, rebuilding it through the model.",
      "latexml_core::document::Document::rename_node",
    ),
  ),
  (
    "replaceNode",
    Mirror("$document->replaceNode", ""),
    Doc::Rust(
      "Replace a node by other nodes.",
      "latexml_core::document::Document::replace_node",
    ),
  ),
  // `setAttribute` is deliberately absent: it is registered on two handles and
  // means something different on each, so it lives in HANDLE_DOCS.
  (
    "setContent",
    Mirror("", ""),
    Doc::Rust(
      "Replace the node's text content.",
      "libxml::tree::Node::set_content",
    ),
  ),
  (
    "setNode",
    Mirror("$document->setNode", ""),
    Doc::Rust(
      "Move the insertion point to a node.",
      "latexml_core::document::Document::set_node",
    ),
  ),
  (
    "setProperty",
    Mirror("$whatsit->setProperty", ""),
    Doc::Note(
      "Set a property (any value) on the whatsit. Only a DIGEST hook may: in a construction hook the whatsit is already read-only, and this is a clean script error.",
    ),
  ),
  (
    "sign",
    Mirror("$v->sign", ""),
    Doc::Note("-1, 0 or 1 by a value handle's `valueOf`."),
  ),
  (
    "skipSpaces",
    Mirror("$gullet->skipSpaces", ""),
    Doc::Rust(
      "`gullet.skipSpaces()`: `SkipSpaces` on a `Gullet` handle.",
      "latexml_core::gullet::skip_spaces",
    ),
  ),
  (
    "smaller",
    Mirror("$v->smaller", ""),
    Doc::Note("The smaller of two value handles of one type, by `valueOf`."),
  ),
  (
    "spValue",
    Mirror("$v->spValue", ""),
    Doc::Note("A Dimension's or Glue's natural size in sp."),
  ),
  (
    "subtract",
    Mirror("$v->subtract", ""),
    Doc::Note("Perl's `$a->subtract($b)`: as `-`."),
  ),
  (
    "toAttribute",
    Mirror("$v->toAttribute", ""),
    Doc::Note(
      "A value handle as an XML attribute value (`3.0pt`; Dimension.pm `attributeformat`).",
    ),
  ),
  (
    "unlink",
    Mirror("$node->unbindNode", ""),
    Doc::Rust(
      "Detach the node from its tree.",
      "libxml::tree::Node::unlink",
    ),
  ),
  (
    "unshiftAfterConstruct",
    Mirror("unshift(@{$$defn{afterConstruct}}, …)", ""),
    Doc::Note("Prepend a hook to that definition's `afterConstruct` list."),
  ),
  (
    "unshiftAfterDigest",
    Mirror("unshift(@{$$defn{afterDigest}}, …)", ""),
    Doc::Note("Prepend a hook to that definition's `afterDigest` list."),
  ),
  (
    "unshiftAfterDigestBody",
    Mirror("unshift(@{$$defn{afterDigestBody}}, …)", ""),
    Doc::Note("Prepend a hook to that definition's `afterDigestBody` list."),
  ),
  (
    "unshiftBeforeConstruct",
    Mirror("unshift(@{$$defn{beforeConstruct}}, …)", ""),
    Doc::Note("Prepend a hook to that definition's `beforeConstruct` list."),
  ),
  (
    "unshiftBeforeDigest",
    Mirror("unshift(@{$$defn{beforeDigest}}, …)", ""),
    Doc::Note("Prepend a hook to that definition's `beforeDigest` list."),
  ),
  (
    "unwrapNodes",
    Mirror("$document->unwrapNodes", ""),
    Doc::Rust(
      "Replace a node by its own children.",
      "latexml_core::document::Document::unwrap_nodes",
    ),
  ),
  (
    "valueOf",
    Mirror("$v->valueOf", ""),
    Doc::Note(
      "A value handle's integer value: a Number's value, a Dimension's or Glue's sp (Glue: the natural size).",
    ),
  ),
  (
    "whatsit",
    Mirror("$whatsit", ""),
    Doc::Note(
      "A handle on the whatsit under construction, inside a hook. USING it outside one is a clean script error, not a crash.",
    ),
  ),
  (
    "wrapNodes",
    Mirror("$document->wrapNodes", ""),
    Doc::Rust(
      "Wrap a run of sibling nodes in a new element.",
      "latexml_core::document::Document::wrap_nodes",
    ),
  ),
  (
    "AddToMacro",
    Mirror("AddToMacro", "AddToMacro!"),
    Doc::Rust(
      "Append tokens (or TeX source) to a macro's expansion, keeping its lock, as Perl's `AddToMacro`; a \
       non-expandable `cs` Warns and is ignored, an undefined one is defined with the tokens.",
      "latexml_core::binding::content::add_to_macro",
    ),
  ),
  (
    "BeginItemize",
    Mirror("beginItemize", "BeginItemize!"),
    Doc::Rust(
      "Open a list level: the `properties` map numbering its items (Perl `beginItemize`).",
      "latexml_core::binding::counter::dialect::begin_itemize",
    ),
  ),
  (
    "CleanID",
    Mirror("CleanID", ""),
    Doc::Rust(
      "Clean a string into a valid `xml:id`.",
      "latexml_core::common::cleaners::clean_id",
    ),
  ),
  (
    "CleanLabel",
    Mirror("CleanLabel", ""),
    Doc::Rust(
      "A label's key: `LABEL:` (or the given prefix) plus the label, spaces to `_`.",
      "latexml_core::common::cleaners::clean_label",
    ),
  ),
  (
    "DeclareFontMap",
    Mirror("DeclareFontMap", "DeclareFontMap!"),
    Doc::Note(
      "Declare an encoding's font map: an array of 1-char strings or `()`, with Perl's `family` and `*_mathstyle` options.",
    ),
  ),
  (
    "DeclareFontMapMultichar",
    Mirror("", "DeclareFontMapMultichar!"),
    Doc::Note(
      "Declare the multi-character glyphs of a font map (optionally a family's): position (decimal string) to its text.",
    ),
  ),
  (
    "DigestIf",
    Mirror("DigestIf", "DigestIf!"),
    Doc::Rust(
      "Digest a control sequence if it is defined; `()` otherwise.",
      "latexml_core::binding::content::digest_if",
    ),
  ),
  (
    "Equals",
    Mirror("Equals", ""),
    Doc::Note(
      "Perl's `Equals`: `()` equals only `()`; tokens compare their meanings (themselves when neither has one), `Tokens` token by token, other values by their string.",
    ),
  ),
  (
    "Explode",
    Mirror("Explode", "Explode!"),
    Doc::Note("A string's characters as `Tokens`, each OTHER (a space a space token)."),
  ),
  (
    "ExplodeText",
    Mirror("ExplodeText", "ExplodeText!"),
    Doc::Note("A string's characters as `Tokens`, letters LETTER and the rest OTHER."),
  ),
  (
    "FindFile",
    Mirror("FindFile", "FindFile!"),
    Doc::Rust(
      "The path of a file on the search paths (Perl's `type`/`notex`/`noltxml`/`searchpaths_only`), or `()`.",
      "latexml_core::binding::content::find_file",
    ),
  ),
  (
    "Input",
    Mirror("Input", "Input!"),
    Doc::Rust(
      "Input a TeX file (`\\input`'s core).",
      "latexml_core::binding::content::input",
    ),
  ),
  (
    "Invocation",
    Mirror("Invocation", "Invocation!"),
    Doc::Rust(
      "The tokens invoking a control sequence on its arguments (each `Tokens`, TeX source or `()` for an omitted optional); TeX with `#n` is an anonymous macro.",
      "latexml_core::binding::content::build_invocation",
    ),
  ),
  (
    "IsEmpty",
    Mirror("IsEmpty", ""),
    Doc::Note(
      "Perl's `IsEmpty`: true when nothing but spaces, braces and other non-printing tokens (or empty digested content).",
    ),
  ),
  (
    "LoadPool",
    Mirror("LoadPool", "LoadPool!"),
    Doc::Rust(
      "Load a pool (`<name>.pool`).",
      "latexml_core::binding::content::input_definitions",
    ),
  ),
  (
    "LookupRegisterOrDefault",
    Mirror("", "LookupRegisterOrDefault!"),
    Doc::Note(
      "A register's typed value (as `LookupRegister`), a Number 0 when `cs` is no register, without a Warning.",
    ),
  ),
  (
    "ProcessKeyOptions",
    Mirror("", "ProcessKeyOptions!"),
    Doc::Rust(
      "`ProcessOptions` for a package whose options are key=value pairs; `\\@curroptions` is kept.",
      "latexml_core::binding::content::process_key_options",
    ),
  ),
  (
    "Roman",
    Mirror("Roman", "Roman!"),
    Doc::Note("Upper-case roman numerals of a number, as `Tokens`."),
  ),
  (
    "SetCounter",
    Mirror("SetCounter", "SetCounter!"),
    Doc::Rust(
      "Set a counter's value globally.",
      "latexml_core::binding::counter::dialect::set_counter",
    ),
  ),
  (
    "UTF",
    Mirror("UTF", ""),
    Doc::Note("The character of a Unicode code point."),
  ),
  (
    "getEncoding",
    Mirror("$font->getEncoding", ""),
    Doc::Note("A `Font`'s encoding, or `()`."),
  ),
  (
    "getFamily",
    Mirror("$font->getFamily", ""),
    Doc::Note("A `Font`'s family (`serif`, `typewriter`, …), or `()`."),
  ),
  (
    "getLanguage",
    Mirror("$font->getLanguage", ""),
    Doc::Note("A `Font`'s language, or `()`."),
  ),
  (
    "getSeries",
    Mirror("$font->getSeries", ""),
    Doc::Note("A `Font`'s series, or `()`."),
  ),
  (
    "getShape",
    Mirror("$font->getShape", ""),
    Doc::Note("A `Font`'s shape, or `()`."),
  ),
  (
    "getSize",
    Mirror("$font->getSize", ""),
    Doc::Note("A `Font`'s size in points, or `()`."),
  ),
  (
    "roman",
    Mirror("roman", "roman!"),
    Doc::Note("Lower-case roman numerals of a number, as `Tokens`."),
  ),
];

/// Calls whose meaning depends on the HANDLE they are called on. One name
/// registered on two receivers is two different functions, and a single
/// name-keyed [`DOCS`] entry would document one of them wrongly:
/// `document.setAttribute` writes to wherever the document is currently
/// positioned and goes through the model (prefix decoding, id bookkeeping),
/// while `node.setAttribute` writes to the node the script is holding.
///
/// [`ambiguous_names`] enforces the rule: a name that appears under more than
/// one handle must have an entry here for EACH of them.
const HANDLE_DOCS: &[(&str, &str, Mirror, Doc)] = &[
  (
    "Document",
    "getProperties",
    Mirror("$whatsit->getProperties", ""),
    Doc::Note("The whatsit's whole property map (a Rhai object map), inside a constructor body."),
  ),
  (
    "Document",
    "getProperty",
    Mirror("$whatsit->getProperty", ""),
    Doc::Note("One whatsit property by name, inside a constructor body; `()` when absent."),
  ),
  (
    "Whatsit",
    "getProperties",
    Mirror("$whatsit->getProperties", ""),
    Doc::Rust(
      "The whole property map (a Rhai object map), each value typed.",
      "latexml_core::BoxOps::get_properties",
    ),
  ),
  (
    "Whatsit",
    "getProperty",
    Mirror("$whatsit->getProperty", ""),
    Doc::Rust(
      "One property by name, its typed value (string / int / bool / `Tokens` / `Digested` handle / …); `()` when absent.",
      "latexml_core::BoxOps::get_property",
    ),
  ),
  (
    "Dimension",
    "to_string",
    Mirror("$v->toString", ""),
    Doc::Note("The value as TeX prints it (`toString`, `\\the`): `1.0pt plus 2.0pt`."),
  ),
  (
    "Glue",
    "to_string",
    Mirror("$v->toString", ""),
    Doc::Note("The value as TeX prints it (`toString`, `\\the`): `1.0pt plus 2.0pt`."),
  ),
  (
    "MuGlue",
    "to_string",
    Mirror("$v->toString", ""),
    Doc::Note("The value as TeX prints it (`toString`, `\\the`): `1.0pt plus 2.0pt`."),
  ),
  (
    "Number",
    "to_string",
    Mirror("$v->toString", ""),
    Doc::Note("The value as TeX prints it (`toString`, `\\the`): `1.0pt plus 2.0pt`."),
  ),
  (
    "Node",
    "toString",
    Mirror("$node->toString", ""),
    Doc::Note("The node and its subtree serialized back to markup — the inverse of `ParseXML`."),
  ),
  (
    "Dimension",
    "toString",
    Mirror("$v->toString", ""),
    Doc::Note("Perl's `toString`: the value as TeX prints it (`\\the`), `1.0pt plus 2.0pt`."),
  ),
  (
    "Glue",
    "toString",
    Mirror("$v->toString", ""),
    Doc::Note("Perl's `toString`: the value as TeX prints it (`\\the`), `1.0pt plus 2.0pt`."),
  ),
  (
    "MuGlue",
    "toString",
    Mirror("$v->toString", ""),
    Doc::Note("Perl's `toString`: the value as TeX prints it (`\\the`), `1.0pt plus 2.0pt`."),
  ),
  (
    "Number",
    "toString",
    Mirror("$v->toString", ""),
    Doc::Note("Perl's `toString`: the value as TeX prints it (`\\the`), `1.0pt plus 2.0pt`."),
  ),
  (
    "Token",
    "to_string",
    Mirror("", ""),
    Doc::Rust(
      "The token as TeX source (what a string context shows).",
      "latexml_core::tokens::Tokens::untex",
    ),
  ),
  (
    "Digested",
    "to_string",
    Mirror("", ""),
    Doc::Rust(
      "The box's `ToString` (what a string context shows).",
      "latexml_core::digested::Digested",
    ),
  ),
  (
    "KeyVals",
    "to_string",
    Mirror("", ""),
    Doc::Rust(
      "The keyval set as `k=v,…` TeX (what a string context shows).",
      "latexml_core::keyvals::KeyVals",
    ),
  ),
  (
    "Stomach",
    "getGullet",
    Mirror("$stomach->getGullet", ""),
    Doc::Note("`stomach.getGullet()`: the `Gullet` handle, Perl's `$stomach->getGullet`."),
  ),
  (
    "Whatsit",
    "to_string",
    Mirror("ToString($whatsit)", ""),
    Doc::Note(
      "The whatsit in a string: Perl's `ToString($whatsit)`, its reversion (an error inside its own reversion).",
    ),
  ),
  (
    "Tokens",
    "to_string",
    Mirror("", ""),
    Doc::Rust(
      "The tokens as TeX source (what a string context shows).",
      "latexml_core::tokens::Tokens::untex",
    ),
  ),
  (
    "Document",
    "setAttribute",
    Mirror("$document->setAttribute", ""),
    Doc::Rust(
      "Set an attribute on the current node, if the model allows it.",
      "latexml_core::document::Document::set_attribute",
    ),
  ),
  (
    "Node",
    "setAttribute",
    Mirror("$node->setAttribute", ""),
    Doc::Rust(
      "Set an attribute on this node directly.",
      "libxml::tree::Node::set_attribute",
    ),
  ),
];

/// Registered calls with neither a [`DOCS`] nor a [`HANDLE_DOCS`] entry — a new
/// `register_fn` nobody documented. Reported as `Handle::name` so the failure
/// message says which table to add to.
pub(super) fn undocumented_names(engine: &Engine) -> Vec<String> {
  let mut missing: Vec<String> = registered_calls(engine)
    .into_iter()
    .filter(|(receiver, name)| lookup(receiver.as_deref(), name).is_none())
    .map(|(receiver, name)| qualified(receiver.as_deref(), &name))
    .collect();
  missing.sort();
  missing.dedup();
  missing
}

/// Entries naming a call the engine no longer registers — from either table, so
/// a rename cannot leave a stale row behind in whichever one holds it.
pub(super) fn stale_names(engine: &Engine) -> Vec<String> {
  let live = registered_calls(engine);
  let docs = DOCS
    .iter()
    .map(|(name, ..)| (None, *name))
    .filter(|(_, name)| !live.iter().any(|(_, n)| n == name));
  let handle_docs = HANDLE_DOCS
    .iter()
    .map(|(receiver, name, ..)| (Some(*receiver), *name))
    .filter(|(receiver, name)| {
      !live
        .iter()
        .any(|(r, n)| r.as_deref() == *receiver && n == name)
    });
  docs
    .chain(handle_docs)
    .map(|(receiver, name)| qualified(receiver, name))
    .collect()
}

/// Names registered on more than one handle that do not have a [`HANDLE_DOCS`]
/// entry for each — they would otherwise share one [`DOCS`] cell, which can only
/// be right for one of them (`setAttribute` shipped documented as
/// `Document::get_node` until this check existed).
pub(super) fn ambiguous_names(engine: &Engine) -> Vec<String> {
  let calls = registered_calls(engine);
  let mut ambiguous: Vec<String> = calls
    .iter()
    .filter(|(receiver, name)| {
      calls.iter().any(|(r, n)| n == name && r != receiver)
        && !HANDLE_DOCS
          .iter()
          .any(|(r, n, ..)| Some(*r) == receiver.as_deref() && n == name)
    })
    .map(|(receiver, name)| qualified(receiver.as_deref(), name))
    .collect();
  ambiguous.sort();
  ambiguous.dedup();
  ambiguous
}

/// Every registered call as `(handle, name)` — the pair a doc entry is keyed
/// by, since the same name on two handles is two different calls.
fn registered_calls(engine: &Engine) -> Vec<(Option<String>, String)> {
  engine
    .gen_fn_signatures(false)
    .iter()
    .map(|s| {
      let sig = Signature::parse(s);
      (sig.receiver, sig.name)
    })
    .collect()
}

fn qualified(receiver: Option<&str>, name: &str) -> String {
  match receiver {
    Some(handle) => format!("{handle}::{name}"),
    None => name.to_string(),
  }
}

/// The doc entry for one call: the handle-specific one when there is one,
/// otherwise the name-keyed one.
fn lookup(receiver: Option<&str>, name: &str) -> Option<&'static Doc> {
  lookup_entry(receiver, name).map(|(_, doc)| doc)
}

/// [`lookup`]'s entry with its [`Mirror`].
fn lookup_entry(receiver: Option<&str>, name: &str) -> Option<(&'static Mirror, &'static Doc)> {
  HANDLE_DOCS
    .iter()
    .find(|(r, n, ..)| Some(*r) == receiver && *n == name)
    .map(|(_, _, mirror, doc)| (mirror, doc))
    .or_else(|| {
      DOCS
        .iter()
        .find(|(n, ..)| *n == name)
        .map(|(_, mirror, doc)| (mirror, doc))
    })
}

/// The documentation cell for one call: a rustdoc intra-doc link, or the note; a deprecated alias says so first.
fn doc_cell(receiver: Option<&str>, name: &str) -> String {
  let doc = match lookup(receiver, name) {
    Some(Doc::Rust(summary, path)) => {
      format!("{summary} ([`{}`]({path}))", short_path(path))
    },
    Some(Doc::Note(note)) => (*note).to_string(),
    None => String::new(),
  };
  let alias = match receiver {
    Some(handle) => format!("{}.{name}", handle.to_lowercase()),
    None => name.to_string(),
  };
  let Some((_, new)) = super::DEPRECATED_ALIASES
    .iter()
    .find(|(old, _)| *old == alias)
  else {
    return doc;
  };
  let new = if new.contains('(') {
    format!("`{new}`")
  } else {
    new.to_string()
  };
  // A replacement spelled as the same call deprecates only one of its forms.
  if new.starts_with(&format!("`{name}(")) {
    format!("{doc} **Deprecated:** its older form (Warns once per binding load); use {new}.")
  } else {
    format!("**Deprecated** (Warns once per binding load): use {new}. {doc}")
  }
}

/// Show `Document::insert_xml`, not the whole module path — the type qualifier
/// is what disambiguates, the module chain is noise in a table cell.
fn short_path(path: &str) -> String {
  let Some((head, tail)) = path.rsplit_once("::") else {
    return path.to_string();
  };
  match head.rsplit_once("::") {
    Some((_, ty)) if ty.starts_with(char::is_uppercase) => format!("{ty}::{tail}"),
    _ => tail.to_string(),
  }
}

/// [`PERL_GAPS`] entries the engine now registers: move each to its [`DOCS`] entry's [`Mirror`].
pub(super) fn stale_gaps(engine: &Engine) -> Vec<&'static str> {
  let live = registered_calls(engine);
  PERL_GAPS
    .iter()
    .map(|(perl, _)| *perl)
    .filter(|perl| live.iter().any(|(r, n)| r.is_none() && n == perl))
    .collect()
}

/// Every macro the cheat sheet names, without its `!`, for the check that each is still defined.
pub(super) fn named_macros() -> Vec<&'static str> {
  let docs = DOCS.iter().map(|(_, Mirror(_, mac), _)| *mac);
  let gaps = PERL_GAPS.iter().map(|(_, mac)| *mac);
  docs
    .chain(gaps)
    .filter_map(|mac| mac.strip_suffix('!'))
    .collect()
}

/// Render the three-column cheat sheet, Perl | Rust macro | Rhai, as Markdown: a row per registered call (in
/// [`GROUPS`] order) and per [`PERL_GAPS`] entry, then the empty cells listed as the dialect's gaps.
pub(super) fn generate_cheatsheet(engine: &Engine) -> String {
  let mut calls = registered_calls(engine);
  calls.sort();
  calls.dedup();
  let cell = |s: &str| {
    if s.is_empty() {
      String::new()
    } else {
      format!("`{s}`")
    }
  };
  let mut out = String::from(CHEATSHEET_HEADER);
  out.push_str("\n| Perl | Rust macro | Rhai |\n|---|---|---|\n");
  let (mut no_perl, mut no_macro) = (Vec::new(), Vec::new());
  for (receiver, ..) in GROUPS {
    for (_, name) in calls.iter().filter(|(r, _)| r.as_deref() == *receiver) {
      let Some((Mirror(perl, mac), doc)) = lookup_entry(*receiver, name) else {
        continue;
      };
      let rhai = match receiver {
        Some(handle) => format!("{}.{name}", handle.to_lowercase()),
        None => name.clone(),
      };
      out.push_str(&format!(
        "| {} | {} | {} |\n",
        cell(perl),
        cell(mac),
        cell(&rhai)
      ));
      if perl.is_empty() {
        no_perl.push(format!("`{rhai}`"));
      }
      // A plain Perl function the macros do not spell: compiled bindings call the Rust function instead.
      if receiver.is_none()
        && mac.is_empty()
        && !perl.is_empty()
        && perl.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
      {
        no_macro.push(match doc {
          Doc::Rust(_, path) => format!("`{name}` (`{}`)", short_path(path)),
          Doc::Note(_) => format!("`{name}`"),
        });
      }
    }
  }
  for (perl, mac) in PERL_GAPS {
    out.push_str(&format!("| {} | {} |  |\n", cell(perl), cell(mac)));
  }
  let no_rhai: Vec<String> = PERL_GAPS
    .iter()
    .map(|(perl, _)| format!("`{perl}`"))
    .collect();
  for (title, list) in [
    ("Perl calls the dialect lacks (an empty Rhai cell)", no_rhai),
    (
      "Rhai calls with no Perl counterpart (an empty Perl cell)",
      no_perl,
    ),
    (
      "Perl functions with no macro (compiled bindings call the Rust function named)",
      no_macro,
    ),
  ] {
    out.push_str(&format!(
      "\n## {title}\n\n{} calls: {}.\n",
      list.len(),
      list.join(", ")
    ));
  }
  out
}

const CHEATSHEET_HEADER: &str = r#"# Perl | Rust macro | Rhai: the binding dialects side by side

One row per call: its Perl LaTeXML spelling (a `Package.pm` function, or the Perl method or expression that does the
same), the compile-time Rust macro that lowers to the same core call, and the runtime `.rhai` call. **Generated** from
`api_doc.rs` with [`API.md`](API.md), and checked with it on every test run (`api_reference_is_up_to_date`); regenerate
with `UPDATE_API_DOC=1 cargo test -p latexml_contrib --lib api_reference_is_up_to_date`.

An empty cell is a gap, listed at the end: a Perl call the dialect lacks, a Rhai call Perl has no counterpart for (a
Rhai-only helper), or a Perl function the macros do not spell. Methods (`document.openElement`) have no macro: a
compiled binding calls the Rust method. Each Rhai call's arguments and meaning are in [`API.md`](API.md); the
newcomer guide is [`docs/parity/RHAI_BINDINGS_GUIDE.md`](../../../docs/parity/RHAI_BINDINGS_GUIDE.md).
"#;

/// Render the whole reference as Markdown.
pub(super) fn generate(engine: &Engine) -> String {
  let signatures = engine.gen_fn_signatures(false);
  let mut parsed: Vec<Signature> = signatures.iter().map(|s| Signature::parse(s)).collect();
  // Rhai hands these back in hash order — sort so the file is stable and its
  // diffs are reviewable.
  parsed.sort_by(|a, b| (&a.name, &a.params).cmp(&(&b.name, &b.params)));

  let mut out = String::new();
  out.push_str(HEADER);
  for (receiver, title, blurb) in GROUPS {
    let group: Vec<&Signature> = parsed
      .iter()
      .filter(|s| s.receiver.as_deref() == *receiver)
      .collect();
    if group.is_empty() {
      continue;
    }
    // Overloads collapse into one row: they share a name, and a name is what
    // has documentation.
    let mut by_name: Vec<(&str, Vec<String>)> = Vec::new();
    for sig in &group {
      match by_name.last_mut() {
        Some((n, sigs)) if *n == sig.name => sigs.push(sig.render()),
        _ => by_name.push((&sig.name, vec![sig.render()])),
      }
    }
    out.push_str(&format!(
      "\n## {title}\n\n{blurb}\n\n{} function{}, {} call{}.\n\n\
       | call | documentation |\n|---|---|\n",
      by_name.len(),
      if by_name.len() == 1 { "" } else { "s" },
      group.len(),
      if group.len() == 1 { "" } else { "s" }
    ));
    for (name, sigs) in by_name {
      out.push_str(&format!(
        "| {} | {} |\n",
        sigs
          .iter()
          .map(|s| format!("`{s}`"))
          .collect::<Vec<_>>()
          .join("<br>"),
        doc_cell(receiver.as_deref(), name)
      ));
    }
  }
  out
}

const HEADER: &str = r#"# The `.rhai` binding interface

Every function a runtime (Rhai) binding can call, grouped by the handle it is
called on. **Generated** from the live engine via Rhai's reflections API
(`Engine::gen_fn_signatures`) and checked on every test run, so it always
matches what is actually registered — see `api_doc.rs`.

Overloads share a row — the accepted argument shapes differ, what the call means
does not.

The **documentation** column says in one line what each call does, and links the
Rust item that implements it — where the exact semantics live, and what keeps
this reference from drifting out of step with the engine. Read the line; follow
the link when you need the detail, the edge cases, or the Perl original it is
ported from.

Some calls have no such counterpart — a Rhai-only helper, or a shim whose whole
behaviour is in the registration — and carry only the description. A name
registered on two different handles is two different calls, and is documented
separately under each.

Types are Rhai's: `string`, `int`, `bool`, `array`, `map`, `Fn` (a closure),
`?` (any value), and the handle types named by the sections below. A trailing `?`
on a handle — `Node?` — means "that handle, or `()` when there is none". A
missing return type means the function returns nothing. Anything that can fail raises an
ordinary script error; what that costs is in
[`script_bindings_plan.md`](https://github.com/dginev/latexml-oxide/blob/main/docs/parity/script_bindings_plan.md)
under "Failure containment" — briefly, a failing body degrades its own binding
and the conversion continues.

Argument NAMES are not shown because Rhai does not record them for
closure-registered functions; only the accepted types are.

New to the dialect? Start with the guide, [`RHAI_BINDINGS_GUIDE.md`](https://github.com/dginev/latexml-oxide/blob/main/docs/parity/RHAI_BINDINGS_GUIDE.md);
[`CHEATSHEET.md`](https://github.com/dginev/latexml-oxide/blob/main/latexml_contrib/src/script_bindings/CHEATSHEET.md)
sets each call beside its Perl spelling and its compile-time macro.
"#;

/// One registered function, split into the parts we present.
struct Signature {
  /// The handle it is called on (`Document`, `Node`, …); `None` for a free
  /// function.
  receiver: Option<String>,
  name:     String,
  params:   Vec<String>,
  ret:      Option<String>,
}

impl Signature {
  /// Parse one `gen_fn_signatures` line:
  /// `name(_: &mut Document, _: string) -> core::result::Result<…>`.
  fn parse(raw: &str) -> Self {
    let (name, rest) = raw.split_once('(').unwrap_or((raw, ""));
    let (args, ret) = match rest.rsplit_once(") -> ") {
      Some((a, r)) => (a, Some(simplify_type(r))),
      None => (rest.trim_end_matches(')'), None),
    };
    let mut params: Vec<String> = args
      .split(", ")
      .filter(|p| !p.is_empty())
      .map(|p| simplify_type(p.split_once(": ").map_or(p, |(_, t)| t)))
      .collect();
    // A leading `&mut T` parameter is the receiver, not an argument.
    let receiver = params
      .first()
      .and_then(|p| p.strip_prefix("&mut ").map(str::to_string));
    if receiver.is_some() {
      params.remove(0);
    }
    let name = name.to_string();
    if TOKEN_ARG_FNS.contains(&name.as_str()) {
      for p in params.iter_mut().filter(|p| *p == "?") {
        // `\|`: a bare pipe would split the Markdown table cell.
        *p = "string\\|Token".into();
      }
    }
    // A known handle beats the opaque `Dynamic` the registration reports.
    let ret = match RETURN_OVERRIDES.iter().find(|(n, _)| *n == name) {
      Some((_, real)) => Some((*real).to_string()),
      None => ret.filter(|r| r != "()"),
    };
    Self { receiver, name, params, ret }
  }

  fn render(&self) -> String {
    let args = self.params.join(", ");
    match &self.ret {
      Some(r) => format!("{}({args}) -> {r}", self.name),
      None => format!("{}({args})", self.name),
    }
  }
}

/// Rhai reports Rust paths verbatim (`alloc::string::String`,
/// `core::result::Result<…, Box<EvalAltResult>>`). Render them as the Rhai types
/// a binding author actually writes.
fn simplify_type(raw: &str) -> String {
  let t = raw.trim();
  // `Result<T, Box<EvalAltResult>>` is "returns T, or raises" — the raising is a
  // property of every fallible call, so present just T (see the header note).
  if let Some(inner) = t
    .strip_prefix("core::result::Result<")
    .and_then(|r| r.strip_suffix(">"))
    && let Some((ok, _err)) = split_top_level_comma(inner)
  {
    return simplify_type(ok);
  }
  match t {
    "alloc::string::String" => "string".into(),
    "alloc::vec::Vec<rhai::types::dynamic::Dynamic>" => "array".into(),
    "i64" => "int".into(),
    "f64" => "float".into(),
    "rhai::types::dynamic::Dynamic" | "types::dynamic::Dynamic" => "?".into(),
    _ => {
      // A BTreeMap<SmartString, Dynamic> is Rhai's `map`; the rest just lose
      // their module path (`latexml_core::tokens::Tokens` -> `Tokens`).
      if t.starts_with("alloc::collections::btree::map::BTreeMap<") {
        return "map".into();
      }
      match t.strip_prefix("&mut ") {
        Some(inner) => format!("&mut {}", last_path_segment(inner)),
        None => last_path_segment(t).to_string(),
      }
    },
  }
}

fn last_path_segment(t: &str) -> &str { t.rsplit("::").next().unwrap_or(t) }

/// Split `T, E` at the comma that is not inside angle brackets — the Ok/Err
/// boundary of a `Result<…>` whose Ok type may itself be generic.
fn split_top_level_comma(s: &str) -> Option<(&str, &str)> {
  let mut depth = 0usize;
  for (i, c) in s.char_indices() {
    match c {
      '<' => depth += 1,
      '>' => depth = depth.saturating_sub(1),
      ',' if depth == 0 => return Some((&s[..i], &s[i + 1..])),
      _ => {},
    }
  }
  None
}
