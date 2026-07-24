# The `.rhai` binding interface

Every function a runtime (Rhai) binding can call, grouped by the handle it is
called on. **Generated** from the live engine via Rhai's reflections API
(`Engine::gen_fn_signatures`) and checked on every test run, so it always
matches what is actually registered — see `api_doc.rs`.

Overloads share a row — the accepted argument shapes differ, what the call means
does not.

The **documentation** column links the Rust item that implements the call: that
is where its exact semantics live, and linking rather than paraphrasing is what
keeps this reference from drifting out of step with the engine. Where a call has
no single counterpart — a Rhai-only helper, or a shim whose whole behaviour is in
the registration — the column describes it instead. A name registered on two
different handles is two different calls, and is documented separately under
each.

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

## Global functions

Called at the top level of a binding file, or from inside any body.

88 functions, 115 calls.

| call | documentation |
|---|---|
| `AddToCounter(string, int)` | [`add_to_counter`](latexml_core::binding::counter::dialect::add_to_counter) |
| `AssignCatcode(string, int)` | [`assign_catcode`](latexml_core::state::assign_catcode) |
| `AssignMapping(string, string, string)` | [`assign_mapping`](latexml_core::state::assign_mapping) |
| `AssignMeaning(string, string)` | [`assign_meaning`](latexml_core::state::assign_meaning) |
| `AssignValue(string, string)`<br>`AssignValue(string, string, string)` | [`assign_value`](latexml_core::state::assign_value) |
| `Command(string) -> Command` | Start a `std::process::Command` builder. A Rhai-only shim; nothing runs until `output()`. |
| `CounterValue(string) -> int` | [`counter_value`](latexml_core::binding::counter::dialect::counter_value) |
| `DeclareOption(Fn)`<br>`DeclareOption(string, Fn)`<br>`DeclareOption(string, string)` | Declare a class/package option: named with a body, or the bare-closure form for the default handler (Perl `DeclareOption(undef, sub {...})`). |
| `DefAccent(string, string, string)`<br>`DefAccent(string, string, string, bool)` | Define an accent command in both its combining and standalone forms. |
| `DefColumnType(string, Fn)` | [`def_macro`](latexml_core::binding::def::dialect::def_macro) |
| `DefConditional(string, Fn)` | [`def_conditional`](latexml_core::binding::def::dialect::def_conditional) |
| `DefConstructor(string, Fn)`<br>`DefConstructor(string, Fn, map)`<br>`DefConstructor(string, string)`<br>`DefConstructor(string, string, map)` | [`def_constructor`](latexml_core::binding::def::dialect::def_constructor) |
| `DefEnvironment(string, Fn)`<br>`DefEnvironment(string, Fn, map)`<br>`DefEnvironment(string, string)`<br>`DefEnvironment(string, string, map)` | [`def_environment`](latexml_core::binding::def::dialect::def_environment) |
| `DefKeyVal(string, string, string)`<br>`DefKeyVal(string, string, string, string)`<br>`DefKeyVal(string, string, string, string, map)` | Declare one key in a keyval family. |
| `DefLigature(string, string)` | Register a text ligature pattern. |
| `DefMacro(string, Fn)`<br>`DefMacro(string, Fn, map)`<br>`DefMacro(string, string)`<br>`DefMacro(string, string, map)` | [`def_macro`](latexml_core::binding::def::dialect::def_macro) |
| `DefMath(string, string)`<br>`DefMath(string, string, map)` | [`def_math`](latexml_core::binding::def::dialect::def_math) |
| `DefMathLigature(Fn)`<br>`DefMathLigature(string, string, map)` | Register a math ligature, as a pattern or a matcher closure. |
| `DefMathRewrite(map)`<br>`DefMathRewrite(map, Fn)` | As `DefRewrite`, scoped to math. |
| `DefPrimitive(string, Fn)`<br>`DefPrimitive(string, Fn, map)` | [`def_primitive`](latexml_core::binding::def::dialect::def_primitive) |
| `DefRegister(string, int)`<br>`DefRegister(string, string)` | [`def_register`](latexml_core::binding::def::dialect::def_register) |
| `DefRewrite(map)`<br>`DefRewrite(map, Fn)` | Register a document rewrite rule (data form, or a replace closure). |
| `Digest(Tokens) -> Digested` | [`digest`](latexml_core::stomach::digest) |
| `DigestText(string) -> Digested` | [`digest_text`](latexml_core::binding::content::digest_text) |
| `Error(string, string, string)` | Log an `Error:`. Past `MAX_ERRORS` this escalates to `Fatal` and ends the conversion. |
| `ExecuteOptions(array)` | [`execute_options`](latexml_core::binding::content::execute_options) |
| `Expand(Tokens) -> Tokens` | [`do_expand`](latexml_core::gullet::do_expand) |
| `ExpandPartially(Tokens) -> Tokens` | [`do_expand_partially`](latexml_core::gullet::do_expand_partially) |
| `Fatal(string, string, string)` | End the conversion with a `Fatal:`. |
| `GetKeyVal(?, string) -> string` | One value out of a parsed keyval set. |
| `GetKeyVals(string) -> map` | A whole keyval string parsed into a map. |
| `Info(string, string, string)` | Log an `Info:` line. |
| `InputDefinitions(string)`<br>`InputDefinitions(string, map)` | [`input_definitions`](latexml_core::binding::content::input_definitions) |
| `IsDefined(string) -> bool` | [`is_defined_token`](latexml_core::binding::def::dialect::is_defined_token) |
| `LaTeXMLVersion() -> string` | The engine version string. |
| `Let(string, string)` | [`let_i`](latexml_core::state::let_i) |
| `LoadClass(string)`<br>`LoadClass(string, array)` | [`load_class`](latexml_core::binding::content::load_class) |
| `LookupBool(string) -> bool` | [`lookup_bool`](latexml_core::state::lookup_bool) |
| `LookupCatcode(string) -> int` | [`lookup_catcode`](latexml_core::state::lookup_catcode) |
| `LookupDefinition(string) -> Definition?` | Fetch an installed definition so hooks can be pushed onto it; `()` when undefined. |
| `LookupMapping(string, string) -> string` | [`with_mapping`](latexml_core::state::with_mapping) |
| `LookupMeaning(string) -> string` | [`lookup_meaning`](latexml_core::state::lookup_meaning) |
| `LookupNumber(string) -> int` | [`lookup_number`](latexml_core::state::lookup_number) |
| `LookupString(string) -> string` | [`lookup_string`](latexml_core::state::lookup_string) |
| `LookupTokens(string) -> Tokens` | [`lookup_tokens`](latexml_core::state::lookup_tokens) |
| `LookupValue(string) -> ?` | [`lookup_value`](latexml_core::state::lookup_value) |
| `MergeFont(map)` | [`merge_font`](latexml_core::binding::content::merge_font) |
| `NewCounter(string)`<br>`NewCounter(string, string)` | [`new_counter`](latexml_core::binding::counter::dialect::new_counter) |
| `NoteLog(string)` | Write a progress note to the conversion log only. |
| `NoteSTDERR(string)` | Write a progress note to stderr as well as the log. |
| `ParseXML(string) -> array` | [`parse_fragment`](latexml_core::common::xml::parse_fragment) |
| `PassOptions(string, string, array)` | [`pass_options`](latexml_core::binding::content::pass_options) |
| `ProcessOptions()`<br>`ProcessOptions(bool)` | [`process_options`](latexml_core::binding::content::process_options) |
| `ProgressSpindown(string)` | [`note_end`](latexml_core::common::error::note_end) |
| `ProgressSpinup(string)` | [`note_begin`](latexml_core::common::error::note_begin) |
| `ProgressStep(string)` | [`progress_step`](latexml_core::common::error::progress_step) |
| `RawTeX(string)` | [`raw_tex`](latexml_core::stomach::raw_tex) |
| `ReadArg() -> string` | [`read_arg`](latexml_core::gullet::read_arg) |
| `ReadOptional() -> string` | [`read_optional`](latexml_core::gullet::read_optional) |
| `ReadUntil(string) -> string` | [`read_until`](latexml_core::gullet::read_until) |
| `RefCurrentID(string) -> map` | [`ref_current_id`](latexml_core::binding::counter::dialect::ref_current_id) |
| `RefStepCounter(string) -> map` | [`ref_step_counter`](latexml_core::binding::counter::dialect::ref_step_counter) |
| `RefStepID(string) -> map` | [`ref_step_id`](latexml_core::binding::counter::dialect::ref_step_id) |
| `RegisterDocumentNamespace(string, string)` | Bind an OUTPUT-document prefix to a namespace URI. |
| `RegisterNamespace(string, string)` | Bind a CODE prefix to a namespace URI (Perl `Package.pm` `RegisterNamespace`). |
| `RelaxNGSchema(string)` | [`select_relaxng_schema`](latexml_core::binding::content::select_relaxng_schema) |
| `RequirePackage(string)`<br>`RequirePackage(string, array)` | [`require_package`](latexml_core::binding::content::require_package) |
| `RequireResource(string)`<br>`RequireResource(string, map)` | [`require_resource`](latexml_core::binding::content::require_resource) |
| `ResetCounter(string)` | [`reset_counter`](latexml_core::binding::counter::dialect::reset_counter) |
| `Revert(Digested) -> Tokens` | [`Digested::revert`](latexml_core::digested::Digested::revert) |
| `SkipSpaces()` | [`skip_spaces`](latexml_core::gullet::skip_spaces) |
| `StepCounter(string)` | [`step_counter`](latexml_core::binding::counter::dialect::step_counter) |
| `T_CS(string) -> Tokens` | One control-sequence token, wrapped as `Tokens` so it composes with `Digest`/`Expand`. |
| `Tag(string, map)` | [`install_tag`](latexml_core::binding::content::install_tag) |
| `TeX(string)` | [`digest`](latexml_core::stomach::digest) |
| `ToAttribute(Digested) -> string` | [`Digested::to_attribute`](latexml_core::digested::Digested::to_attribute) |
| `ToString(Digested) -> string` | The digested value as plain text (its `Display`). |
| `Today() -> string` | [`today`](latexml_engine::base_utilities::today) |
| `Tokenize(string) -> Tokens` | [`tokenize`](latexml_core::mouth::tokenize) |
| `TokenizeInternal(string) -> Tokens` | [`tokenize_internal`](latexml_core::mouth::tokenize_internal) |
| `UnTeX(Tokens) -> string` | [`Tokens::untex`](latexml_core::tokens::Tokens::untex) |
| `Warn(string, string, string)` | Log a `Warning:` with the given category and object. |
| `XEquals(string, string) -> bool` | [`x_equals`](latexml_core::state::x_equals) |
| `assign_global(string, string)` | `AssignValue` with global scope: the binding survives the enclosing TeX group. |
| `assign_value(string, string)` | `AssignValue` with the default (group-local) scope. |
| `lookup_value(string) -> string` | `LookupValue` coerced to a string; empty when unset. |
| `neutralize_font()` | [`neutralize_font`](latexml_engine::base_utilities::neutralize_font) |
| `whatsit() -> Whatsit` | A handle on the whatsit under construction. Meaningful only inside a digest hook — the handle itself is always returned, but USING it outside one is a clean script error, not a crash. |

