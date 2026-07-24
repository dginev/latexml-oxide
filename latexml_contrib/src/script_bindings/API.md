# The `.rhai` binding interface

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

## Global functions

Called at the top level of a binding file, or from inside any body.

88 functions, 115 calls.

| call | documentation |
|---|---|
| `AddToCounter(string, int)` | Add a number to a counter. ([`add_to_counter`](latexml_core::binding::counter::dialect::add_to_counter)) |
| `AssignCatcode(string, int)` | Set a character's category code. ([`assign_catcode`](latexml_core::state::assign_catcode)) |
| `AssignMapping(string, string, string)` | Bind one key inside a named mapping. ([`assign_mapping`](latexml_core::state::assign_mapping)) |
| `AssignMeaning(string, string)` | Make a control sequence mean a definition, or another token. ([`assign_meaning`](latexml_core::state::assign_meaning)) |
| `AssignValue(string, string)`<br>`AssignValue(string, string, string)` | Bind a key in the value table, with a scope. ([`assign_value`](latexml_core::state::assign_value)) |
| `Command(string) -> Command` | Start a `std::process::Command` builder. A Rhai-only shim; nothing runs until `output()`. |
| `CounterValue(string) -> int` | The current value of a counter. ([`counter_value`](latexml_core::binding::counter::dialect::counter_value)) |
| `DeclareOption(Fn)`<br>`DeclareOption(string, Fn)`<br>`DeclareOption(string, string)` | Declare a class/package option: named with a body, or the bare-closure form for the default handler (Perl `DeclareOption(undef, sub {...})`). |
| `DefAccent(string, string, string)`<br>`DefAccent(string, string, string, bool)` | Define an accent command in both its combining and standalone forms. |
| `DefColumnType(string, Fn)` | Define a tabular column type, as its rewrite expansion. ([`def_macro`](latexml_core::binding::def::dialect::def_macro)) |
| `DefConditional(string, Fn)` | Define a conditional control sequence. ([`def_conditional`](latexml_core::binding::def::dialect::def_conditional)) |
| `DefConstructor(string, Fn)`<br>`DefConstructor(string, Fn, map)`<br>`DefConstructor(string, string)`<br>`DefConstructor(string, string, map)` | Define a control sequence that constructs XML. ([`def_constructor`](latexml_core::binding::def::dialect::def_constructor)) |
| `DefEnvironment(string, Fn)`<br>`DefEnvironment(string, Fn, map)`<br>`DefEnvironment(string, string)`<br>`DefEnvironment(string, string, map)` | Define an environment that constructs XML. ([`def_environment`](latexml_core::binding::def::dialect::def_environment)) |
| `DefKeyVal(string, string, string)`<br>`DefKeyVal(string, string, string, string)`<br>`DefKeyVal(string, string, string, string, map)` | Declare one key of a keyval family. ([`define`](latexml_core::keyval::define)) |
| `DefLigature(string, string)` | Register a text ligature pattern. |
| `DefMacro(string, Fn)`<br>`DefMacro(string, Fn, map)`<br>`DefMacro(string, string)`<br>`DefMacro(string, string, map)` | Define a macro's expansion. ([`def_macro`](latexml_core::binding::def::dialect::def_macro)) |
| `DefMath(string, string)`<br>`DefMath(string, string, map)` | Define a mathematical symbol or function. ([`def_math`](latexml_core::binding::def::dialect::def_math)) |
| `DefMathLigature(Fn)`<br>`DefMathLigature(string, string, map)` | Register a math ligature, as a pattern or a matcher closure. |
| `DefMathRewrite(map)`<br>`DefMathRewrite(map, Fn)` | As `DefRewrite`, scoped to math. |
| `DefPrimitive(string, Fn)`<br>`DefPrimitive(string, Fn, map)` | Define a primitive: it runs at digestion, after expansion. ([`def_primitive`](latexml_core::binding::def::dialect::def_primitive)) |
| `DefRegister(string, int)`<br>`DefRegister(string, string)` | Define a register with an initial value. ([`def_register`](latexml_core::binding::def::dialect::def_register)) |
| `DefRewrite(map)`<br>`DefRewrite(map, Fn)` | Register a document rewrite rule (data form, or a replace closure). |
| `Digest(Tokens) -> Digested` | Digest tokens into boxes, independent of the current gullet. ([`digest`](latexml_core::stomach::digest)) |
| `DigestText(string) -> Digested` | Digest tokens in text mode, whatever mode the caller is in. ([`digest_text`](latexml_core::binding::content::digest_text)) |
| `Error(string, string, string)` | Log an `Error:`. Past `MAX_ERRORS` this escalates to `Fatal` and ends the conversion. |
| `ExecuteOptions(array)` | Run the handlers for a list of class/package options. ([`execute_options`](latexml_core::binding::content::execute_options)) |
| `Expand(Tokens) -> Tokens` | Fully expand tokens, without digesting them. ([`do_expand`](latexml_core::gullet::do_expand)) |
| `ExpandPartially(Tokens) -> Tokens` | Expand tokens only up to the first unexpandable one. ([`do_expand_partially`](latexml_core::gullet::do_expand_partially)) |
| `Fatal(string, string, string)` | End the conversion with a `Fatal:`. |
| `GetKeyVal(?, string) -> string` | One value out of a parsed keyval set. |
| `GetKeyVals(string) -> map` | Parse a whole keyval string into a map. ([`split_keyval_source`](latexml_core::keyval::split_keyval_source)) |
| `Info(string, string, string)` | Log an `Info:` line. |
| `InputDefinitions(string)`<br>`InputDefinitions(string, map)` | Find and load the definitions for a package or class. ([`input_definitions`](latexml_core::binding::content::input_definitions)) |
| `IsDefined(string) -> bool` | Whether a control sequence is defined, and not `\let` to `\relax`. ([`is_defined_token`](latexml_core::binding::def::dialect::is_defined_token)) |
| `LaTeXMLVersion() -> string` | The engine version string — the `LATEXML_VERSION` value ([`lookup_string`](latexml_core::state::lookup_string)). |
| `Let(string, string)` | TeX's `\let`: copy one token's meaning onto another. ([`let_i`](latexml_core::state::let_i)) |
| `LoadClass(string)`<br>`LoadClass(string, array)` | Load a document class, falling back to a prefix match then OmniBus. ([`load_class`](latexml_core::binding::content::load_class)) |
| `LookupBool(string) -> bool` | Read a value as a boolean. ([`lookup_bool`](latexml_core::state::lookup_bool)) |
| `LookupCatcode(string) -> int` | The category code currently in force for a character. ([`lookup_catcode`](latexml_core::state::lookup_catcode)) |
| `LookupDefinition(string) -> Definition?` | Fetch an installed definition so hooks can be pushed onto it; `()` when undefined. |
| `LookupMapping(string, string) -> string` | Read one key out of a named mapping. ([`with_mapping`](latexml_core::state::with_mapping)) |
| `LookupMeaning(string) -> string` | What a token means right now: its definition, or itself. ([`lookup_meaning`](latexml_core::state::lookup_meaning)) |
| `LookupNumber(string) -> int` | Read a value as a number. ([`lookup_number`](latexml_core::state::lookup_number)) |
| `LookupString(string) -> string` | Read a value as a string; empty when unset. ([`lookup_string`](latexml_core::state::lookup_string)) |
| `LookupTokens(string) -> Tokens` | Read a value as tokens. ([`lookup_tokens`](latexml_core::state::lookup_tokens)) |
| `LookupValue(string) -> ?` | Read a value, whatever type it was stored as. ([`lookup_value`](latexml_core::state::lookup_value)) |
| `MergeFont(map)` | Merge font attributes into the current font, group-locally. ([`merge_font`](latexml_core::binding::content::merge_font)) |
| `NewCounter(string)`<br>`NewCounter(string, string)` | Declare a new counter. ([`new_counter`](latexml_core::binding::counter::dialect::new_counter)) |
| `NoteLog(string)` | Write a progress note to the conversion log only. |
| `NoteSTDERR(string)` | Write a progress note to stderr as well as the log. |
| `ParseXML(string) -> array` | Parse a markup chunk into nodes; a bare fragment is fine. ([`parse_fragment`](latexml_core::common::xml::parse_fragment)) |
| `PassOptions(string, string, array)` | Forward options to a package or class not yet loaded. ([`pass_options`](latexml_core::binding::content::pass_options)) |
| `ProcessOptions()`<br>`ProcessOptions(bool)` | Execute the options declared so far, in order or as given. ([`process_options`](latexml_core::binding::content::process_options)) |
| `ProgressSpindown(string)` | Close a named progress stage in the log. ([`note_end`](latexml_core::common::error::note_end)) |
| `ProgressSpinup(string)` | Open a named progress stage in the log. ([`note_begin`](latexml_core::common::error::note_begin)) |
| `ProgressStep(string)` | Advance the progress indicator; a no-op in this port. ([`progress_step`](latexml_core::common::error::progress_step)) |
| `RawTeX(string)` | Process a chunk of literal TeX as definitions. ([`raw_tex`](latexml_core::stomach::raw_tex)) |
| `ReadArg() -> string` | Read one TeX argument: a token, or a braced group. ([`read_arg`](latexml_core::gullet::read_arg)) |
| `ReadOptional() -> string` | Read a LaTeX optional `[…]` argument, or a default. ([`read_optional`](latexml_core::gullet::read_optional)) |
| `ReadUntil(string) -> string` | Read a balanced token sequence up to a delimiter. ([`read_until`](latexml_core::gullet::read_until)) |
| `RefCurrentID(string) -> map` | Reuse the last id without stepping, when its box was pruned. ([`ref_current_id`](latexml_core::binding::counter::dialect::ref_current_id)) |
| `RefStepCounter(string) -> map` | Step a counter and return its `refnum` and `id`. ([`ref_step_counter`](latexml_core::binding::counter::dialect::ref_step_counter)) |
| `RefStepID(string) -> map` | Step only the uncounter, for an UN-numbered unit; returns the id. ([`ref_step_id`](latexml_core::binding::counter::dialect::ref_step_id)) |
| `RegisterDocumentNamespace(string, string)` | Bind an OUTPUT-document prefix to a namespace URI. ([`register_document_namespace`](latexml_core::common::model::register_document_namespace)) |
| `RegisterNamespace(string, string)` | Bind a CODE prefix to a namespace URI. ([`register_namespace`](latexml_core::common::model::register_namespace)) |
| `RelaxNGSchema(string)` | Select the RelaxNG schema defining the output language. ([`select_relaxng_schema`](latexml_core::binding::content::select_relaxng_schema)) |
| `RequirePackage(string)`<br>`RequirePackage(string, array)` | Load a package. ([`require_package`](latexml_core::binding::content::require_package)) |
| `RequireResource(string)`<br>`RequireResource(string, map)` | Attach a CSS or JavaScript resource to the document. ([`require_resource`](latexml_core::binding::content::require_resource)) |
| `ResetCounter(string)` | Reset a counter to zero. ([`reset_counter`](latexml_core::binding::counter::dialect::reset_counter)) |
| `Revert(Digested) -> Tokens` | A digested value back to the source tokens that made it. ([`Digested::revert`](latexml_core::digested::Digested::revert)) |
| `SkipSpaces()` | Discard any run of spaces at the head of the input. ([`skip_spaces`](latexml_core::gullet::skip_spaces)) |
| `StepCounter(string)` | Step a counter; usually you want `RefStepCounter` instead. ([`step_counter`](latexml_core::binding::counter::dialect::step_counter)) |
| `T_CS(string) -> Tokens` | One control-sequence token, wrapped as `Tokens` so it composes with `Digest`/`Expand`. |
| `Tag(string, map)` | Declare document-model properties for one element tag. ([`install_tag`](latexml_core::binding::content::install_tag)) |
| `TeX(string)` | Tokenize a TeX source string (style catcodes) and digest it. ([`digest`](latexml_core::stomach::digest)) |
| `ToAttribute(Digested) -> string` | The digested value as a string fit for an XML attribute. ([`Digested::to_attribute`](latexml_core::digested::Digested::to_attribute)) |
| `ToString(Digested) -> string` | The digested value as plain text (its `Display`). |
| `Today() -> string` | Today's date, as `\today` renders it. ([`today`](latexml_engine::base_utilities::today)) |
| `Tokenize(string) -> Tokens` | Tokenize a string under the standard catcode table. ([`tokenize`](latexml_core::mouth::tokenize)) |
| `TokenizeInternal(string) -> Tokens` | Tokenize a string under the style-file table, where `@` is a letter. ([`tokenize_internal`](latexml_core::mouth::tokenize_internal)) |
| `UnTeX(Tokens) -> string` | Tokens back to the TeX source that could have produced them. ([`Tokens::untex`](latexml_core::tokens::Tokens::untex)) |
| `Warn(string, string, string)` | Log a `Warning:` with the given category and object. |
| `XEquals(string, string) -> bool` | Whether two control sequences have the same meaning. ([`x_equals`](latexml_core::state::x_equals)) |
| `assign_global(string, string)` | [`assign_value`](latexml_core::state::assign_value) with [`Scope::Global`](latexml_core::state::Scope::Global): the binding survives the enclosing TeX group. |
| `assign_value(string, string)` | [`assign_value`](latexml_core::state::assign_value) with [`Scope::Local`](latexml_core::state::Scope::Local), TeX's default: the binding expires with the enclosing group. |
| `lookup_value(string) -> string` | [`lookup_value`](latexml_core::state::lookup_value) coerced to a string; empty when unset, so an unbound key and an empty one are indistinguishable here. |
| `neutralize_font()` | Reset the text and math fonts to their defaults, group-locally. ([`neutralize_font`](latexml_engine::base_utilities::neutralize_font)) |
| `whatsit() -> Whatsit` | A handle on the whatsit under construction. Meaningful only inside a digest hook — the handle itself is always returned, but USING it outside one is a clean script error, not a crash. |

