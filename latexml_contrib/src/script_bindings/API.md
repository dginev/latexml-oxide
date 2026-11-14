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

179 functions, 279 calls.

| call | documentation |
|---|---|
| `!(?) -> bool` | Perl's truthiness, for `!!v` (`if ($v)`) and `!v` (`unless ($v)`): `()`, `""`, `"0"`, `0` and `0.0` are false, a bool is itself, every Token, Tokens (even empty), box, whatsit, array and map is true (Perl's objects overload no bool). |
| `!=((), string) -> bool`<br>`!=(Digested, string) -> bool`<br>`!=(Token, string) -> bool`<br>`!=(Tokens, string) -> bool`<br>`!=(string, ()) -> bool`<br>`!=(string, Digested) -> bool`<br>`!=(string, Token) -> bool`<br>`!=(string, Tokens) -> bool` | **Deprecated** (Warns once per binding load): a Tokens, Token or box against a string compares its string form (use `ToString(v) != s`); `()` against a string, either way round, is `""`, as Perl's `undef ne ''` (use `defined(v)` or `!!v`). |
| `==((), string) -> bool`<br>`==(Digested, string) -> bool`<br>`==(Token, string) -> bool`<br>`==(Tokens, string) -> bool`<br>`==(string, ()) -> bool`<br>`==(string, Digested) -> bool`<br>`==(string, Token) -> bool`<br>`==(string, Tokens) -> bool` | **Deprecated** (Warns once per binding load): a Tokens, Token or box against a string compares its string form (use `ToString(v) == s`); `()` against a string, either way round, is `""`, as Perl's `undef eq ''` (use `defined(v)` or `!v`). |
| `AddToCounter(string, int)` | Add a number to a counter. ([`add_to_counter`](latexml_core::binding::counter::dialect::add_to_counter)) |
| `AddToMacro(string\|Token, string\|Token)` | Append tokens (or TeX source) to a macro's expansion, keeping its lock, as Perl's `AddToMacro`; a non-expandable `cs` Warns and is ignored, an undefined one is defined with the tokens. ([`add_to_macro`](latexml_core::binding::content::add_to_macro)) |
| `AppendSearchPath(string)` | Append a directory to the input search path (searched after existing paths). ([`add_search_path`](latexml_core::state::add_search_path)) |
| `AssignBool(string, ?)`<br>`AssignBool(string, ?, string)` | **Deprecated** (Warns once per binding load): use `AssignValue(key, value[, scope])`. Bind a key to a value, as `AssignValue`. ([`assign_value`](latexml_core::state::assign_value)) |
| `AssignCatcode(string, int)`<br>`AssignCatcode(string, int, string)` | Set a character's category code, with an optional scope. ([`assign_catcode`](latexml_core::state::assign_catcode)) |
| `AssignDelcode(string, int)`<br>`AssignDelcode(string, int, string)` | Set a character's delimiter code, with an optional scope. ([`assign_delcode`](latexml_core::state::assign_delcode)) |
| `AssignFloat(string, float)`<br>`AssignFloat(string, float, string)` | Bind a key to a fractional (float) value, with a scope. ([`assign_value`](latexml_core::state::assign_value)) |
| `AssignLCcode(string, int)`<br>`AssignLCcode(string, int, string)` | Set a character's lowercase code, with an optional scope. ([`assign_lccode`](latexml_core::state::assign_lccode)) |
| `AssignMapping(string, string, ?)` | Bind one key inside a named mapping to any value, as `AssignValue`; `()` removes the key. ([`assign_mapping`](latexml_core::state::assign_mapping)) |
| `AssignMathcode(string, int)`<br>`AssignMathcode(string, int, string)` | Set a character's math code, with an optional scope. ([`assign_mathcode`](latexml_core::state::assign_mathcode)) |
| `AssignMeaning(string\|Token, string\|Token)` | **Deprecated** (Warns once per binding load): use `Let(cs, other[, scope])`. Give a token (CS name or `Token`) the current meaning of another, as `Let`. ([`let_definition`](latexml_core::state::let_definition)) |
| `AssignNumber(string, ?)`<br>`AssignNumber(string, ?, string)` | **Deprecated** (Warns once per binding load): use `AssignValue(key, value[, scope])`. Bind a key to a value, as `AssignValue`. ([`assign_value`](latexml_core::state::assign_value)) |
| `AssignSFcode(string, int)`<br>`AssignSFcode(string, int, string)` | Set a character's space-factor code, with an optional scope. ([`assign_sfcode`](latexml_core::state::assign_sfcode)) |
| `AssignString(string, ?)`<br>`AssignString(string, ?, string)` | **Deprecated** (Warns once per binding load): use `AssignValue(key, value[, scope])`. Bind a key to a value, as `AssignValue`. ([`assign_value`](latexml_core::state::assign_value)) |
| `AssignUCcode(string, int)`<br>`AssignUCcode(string, int, string)` | Set a character's uppercase code, with an optional scope. ([`assign_uccode`](latexml_core::state::assign_uccode)) |
| `AssignValue(string, ?)`<br>`AssignValue(string, ?, string)` | Bind a key in the value table to any value (string, number, bool, `Token`, `Tokens`, handle), with a scope; `()` stores undef. ([`assign_value`](latexml_core::state::assign_value)) |
| `AtBeginDocument(Fn)`<br>`AtBeginDocument(Token)`<br>`AtBeginDocument(Tokens)`<br>`AtBeginDocument(string)` | Queue TeX source, a `Tokens`/`Token` value, or a closure (installed as a macro) to run at `\begin{document}`. ([`push_value`](latexml_core::state::push_value)) |
| `AtEndDocument(Fn)`<br>`AtEndDocument(Token)`<br>`AtEndDocument(Tokens)`<br>`AtEndDocument(string)` | Queue TeX source, a `Tokens`/`Token` value, or a closure (installed as a macro) to run at `\end{document}`. ([`push_value`](latexml_core::state::push_value)) |
| `BeginItemize(string, string) -> map`<br>`BeginItemize(string, string, map) -> map` | Open a list level: the `properties` map numbering its items (Perl `beginItemize`). ([`begin_itemize`](latexml_core::binding::counter::dialect::begin_itemize)) |
| `CC_ACTIVE() -> int` | The catcode constant `CC_ACTIVE` (13), for `Token(string, catcode)`. |
| `CC_ALIGN() -> int` | The catcode constant `CC_ALIGN` (4), for `Token(string, catcode)`. |
| `CC_ARG() -> int` | The catcode constant `CC_ARG` (18), for `Token(string, catcode)`. |
| `CC_BEGIN() -> int` | The catcode constant `CC_BEGIN` (1), for `Token(string, catcode)`. |
| `CC_COMMENT() -> int` | The catcode constant `CC_COMMENT` (14), for `Token(string, catcode)`. |
| `CC_CS() -> int` | The catcode constant `CC_CS` (16), for `Token(string, catcode)`. |
| `CC_END() -> int` | The catcode constant `CC_END` (2), for `Token(string, catcode)`. |
| `CC_EOL() -> int` | The catcode constant `CC_EOL` (5), for `Token(string, catcode)`. |
| `CC_ESCAPE() -> int` | The catcode constant `CC_ESCAPE` (0), for `Token(string, catcode)`. |
| `CC_IGNORE() -> int` | The catcode constant `CC_IGNORE` (9), for `Token(string, catcode)`. |
| `CC_INVALID() -> int` | The catcode constant `CC_INVALID` (15), for `Token(string, catcode)`. |
| `CC_LETTER() -> int` | The catcode constant `CC_LETTER` (11), for `Token(string, catcode)`. |
| `CC_MARKER() -> int` | The catcode constant `CC_MARKER` (17), for `Token(string, catcode)`. |
| `CC_MATH() -> int` | The catcode constant `CC_MATH` (3), for `Token(string, catcode)`. |
| `CC_OTHER() -> int` | The catcode constant `CC_OTHER` (12), for `Token(string, catcode)`. |
| `CC_PARAM() -> int` | The catcode constant `CC_PARAM` (6), for `Token(string, catcode)`. |
| `CC_SPACE() -> int` | The catcode constant `CC_SPACE` (10), for `Token(string, catcode)`. |
| `CC_SUB() -> int` | The catcode constant `CC_SUB` (8), for `Token(string, catcode)`. |
| `CC_SUPER() -> int` | The catcode constant `CC_SUPER` (7), for `Token(string, catcode)`. |
| `CleanID(?) -> string` | Clean a string into a valid `xml:id`. ([`clean_id`](latexml_core::common::cleaners::clean_id)) |
| `CleanLabel(?) -> string`<br>`CleanLabel(?, string) -> string` | A label's key: `LABEL:` (or the given prefix) plus the label, spaces to `_`. ([`clean_label`](latexml_core::common::cleaners::clean_label)) |
| `Command(string) -> Command` | Start a `std::process::Command` builder. A Rhai-only shim; nothing runs until `output()`. |
| `CounterValue(string) -> int` | The current value of a counter. ([`counter_value`](latexml_core::binding::counter::dialect::counter_value)) |
| `DeclareFontMap(string, array)`<br>`DeclareFontMap(string, array, map)` | Declare an encoding's font map: an array of 1-char strings or `()`, with Perl's `family` and `*_mathstyle` options. |
| `DeclareFontMapMultichar(string, map)`<br>`DeclareFontMapMultichar(string, string, map)` | Declare the multi-character glyphs of a font map (optionally a family's): position (decimal string) to its text. |
| `DeclareOption(Fn)`<br>`DeclareOption(string, Fn)`<br>`DeclareOption(string, string)` | Declare a class/package option: named with a body, or the bare-closure form for the default handler (Perl `DeclareOption(undef, sub {...})`). |
| `DefAccent(string, string, string)`<br>`DefAccent(string, string, string, bool)`<br>`DefAccent(string, string, string, map)` | Define an accent command in both its combining and standalone forms; Perl's options map `#{below: true}`. **Deprecated:** its older form (Warns once per binding load); use `DefAccent(accent, combining, standalone, #{below: true})`. |
| `DefColumnType(string, Fn)` | Define a tabular column type, as its rewrite expansion. ([`def_macro`](latexml_core::binding::def::dialect::def_macro)) |
| `DefConditional(string)`<br>`DefConditional(string, (), map)`<br>`DefConditional(string, Fn)`<br>`DefConditional(string, Fn, map)` | Define a conditional control sequence. ([`def_conditional`](latexml_core::binding::def::dialect::def_conditional)) |
| `DefConstructor(string, Fn)`<br>`DefConstructor(string, Fn, map)`<br>`DefConstructor(string, string)`<br>`DefConstructor(string, string, map)` | Define a control sequence that constructs XML. ([`def_constructor`](latexml_core::binding::def::dialect::def_constructor)) |
| `DefEnvironment(string, Fn)`<br>`DefEnvironment(string, Fn, map)`<br>`DefEnvironment(string, string)`<br>`DefEnvironment(string, string, map)` | Define an environment that constructs XML. ([`def_environment`](latexml_core::binding::def::dialect::def_environment)) |
| `DefKeyVal(string, string, string)`<br>`DefKeyVal(string, string, string, string)`<br>`DefKeyVal(string, string, string, string, map)` | Declare one key of a keyval family. ([`define`](latexml_core::keyval::define)) |
| `DefLigature(string, string)`<br>`DefLigature(string, string, map)` | Register a text ligature pattern; `#{fontTest: |font| …}` limits it to the fonts the closure accepts. |
| `DefMacro(string, Fn)`<br>`DefMacro(string, Fn, map)`<br>`DefMacro(string, string)`<br>`DefMacro(string, string, map)` | Define a macro's expansion; a closure body is Perl's `|gullet, args|`, each argument Tokens (or a `KeyVals`). A Perl-form closure declares its handle and every argument, `|gullet, x|`; fewer parameters is the old form, with a Warning naming the fix. ([`def_macro`](latexml_core::binding::def::dialect::def_macro)) |
| `DefMath(string, string)`<br>`DefMath(string, string, map)` | Define a mathematical symbol or function. ([`def_math`](latexml_core::binding::def::dialect::def_math)) |
| `DefMathLigature(Fn)`<br>`DefMathLigature(string, string, map)` | Register a math ligature, as a pattern or a matcher closure. |
| `DefMathRewrite(map)`<br>`DefMathRewrite(map, Fn)` | As `DefRewrite`, scoped to math. |
| `DefParameterType(string, Fn)`<br>`DefParameterType(string, Fn, map)` | Define a parameter type: the reader `|gullet, extra…|` returns its value (Tokens, a Token, `()` for none), with Perl's options `optional`, `novalue`, `semiverbatim`, `undigested`, `reversion`, `beforeDigest`, `afterDigest`, and `nargs` accepted unread, as Perl's. ([`Parameter`](latexml_core::parameter::Parameter)) |
| `DefPrimitive(string, Fn)`<br>`DefPrimitive(string, Fn, map)` | Define a primitive: it runs at digestion, after expansion; a closure body is Perl's `|stomach, args|`. A Perl-form closure declares its handle and every argument, `|stomach, x|`; fewer parameters is the old form, with a Warning naming the fix. ([`def_primitive`](latexml_core::binding::def::dialect::def_primitive)) |
| `DefRegister(string, int)`<br>`DefRegister(string, string)` | Define a register with an initial value. ([`def_register`](latexml_core::binding::def::dialect::def_register)) |
| `DefRewrite(map)`<br>`DefRewrite(map, Fn)` | Register a document rewrite rule (data form, or a replace closure). |
| `Digest(Token) -> Digested`<br>`Digest(Tokens) -> Digested`<br>`Digest(string) -> Digested` | Digest TeX source (a string, TokenizeInternal'd), a `Token` or `Tokens`, returning the digested handle. ([`digest`](latexml_core::stomach::digest)) |
| `DigestIf(string\|Token) -> ?` | Digest a control sequence if it is defined; `()` otherwise. ([`digest_if`](latexml_core::binding::content::digest_if)) |
| `DigestText(Token) -> Digested`<br>`DigestText(Tokens) -> Digested`<br>`DigestText(string) -> Digested` | Digest TeX source, a `Token` or `Tokens` in restricted horizontal mode, returning the digested handle. ([`digest_text`](latexml_core::binding::content::digest_text)) |
| `EndSemiverbatim()` | Close a `StartSemiverbatim` region, restoring the saved catcodes. ([`end_semiverbatim`](latexml_core::state::end_semiverbatim)) |
| `Equals(?, ?) -> bool` | Perl's `Equals`: `()` equals only `()`; tokens compare their meanings (themselves when neither has one), `Tokens` token by token, other values by their string. |
| `Error(string, string, string)`<br>`Error(string, string, string, ?)` | Log an `Error:`, with Perl's trailing details (a string or an array). Past `MAX_ERRORS` this escalates to `Fatal` and ends the conversion. |
| `ExecuteOptions(array)` | Run the handlers for a list of class/package options. ([`execute_options`](latexml_core::binding::content::execute_options)) |
| `Expand(Token) -> Tokens`<br>`Expand(Tokens) -> Tokens` | Fully expand tokens (a `Tokens`, or one `Token`), without digesting them. ([`do_expand`](latexml_core::gullet::do_expand)) |
| `ExpandPartially(Token) -> Tokens`<br>`ExpandPartially(Tokens) -> Tokens` | Expand tokens only up to the first unexpandable one. ([`do_expand_partially`](latexml_core::gullet::do_expand_partially)) |
| `Explode(string) -> Tokens` | A string's characters as `Tokens`, each OTHER (a space a space token). |
| `ExplodeText(string) -> Tokens` | A string's characters as `Tokens`, letters LETTER and the rest OTHER. |
| `Fatal(string, string, string)`<br>`Fatal(string, string, string, ?)` | End the conversion with `Fatal:<cat>:<obj>`, plus optional details. |
| `FindFile(string) -> ?`<br>`FindFile(string, map) -> ?` | The path of a file on the search paths (Perl's `type`/`notex`/`noltxml`/`searchpaths_only`), or `()`. ([`find_file`](latexml_core::binding::content::find_file)) |
| `GetKeyVal(?, string) -> ?` | One value of a keyval set, as Perl's: Tokens as read, a box once digested, `()` when missing; a repeated key's last value. Test it as Perl's `if ($v)` with `!!v`, or `defined(v)`. |
| `GetKeyVals(?) -> map` | A whole keyval set as a map of `GetKeyVal`'s values, a repeated key's as an array. Accepts a `KeyVals` argument (read, or digested in a constructor) or its `k=v,…` source string. |
| `Info(string, string, string)`<br>`Info(string, string, string, ?)` | Log an `Info:` line, plus optional details. |
| `Input(string)` | Input a TeX file (`\input`'s core). ([`input`](latexml_core::binding::content::input)) |
| `InputDefinitions(string)`<br>`InputDefinitions(string, map)` | Find and load the definitions for a package or class. ([`input_definitions`](latexml_core::binding::content::input_definitions)) |
| `Invocation(?) -> Tokens`<br>`Invocation(?, ?) -> Tokens`<br>`Invocation(?, ?, ?) -> Tokens`<br>`Invocation(?, ?, ?, ?) -> Tokens`<br>`Invocation(?, array) -> Tokens` | The tokens invoking a control sequence on its arguments (each `Tokens`, TeX source or `()` for an omitted optional); TeX with `#n` is an anonymous macro. ([`build_invocation`](latexml_core::binding::content::build_invocation)) |
| `IsDefined(string\|Token) -> bool` | Whether a token (CS name or `Token`) is defined, and not `\let` to `\relax`. ([`is_defined_token`](latexml_core::binding::def::dialect::is_defined_token)) |
| `IsEmpty(?) -> bool` | Perl's `IsEmpty`: true when nothing but spaces, braces and other non-printing tokens (or empty digested content). |
| `LaTeXMLVersion() -> string` | The engine version string — the `LATEXML_VERSION` value ([`lookup_string`](latexml_core::state::lookup_string)). |
| `Let(string\|Token, string\|Token)`<br>`Let(string\|Token, string\|Token, string)` | TeX's `\let`: copy one token's meaning onto another, with an optional scope (`"global"` is `\global\let`). Each token is a CS name string or a `Token`. ([`let_definition`](latexml_core::state::let_definition)) |
| `LoadClass(string)`<br>`LoadClass(string, array)`<br>`LoadClass(string, map)` | Load a document class, falling back to a prefix match then OmniBus. ([`load_class`](latexml_core::binding::content::load_class)) |
| `LoadPool(string)` | Load a pool (`<name>.pool`). ([`input_definitions`](latexml_core::binding::content::input_definitions)) |
| `LookupBool(string) -> bool` | Read a value as a boolean. ([`lookup_bool`](latexml_core::state::lookup_bool)) |
| `LookupCatcode(string) -> ?` | The category code currently in force for a character; `()` when the table holds none. ([`lookup_catcode`](latexml_core::state::lookup_catcode)) |
| `LookupDefinition(string\|Token) -> Definition?` | Fetch the installed definition of a CS name or `Token` so hooks can be pushed onto it; `()` when undefined or `\let` to a character. |
| `LookupDelcode(string) -> ?` | A character's delimiter code; `()` when the table holds none. ([`lookup_delcode`](latexml_core::state::lookup_delcode)) |
| `LookupFloat(string) -> float` | Read a value as a fractional (float) number, keeping the fraction `LookupNumber` would truncate. ([`lookup_float`](latexml_core::state::lookup_float)) |
| `LookupLCcode(string) -> ?` | A character's lowercase code; `()` when the table holds none. ([`lookup_lccode`](latexml_core::state::lookup_lccode)) |
| `LookupMapping(string, string) -> ?` | Read one key out of a named mapping, with the type it was stored as; `()` when absent. ([`lookup_mapping`](latexml_core::state::lookup_mapping)) |
| `LookupMappingKeys(string) -> array` | The keys of a named mapping, sorted. ([`with_mapping_keys`](latexml_core::state::with_mapping_keys)) |
| `LookupMathcode(string) -> ?` | A character's math code; `()` when the table holds none. ([`lookup_mathcode`](latexml_core::state::lookup_mathcode)) |
| `LookupMeaning(string\|Token) -> string` | What a token means right now: its definition, or itself. ([`lookup_meaning`](latexml_core::state::lookup_meaning)) |
| `LookupNumber(string) -> int` | Read a value as a number. ([`lookup_number`](latexml_core::state::lookup_number)) |
| `LookupRegisterOrDefault(string\|Token) -> string` | A register's value as its TeX string (the type's zero when `cs` is no register); the typed handle is round C's. |
| `LookupSFcode(string) -> ?` | A character's space-factor code; `()` when the table holds none. ([`lookup_sfcode`](latexml_core::state::lookup_sfcode)) |
| `LookupString(string) -> string` | Read a value as a string; empty when unset. ([`lookup_string`](latexml_core::state::lookup_string)) |
| `LookupTokens(string) -> Tokens` | Read a value as tokens. ([`lookup_tokens`](latexml_core::state::lookup_tokens)) |
| `LookupUCcode(string) -> ?` | A character's uppercase code; `()` when the table holds none. ([`lookup_uccode`](latexml_core::state::lookup_uccode)) |
| `LookupValue(string) -> ?` | Read a value with the type it was stored as (a list as an array); `()` when unset. ([`lookup_value`](latexml_core::state::lookup_value)) |
| `MergeFont(map)` | Merge font attributes into the current font, group-locally. ([`merge_font`](latexml_core::binding::content::merge_font)) |
| `NewCounter(string)`<br>`NewCounter(string, string)`<br>`NewCounter(string, string, map)` | Declare a new counter. ([`new_counter`](latexml_core::binding::counter::dialect::new_counter)) |
| `Note(string)` | Write a progress note to both the conversion log and stderr. |
| `NoteLog(string)` | Write a progress note to the conversion log only. |
| `NoteSTDERR(string)` | Write a progress note to stderr only. |
| `ParseXML(string) -> array` | Parse a markup chunk into nodes; a bare fragment is fine. ([`parse_fragment`](latexml_core::common::xml::parse_fragment)) |
| `PassOptions(string, string, array)` | Forward options to a package or class not yet loaded. ([`pass_options`](latexml_core::binding::content::pass_options)) |
| `PopValue(string) -> ?` | Pop and return the last value of a value-table list, type preserved (`()` if empty). ([`pop_value`](latexml_core::state::pop_value)) |
| `PrependSearchPath(string)` | Prepend a directory to the input search path (searched first). ([`search_paths_push_front`](latexml_core::state::search_paths_push_front)) |
| `ProcessKeyOptions()`<br>`ProcessKeyOptions(bool)`<br>`ProcessKeyOptions(map)` | `ProcessOptions` for a package whose options are key=value pairs; `\@curroptions` is kept. ([`process_key_options`](latexml_core::binding::content::process_key_options)) |
| `ProcessOptions()`<br>`ProcessOptions(bool)`<br>`ProcessOptions(map)` | Execute the declared options — in declaration order, or (`*`) in the order passed. ([`process_options`](latexml_core::binding::content::process_options)) |
| `ProgressSpindown(string)` | Close a named progress stage in the log. ([`note_end`](latexml_core::common::error::note_end)) |
| `ProgressSpinup(string)` | Open a named progress stage in the log. ([`note_begin`](latexml_core::common::error::note_begin)) |
| `ProgressStep(string)` | Advance the progress indicator; a no-op in this port. ([`progress_step`](latexml_core::common::error::progress_step)) |
| `PushValue(string, ?)` | Push a value (string/int/float/bool/Tokens) onto a value-table list, creating it (global) if absent. ([`push_value`](latexml_core::state::push_value)) |
| `RawTeX(string)` | Process a chunk of literal TeX as definitions. ([`raw_tex`](latexml_core::stomach::raw_tex)) |
| `ReadArg() -> Tokens` | Read one TeX argument, a token or a braced group, as `Tokens`. ([`read_arg`](latexml_core::gullet::read_arg)) |
| `ReadOptional() -> ?` | Read a LaTeX optional `[…]` argument as `Tokens`; `()` when there is none. ([`read_optional`](latexml_core::gullet::read_optional)) |
| `ReadToken() -> ?` | Read the next token, unexpanded; `()` at the end of input. ([`read_token`](latexml_core::gullet::read_token)) |
| `ReadUntil(?) -> Tokens` | Read a balanced token sequence up to a delimiter (TeX source, `Token` or `Tokens`), as `Tokens`. ([`read_until`](latexml_core::gullet::read_until)) |
| `RefCurrentID(string) -> map` | Reuse the last id without stepping, when its box was pruned. ([`ref_current_id`](latexml_core::binding::counter::dialect::ref_current_id)) |
| `RefStepCounter(string) -> map`<br>`RefStepCounter(string, bool) -> map` | Step a counter through `\refstepcounter`'s meaning (`noreset`: directly) and return its `refnum` and `id`. ([`ref_step_counter_by_meaning`](latexml_core::binding::counter::dialect::ref_step_counter_by_meaning)) |
| `RefStepID(string) -> map` | Step only the uncounter, for an UN-numbered unit; returns the id. ([`ref_step_id`](latexml_core::binding::counter::dialect::ref_step_id)) |
| `RegisterDocumentNamespace(string, string)` | Bind an OUTPUT-document prefix to a namespace URI. ([`register_document_namespace`](latexml_core::common::model::register_document_namespace)) |
| `RegisterNamespace(string, string)` | Bind a CODE prefix to a namespace URI. ([`register_namespace`](latexml_core::common::model::register_namespace)) |
| `RelaxNGSchema(string)` | Select the RelaxNG schema defining the output language. ([`select_relaxng_schema`](latexml_core::binding::content::select_relaxng_schema)) |
| `RequirePackage(string)`<br>`RequirePackage(string, array)`<br>`RequirePackage(string, map)` | Load a package (options: Perl's `options`, `type`, `notex`, `after`, …): its binding, or the raw `.sty` when `INCLUDE_STYLES` allows. ([`require_package`](latexml_core::binding::content::require_package)) |
| `RequireResource(string)`<br>`RequireResource(string, map)` | Attach a CSS or JavaScript resource to the document. ([`require_resource`](latexml_core::binding::content::require_resource)) |
| `ResetCounter(string)` | Reset a counter to zero. ([`reset_counter`](latexml_core::binding::counter::dialect::reset_counter)) |
| `Revert(Digested) -> Tokens` | A digested value back to the source tokens that made it. ([`Digested::revert`](latexml_core::digested::Digested::revert)) |
| `Roman(int) -> Tokens` | Upper-case roman numerals of a number, as `Tokens`. |
| `SetCondition(string\|Token, bool)`<br>`SetCondition(string\|Token, bool, string)` | Set a test-less conditional (`\newif` style) to true or false. ([`set_condition`](latexml_core::binding::content::set_condition)) |
| `SetCounter(string, int)` | Set a counter's value globally. ([`set_counter`](latexml_core::binding::counter::dialect::set_counter)) |
| `ShiftValue(string) -> ?` | Remove and return the first value of a value-table list, type preserved (`()` if empty). ([`shift_value`](latexml_core::state::shift_value)) |
| `SkipSpaces()` | Discard any run of spaces at the head of the input. ([`skip_spaces`](latexml_core::gullet::skip_spaces)) |
| `StartSemiverbatim()`<br>`StartSemiverbatim(array)` | Begin a SEMI-verbatim region: neutralize the math/special catcodes (`^ _ ~ & $ # '`) to literal `OTHER`, but keep `\ { }` special so control sequences and `\end{env}` still parse. This is NOT full verbatim — a backslash command is still expanded (e.g. the Mathematica `\[Gamma]` fires `\[`, which opens display math), and because `{` `}` stay group-markers a literal brace in the source is consumed as grouping, not printed: `{}` yields nothing and `{x}` yields `x`. Only the `{verbatim}` / fancyvrb environments read a body as raw lines where even `\ { }` are literal. Pass extra single-char strings to neutralize them too — but not `\`, since `\end{env}` needs it. Pair with `EndSemiverbatim` in a `DefEnvironment`'s `beforeDigest`/`beforeDigestEnd`. ([`begin_semiverbatim`](latexml_core::state::begin_semiverbatim)) |
| `StepCounter(string)` | Step a counter; usually you want `RefStepCounter` instead. ([`step_counter`](latexml_core::binding::counter::dialect::step_counter)) |
| `T_ACTIVE(string) -> Token` | One active-character `Token` (Perl `T_ACTIVE`), e.g. for `Let(T_ACTIVE("~"), "\\nobreakspace")`. |
| `T_ALIGN() -> Token` | The `Token` `&` under `CC_ALIGN` (a Perl constant; call it as `T_ALIGN()`). |
| `T_ARG(?) -> Token` | A parameter `Token` `#n` under `CC_ARG`, from an int 1-9 or a digit `Token` (Perl `T_ARG`). |
| `T_BEGIN() -> Token` | The `Token` `{` under `CC_BEGIN` (a Perl constant; call it as `T_BEGIN()`). |
| `T_COMMENT(string) -> Token` | A comment `Token`: `%` followed by the text, under `CC_COMMENT` (Perl `T_COMMENT`). |
| `T_CR() -> Token` | The `Token` a newline under `CC_SPACE` (a Perl constant; call it as `T_CR()`). |
| `T_CS(string) -> Token` | One control-sequence `Token`; `Digest`, `Expand`, `UnTeX`, `absorb` take it as a one-token list. |
| `T_END() -> Token` | The `Token` `}` under `CC_END` (a Perl constant; call it as `T_END()`). |
| `T_LETTER(string) -> Token` | One letter `Token` (catcode 11). |
| `T_MARKER(string) -> Token` | A marker `Token` under `CC_MARKER` (Perl `T_MARKER`, marked there "don't use unless you know"). |
| `T_MATH() -> Token` | The `Token` `$` under `CC_MATH` (a Perl constant; call it as `T_MATH()`). |
| `T_OTHER(string) -> Token` | One other-character `Token` (catcode 12). |
| `T_PARAM() -> Token` | The `Token` `#` under `CC_PARAM` (a Perl constant; call it as `T_PARAM()`). |
| `T_SPACE() -> Token` | The `Token` a space under `CC_SPACE` (a Perl constant; call it as `T_SPACE()`). |
| `T_SUB() -> Token` | The `Token` `_` under `CC_SUB` (a Perl constant; call it as `T_SUB()`). |
| `T_SUPER() -> Token` | The `Token` `^` under `CC_SUPER` (a Perl constant; call it as `T_SUPER()`). |
| `Tag(string, map)` | Declare document-model properties for one element tag: `autoOpen`, `autoClose`, the `afterOpen`/`afterClose` hooks. ([`install_tag`](latexml_core::binding::content::install_tag)) |
| `TeX(string)` | Tokenize a TeX source string (style catcodes) and digest it. ([`digest`](latexml_core::stomach::digest)) |
| `ToAttribute(Digested) -> string` | The digested value as a string fit for an XML attribute. ([`Digested::to_attribute`](latexml_core::digested::Digested::to_attribute)) |
| `ToString(?) -> string`<br>`ToString(Whatsit) -> string` | Perl's `ToString`: a box, Tokens or Token as plain text, a whatsit as its reversion, `()` as `""`. |
| `Today() -> string` | Today's date, as `\today` renders it. ([`today`](latexml_engine::base_utilities::today)) |
| `Token(string) -> Token`<br>`Token(string, int) -> Token` | Perl's `Token(string, catcode)`: one `Token`; the catcode is a `CC_*` value, `CC_OTHER()` if omitted. |
| `Tokenize(string) -> Tokens` | Tokenize a string under the standard catcode table. ([`tokenize`](latexml_core::mouth::tokenize)) |
| `TokenizeInternal(string) -> Tokens` | Tokenize a string under the style-file table, where `@` is a letter. ([`tokenize_internal`](latexml_core::mouth::tokenize_internal)) |
| `Tokens() -> Tokens`<br>`Tokens(?) -> Tokens`<br>`Tokens(?, ?) -> Tokens`<br>`Tokens(?, ?, ?) -> Tokens`<br>`Tokens(?, ?, ?, ?) -> Tokens`<br>`Tokens(array) -> Tokens` | Perl's `Tokens(@tokens)`: a token list from `Token`s and `Tokens` (flattened), given as up to four arguments or as an array. Where a call takes a `Token` (`Let`, `XEquals`, …), a `Tokens` is an error. |
| `UTF(int) -> string` | The character of a Unicode code point. |
| `UnTeX(Token) -> string`<br>`UnTeX(Tokens) -> string` | Tokens (or one `Token`) back to the TeX source that could have produced them. ([`Tokens::untex`](latexml_core::tokens::Tokens::untex)) |
| `UnshiftValue(string, ?)` | Prepend a value (string/int/float/bool/Tokens/Token) to a value-table list, creating it (global) if absent. ([`unshift_value`](latexml_core::state::unshift_value)) |
| `Warn(string, string, string)`<br>`Warn(string, string, string, ?)` | Log a `Warning:` with the given category and object, plus optional details. |
| `XEquals(string\|Token, string\|Token) -> bool` | Whether two tokens (CS name strings or `Token`s) have the same meaning. ([`x_equals`](latexml_core::state::x_equals)) |
| `assign_global(string, string)` | **Deprecated** (Warns once per binding load): use `AssignValue(key, value, "global")`. [`assign_value`](latexml_core::state::assign_value) with [`Scope::Global`](latexml_core::state::Scope::Global): the binding survives the enclosing TeX group. |
| `assign_value(string, string)` | **Deprecated** (Warns once per binding load): use `AssignValue(key, value, "local")`. [`assign_value`](latexml_core::state::assign_value) with [`Scope::Local`](latexml_core::state::Scope::Local), TeX's default: the binding expires with the enclosing group. |
| `defined(?) -> bool` | Perl's `defined($v)`: true for every value but `()` (`""`, `0` and empty Tokens are defined); `v == ()` is not the test for a string, which compares equal to `()` when empty. |
| `lookup_value(string) -> string` | **Deprecated** (Warns once per binding load): use `LookupValue(key)`. [`lookup_value`](latexml_core::state::lookup_value) coerced to a string; empty when unset, so an unbound key and an empty one are indistinguishable here. |
| `neutralize_font()` | Reset the text and math fonts to their defaults, group-locally. ([`neutralize_font`](latexml_engine::base_utilities::neutralize_font)) |
| `roman(int) -> Tokens` | Lower-case roman numerals of a number, as `Tokens`. |
| `whatsit() -> Whatsit` | **Deprecated** (Warns once per binding load): use the hook's whatsit argument. A handle on the whatsit under construction, inside a hook. USING it outside one is a clean script error, not a crash. |

## `document` methods

Reached through the `document` handle a constructor body receives as its first argument — Perl's `$document->method`.

29 functions, 44 calls.

| call | documentation |
|---|---|
| `absorb(Digested)`<br>`absorb(Token)`<br>`absorb(Tokens)` | Absorb a digested value at the current insertion point. ([`Document::absorb`](latexml_core::document::Document::absorb)) |
| `absorbProperty(string)` | Absorb one of the whatsit's properties, by name. ([`Document::absorb`](latexml_core::document::Document::absorb)) |
| `absorbString(?)` | Absorb a plain string at the current insertion point. ([`Document::absorb_string`](latexml_core::document::Document::absorb_string)) |
| `addClass(Node, string)` | Add CSS classes to a node, keeping those it already has. ([`Document::add_class`](latexml_core::document::Document::add_class)) |
| `appendClone(Node, array)` | Append COPIES of nodes, with fresh ids. ([`Document::append_clone`](latexml_core::document::Document::append_clone)) |
| `canContain(Node, string) -> bool`<br>`canContain(string, string) -> bool` | May an element (a node or a qname) contain a child qname? — the schema query before opening an element. ([`can_contain`](latexml_core::document::can_contain)) |
| `closeElement(string) -> Node?` | Close the deepest open element of that name and return it (`()` when it is not open). ([`Document::close_element`](latexml_core::document::Document::close_element)) |
| `closeElementAt(Node)` | Close an element that was opened with `openElementAt`. ([`Document::close_element_at`](latexml_core::document::Document::close_element_at)) |
| `findnode(string) -> Node?`<br>`findnode(string, Node) -> Node?` | The first node matching an XPath. ([`Document::findnode`](latexml_core::document::Document::findnode)) |
| `findnodes(string) -> array`<br>`findnodes(string, Node) -> array` | Every node matching an XPath. ([`Document::findnodes`](latexml_core::document::Document::findnodes)) |
| `floatToElement(string) -> Node?`<br>`floatToElement(string, bool) -> Node?` | Move the insertion point up to the nearest ancestor that can contain a qname; returns the previous point to `setNode` back to (or `()`). ([`Document::float_to_element`](latexml_core::document::Document::float_to_element)) |
| `generateID(Node, string)` | Give a node an `xml:id`, if it has none. ([`Document::generate_id`](latexml_core::document::Document::generate_id)) |
| `getElement() -> Node?` | The element at, or containing, the insertion point. ([`Document::get_element`](latexml_core::document::Document::get_element)) |
| `getNode() -> Node` | The current insertion point. ([`Document::get_node`](latexml_core::document::Document::get_node)) |
| `getProperties() -> map` | **Deprecated** (Warns once per binding load): use the constructor body's trailing properties map. The whatsit's whole property map (a Rhai object map), inside a constructor body. |
| `getProperty(string) -> ?` | **Deprecated** (Warns once per binding load): use the constructor body's trailing properties map. One whatsit property by name, inside a constructor body; `()` when absent. |
| `insertElement(string) -> Node`<br>`insertElement(string, Digested) -> Node`<br>`insertElement(string, Digested, map) -> Node`<br>`insertElement(string, map) -> Node` | Open, absorb and close in one step; returns the new element. ([`Document::insert_element`](latexml_core::document::Document::insert_element)) |
| `insertPI(string)`<br>`insertPI(string, map)` | Insert a processing instruction: a target, with optional attributes. ([`Document::insert_pi`](latexml_core::document::Document::insert_pi)) |
| `insertXML(Node)`<br>`insertXML(array)`<br>`insertXML(string)` | Splice ALREADY-PARSED nodes in at the insertion point. ([`Document::insert_nodes`](latexml_core::document::Document::insert_nodes)) |
| `maybeCloseElement(string) -> Node?` | Close an element if it is open and closeable and return it; otherwise `()`. ([`Document::maybe_close_element`](latexml_core::document::Document::maybe_close_element)) |
| `openElement(string) -> Node`<br>`openElement(string, map) -> Node` | Open an element (with an optional attribute map), make it the insertion point and return it. ([`Document::open_element`](latexml_core::document::Document::open_element)) |
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

## `whatsit` methods

The whatsit a hook receives positionally, as Perl passes `$whatsit` (Whatsit.pm).

7 functions, 7 calls.

| call | documentation |
|---|---|
| `argString(int) -> string` | **Deprecated** (Warns once per binding load): use `whatsit.getArg(n)`. The whatsit's nth argument, reverted to source text. |
| `getArg(int) -> ?` | The nth argument (from 1), as a constructor body receives it: a `Digested` handle, `Tokens` for an undigested one, `()` when absent. ([`Whatsit::get_arg`](latexml_core::whatsit::Whatsit::get_arg)) |
| `getProperties() -> map` | The whole property map (a Rhai object map), each value typed. ([`BoxOps::get_properties`](latexml_core::BoxOps::get_properties)) |
| `getProperty(string) -> ?` | One property by name, its typed value (string / int / bool / `Tokens` / `Digested` handle / …); `()` when absent. ([`BoxOps::get_property`](latexml_core::BoxOps::get_property)) |
| `propertyString(string) -> string` | **Deprecated** (Warns once per binding load): use `whatsit.getProperty(key)`. One property of the whatsit, as a string. |
| `setProperty(string, ?)` | Set a property (any value) on the whatsit. Only a DIGEST hook may: in a construction hook the whatsit is already read-only, and this is a clean script error. |
| `to_string() -> string` | The whatsit in a string: Perl's `ToString($whatsit)`, its reversion (an error inside its own reversion). |

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

## `Gullet` methods

Perl's `$gullet->read…`: the gullet handle a macro body or a parameter-type reader receives first.

5 functions, 5 calls.

| call | documentation |
|---|---|
| `readArg() -> Tokens` | `gullet.readArg()`: `ReadArg` on a `Gullet` handle. ([`read_arg`](latexml_core::gullet::read_arg)) |
| `readOptional() -> ?` | `gullet.readOptional()`: `ReadOptional` on a `Gullet` handle. ([`read_optional`](latexml_core::gullet::read_optional)) |
| `readToken() -> ?` | `gullet.readToken()`: `ReadToken` on a `Gullet` handle. ([`read_token`](latexml_core::gullet::read_token)) |
| `readUntil(?) -> Tokens` | `gullet.readUntil(delim)`: `ReadUntil` on a `Gullet` handle. ([`read_until`](latexml_core::gullet::read_until)) |
| `skipSpaces()` | `gullet.skipSpaces()`: `SkipSpaces` on a `Gullet` handle. ([`skip_spaces`](latexml_core::gullet::skip_spaces)) |

## `Stomach` methods

Perl's `$stomach`: the handle a primitive body receives first (Primitive.pm:78).

1 function, 1 call.

| call | documentation |
|---|---|
| `getGullet() -> Gullet` | `stomach.getGullet()`: the `Gullet` handle, Perl's `$stomach->getGullet`. |

## `Font` methods

Perl's Common/Font.pm getters, on the font a `DefLigature` `fontTest` closure receives.

6 functions, 6 calls.

| call | documentation |
|---|---|
| `getEncoding() -> ?` | A `Font`'s encoding, or `()`. |
| `getFamily() -> ?` | A `Font`'s family (`serif`, `typewriter`, …), or `()`. |
| `getLanguage() -> ?` | A `Font`'s language, or `()`. |
| `getSeries() -> ?` | A `Font`'s series, or `()`. |
| `getShape() -> ?` | A `Font`'s shape, or `()`. |
| `getSize() -> ?` | A `Font`'s size in points, or `()`. |

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