## `document` methods

Reached through the `document` handle a constructor body receives as its first argument — Perl's `$document->method`.

24 functions, 33 calls.

| call | documentation |
|---|---|
| `absorb(Digested)` | [`Document::absorb`](latexml_core::document::Document::absorb) |
| `absorbProperty(string)` | [`Document::absorb`](latexml_core::document::Document::absorb) |
| `absorbString(string)` | [`Document::absorb_string`](latexml_core::document::Document::absorb_string) |
| `addClass(Node, string)` | [`Document::add_class`](latexml_core::document::Document::add_class) |
| `appendClone(Node, array)` | [`Document::append_clone`](latexml_core::document::Document::append_clone) |
| `closeElement(string)` | [`Document::close_element`](latexml_core::document::Document::close_element) |
| `closeElementAt(Node)` | [`Document::close_element_at`](latexml_core::document::Document::close_element_at) |
| `findnode(string) -> Node?`<br>`findnode(string, Node) -> Node?` | [`Document::findnode`](latexml_core::document::Document::findnode) |
| `findnodes(string) -> array`<br>`findnodes(string, Node) -> array` | [`Document::findnodes`](latexml_core::document::Document::findnodes) |
| `generateID(Node, string)` | [`Document::generate_id`](latexml_core::document::Document::generate_id) |
| `getElement() -> Node?` | [`Document::get_element`](latexml_core::document::Document::get_element) |
| `getNode() -> Node` | [`Document::get_node`](latexml_core::document::Document::get_node) |
| `insertElement(string) -> Node`<br>`insertElement(string, Digested) -> Node`<br>`insertElement(string, Digested, map) -> Node`<br>`insertElement(string, map) -> Node` | [`Document::insert_element`](latexml_core::document::Document::insert_element) |
| `insertXML(Node)`<br>`insertXML(array)`<br>`insertXML(string)` | [`Document::insert_nodes`](latexml_core::document::Document::insert_nodes) |
| `maybeCloseElement(string)` | [`Document::maybe_close_element`](latexml_core::document::Document::maybe_close_element) |
| `openElement(string)` | [`Document::open_element`](latexml_core::document::Document::open_element) |
| `openElementAt(Node, string) -> Node`<br>`openElementAt(Node, string, map) -> Node` | [`Document::open_element_at`](latexml_core::document::Document::open_element_at) |
| `removeNode(Node)` | [`Document::remove_node`](latexml_core::document::Document::remove_node) |
| `renameNode(Node, string) -> Node`<br>`renameNode(Node, string, bool) -> Node` | [`Document::rename_node`](latexml_core::document::Document::rename_node) |
| `replaceNode(Node, array)` | [`Document::replace_node`](latexml_core::document::Document::replace_node) |
| `setAttribute(string, string)` | [`Document::set_attribute`](latexml_core::document::Document::set_attribute) |
| `setNode(Node)` | [`Document::set_node`](latexml_core::document::Document::set_node) |
| `unwrapNodes(Node)` | [`Document::unwrap_nodes`](latexml_core::document::Document::unwrap_nodes) |
| `wrapNodes(string, array) -> Node?` | [`Document::wrap_nodes`](latexml_core::document::Document::wrap_nodes) |