## `document` methods

Reached through the `document` handle a constructor body receives as its first argument — Perl's `$document->method`.

24 functions, 33 calls.

| call | documentation |
|---|---|
| `absorb(Digested)` | Absorb a digested value at the current insertion point. ([`Document::absorb`](latexml_core::document::Document::absorb)) |
| `absorbProperty(string)` | Absorb one of the whatsit's properties, by name. ([`Document::absorb`](latexml_core::document::Document::absorb)) |
| `absorbString(string)` | Absorb a plain string at the current insertion point. ([`Document::absorb_string`](latexml_core::document::Document::absorb_string)) |
| `addClass(Node, string)` | Add CSS classes to a node, keeping those it already has. ([`Document::add_class`](latexml_core::document::Document::add_class)) |
| `appendClone(Node, array)` | Append COPIES of nodes, with fresh ids. ([`Document::append_clone`](latexml_core::document::Document::append_clone)) |
| `closeElement(string)` | Close the deepest open element of that name. ([`Document::close_element`](latexml_core::document::Document::close_element)) |
| `closeElementAt(Node)` | Close an element that was opened with `openElementAt`. ([`Document::close_element_at`](latexml_core::document::Document::close_element_at)) |
| `findnode(string) -> Node?`<br>`findnode(string, Node) -> Node?` | The first node matching an XPath. ([`Document::findnode`](latexml_core::document::Document::findnode)) |
| `findnodes(string) -> array`<br>`findnodes(string, Node) -> array` | Every node matching an XPath. ([`Document::findnodes`](latexml_core::document::Document::findnodes)) |
| `generateID(Node, string)` | Give a node an `xml:id`, if it has none. ([`Document::generate_id`](latexml_core::document::Document::generate_id)) |
| `getElement() -> Node?` | The element at, or containing, the insertion point. ([`Document::get_element`](latexml_core::document::Document::get_element)) |
| `getNode() -> Node` | The current insertion point. ([`Document::get_node`](latexml_core::document::Document::get_node)) |
| `insertElement(string) -> Node`<br>`insertElement(string, Digested) -> Node`<br>`insertElement(string, Digested, map) -> Node`<br>`insertElement(string, map) -> Node` | Open, absorb and close in one step; returns the new element. ([`Document::insert_element`](latexml_core::document::Document::insert_element)) |
| `insertXML(Node)`<br>`insertXML(array)`<br>`insertXML(string)` | Splice ALREADY-PARSED nodes in at the insertion point. ([`Document::insert_nodes`](latexml_core::document::Document::insert_nodes)) |
| `maybeCloseElement(string)` | Close an element if it is open and closeable; otherwise do nothing. ([`Document::maybe_close_element`](latexml_core::document::Document::maybe_close_element)) |
| `openElement(string)` | Open an element and make it the insertion point. ([`Document::open_element`](latexml_core::document::Document::open_element)) |
| `openElementAt(Node, string) -> Node`<br>`openElementAt(Node, string, map) -> Node` | Open an element at a given node rather than the insertion point. ([`Document::open_element_at`](latexml_core::document::Document::open_element_at)) |
| `removeNode(Node)` | Remove a node, releasing the ids under it. ([`Document::remove_node`](latexml_core::document::Document::remove_node)) |
| `renameNode(Node, string) -> Node`<br>`renameNode(Node, string, bool) -> Node` | Rename an element, rebuilding it through the model. ([`Document::rename_node`](latexml_core::document::Document::rename_node)) |
| `replaceNode(Node, array)` | Replace a node by other nodes. ([`Document::replace_node`](latexml_core::document::Document::replace_node)) |
| `setAttribute(string, string)` | Set an attribute on the current node, if the model allows it. ([`Document::set_attribute`](latexml_core::document::Document::set_attribute)) |
| `setNode(Node)` | Move the insertion point to a node. ([`Document::set_node`](latexml_core::document::Document::set_node)) |
| `unwrapNodes(Node)` | Replace a node by its own children. ([`Document::unwrap_nodes`](latexml_core::document::Document::unwrap_nodes)) |
| `wrapNodes(string, array) -> Node?` | Wrap a run of sibling nodes in a new element. ([`Document::wrap_nodes`](latexml_core::document::Document::wrap_nodes)) |