## `Node` methods

An XML node: one returned by `ParseXML`, or one handed to a rewrite / ligature-matcher body.

15 functions, 15 calls.

| call | documentation |
|---|---|
| `children() -> array` | [`Node::get_child_nodes`](libxml::tree::Node::get_child_nodes) |
| `content() -> string` | [`Node::get_content`](libxml::tree::Node::get_content) |
| `firstChild() -> Node?` | [`Node::get_first_child`](libxml::tree::Node::get_first_child) |
| `getAttribute(string) -> string` | [`get_node_attribute`](latexml_core::common::model::get_node_attribute) |
| `hasAttribute(string) -> bool` | [`get_node_attribute`](latexml_core::common::model::get_node_attribute) |
| `name() -> string` | [`Node::get_name`](libxml::tree::Node::get_name) |
| `nextSibling() -> Node?` | [`Node::get_next_sibling`](libxml::tree::Node::get_next_sibling) |
| `parent() -> Node?` | The parent node ([`get_parent`](libxml::tree::Node::get_parent)), or `()` when there is none. A node from `ParseXML` is a TOP-LEVEL node of its chunk, so it reports NO parent: everything above it is a parse artifact ([`is_parse_artifact`](latexml_core::common::xml::is_parse_artifact)), never markup the script wrote. |
| `prevSibling() -> Node?` | [`Node::get_prev_sibling`](libxml::tree::Node::get_prev_sibling) |
| `qname() -> string` | [`with_node_qname`](latexml_core::common::model::with_node_qname) |
| `removeAttribute(string)` | [`remove_node_attribute`](latexml_core::common::model::remove_node_attribute) |
| `setAttribute(string, string)` | [`Node::set_attribute`](libxml::tree::Node::set_attribute) |
| `setContent(string)` | [`Node::set_content`](libxml::tree::Node::set_content) |
| `toString() -> string` | The node and its subtree serialized back to markup — the inverse of `ParseXML`. |
| `unlink()` | [`Node::unlink`](libxml::tree::Node::unlink) |