## `Node` methods

An XML node: one returned by `ParseXML`, or one handed to a rewrite / ligature-matcher body.

15 functions, 15 calls.

| call | documentation |
|---|---|
| `children() -> array` | The node's child nodes. ([`Node::get_child_nodes`](libxml::tree::Node::get_child_nodes)) |
| `content() -> string` | The node's text content. ([`Node::get_content`](libxml::tree::Node::get_content)) |
| `firstChild() -> Node?` | The node's first child, or `()`. ([`Node::get_first_child`](libxml::tree::Node::get_first_child)) |
| `getAttribute(string) -> string` | An attribute's value, namespace-aware so `xml:id` is found. ([`get_node_attribute`](latexml_core::common::model::get_node_attribute)) |
| `hasAttribute(string) -> bool` | Whether the node carries that attribute. ([`get_node_attribute`](latexml_core::common::model::get_node_attribute)) |
| `name() -> string` | The node's local name. ([`Node::get_name`](libxml::tree::Node::get_name)) |
| `nextSibling() -> Node?` | The node's next sibling, or `()`. ([`Node::get_next_sibling`](libxml::tree::Node::get_next_sibling)) |
| `parent() -> Node?` | The parent node ([`get_parent`](libxml::tree::Node::get_parent)), or `()` when there is none. A node from `ParseXML` is a TOP-LEVEL node of its chunk, so it reports NO parent: everything above it is a parse artifact ([`is_parse_artifact`](latexml_core::common::xml::is_parse_artifact)), never markup the script wrote. |
| `prevSibling() -> Node?` | The node's previous sibling, or `()`. ([`Node::get_prev_sibling`](libxml::tree::Node::get_prev_sibling)) |
| `qname() -> string` | The node's qualified name (`ltx:section`, `#PCDATA`, …). ([`with_node_qname`](latexml_core::common::model::with_node_qname)) |
| `removeAttribute(string)` | Remove a possibly-prefixed attribute. ([`remove_node_attribute`](latexml_core::common::model::remove_node_attribute)) |
| `setAttribute(string, string)` | Set an attribute on this node directly. ([`Node::set_attribute`](libxml::tree::Node::set_attribute)) |
| `setContent(string)` | Replace the node's text content. ([`Node::set_content`](libxml::tree::Node::set_content)) |
| `toString() -> string` | The node and its subtree serialized back to markup — the inverse of `ParseXML`. |
| `unlink()` | Detach the node from its tree. ([`Node::unlink`](libxml::tree::Node::unlink)) |

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