## `whatsit()` methods

The whatsit under construction, inside a digest hook.

3 functions, 3 calls.

| call | documentation |
|---|---|
| `argString(int) -> string` | The nth digested argument of the current whatsit, reverted to source text. |
| `propertyString(string) -> string` | One property of the current whatsit, as a string. |
| `setProperty(string, string)` | Set a property on the current whatsit. Only a DIGEST hook may: in a construction hook the whatsit is already read-only, and this is a clean script error. |

## `LookupDefinition()` methods

An already-installed definition, for pushing hooks onto it.

10 functions, 10 calls.

| call | documentation |
|---|---|
| `pushAfterConstruct(Fn)` | Append a hook to that definition's `afterConstruct` list. |
| `pushAfterDigest(Fn)` | Append a hook to that definition's `afterDigest` list. |
| `pushAfterDigestBody(Fn)` | Append a hook to that definition's `afterDigestBody` list. |
| `pushBeforeConstruct(Fn)` | Append a hook to that definition's `beforeConstruct` list. |
| `pushBeforeDigest(Fn)` | Append a hook to that definition's `beforeDigest` list. |
| `unshiftAfterConstruct(Fn)` | Prepend a hook to that definition's `afterConstruct` list. |
| `unshiftAfterDigest(Fn)` | Prepend a hook to that definition's `afterDigest` list. |
| `unshiftAfterDigestBody(Fn)` | Prepend a hook to that definition's `afterDigestBody` list. |
| `unshiftBeforeConstruct(Fn)` | Prepend a hook to that definition's `beforeConstruct` list. |
| `unshiftBeforeDigest(Fn)` | Prepend a hook to that definition's `beforeDigest` list. |

## `Command` methods

The `std::process::Command` mirror, for a binding that shells out.

5 functions, 5 calls.

| call | documentation |
|---|---|
| `arg(string)` | Append one argument to the command. |
| `args(array)` | Append several arguments at once, from an array. |
| `current_dir(string)` | Run the command in this working directory. |
| `env(string, string)` | Set one environment variable for the command. |
| `output() -> ?` | Run the command and return `#{ status, stdout, stderr }`. Refused when `LATEXML_DISABLE_SHELL_ESCAPE` is set (see SAFETY.md). |
