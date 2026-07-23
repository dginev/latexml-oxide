# The `.rhai` binding interface

Every function a runtime (Rhai) binding can call, grouped by the handle it is
called on. **Generated** from the live engine via Rhai's reflections API
(`Engine::gen_fn_signatures`) and checked on every test run, so it always
matches what is actually registered — see `api_doc.rs`.

Overloads are listed separately, one line per accepted argument shape.

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

115 functions.

```text
AddToCounter(string, int)
AssignCatcode(string, int)
AssignMapping(string, string, string)
AssignMeaning(string, string)
AssignValue(string, string)
AssignValue(string, string, string)
Command(string) -> Command
CounterValue(string) -> int
DeclareOption(Fn)
DeclareOption(string, Fn)
DeclareOption(string, string)
DefAccent(string, string, string)
DefAccent(string, string, string, bool)
DefColumnType(string, Fn)
DefConditional(string, Fn)
DefConstructor(string, Fn)
DefConstructor(string, Fn, map)
DefConstructor(string, string)
DefConstructor(string, string, map)
DefEnvironment(string, Fn)
DefEnvironment(string, Fn, map)
DefEnvironment(string, string)
DefEnvironment(string, string, map)
DefKeyVal(string, string, string)
DefKeyVal(string, string, string, string)
DefKeyVal(string, string, string, string, map)
DefLigature(string, string)
DefMacro(string, Fn)
DefMacro(string, Fn, map)
DefMacro(string, string)
DefMacro(string, string, map)
DefMath(string, string)
DefMath(string, string, map)
DefMathLigature(Fn)
DefMathLigature(string, string, map)
DefMathRewrite(map)
DefMathRewrite(map, Fn)
DefPrimitive(string, Fn)
DefPrimitive(string, Fn, map)
DefRegister(string, int)
DefRegister(string, string)
DefRewrite(map)
DefRewrite(map, Fn)
Digest(Tokens) -> Digested
DigestText(string) -> Digested
Error(string, string, string)
ExecuteOptions(array)
Expand(Tokens) -> Tokens
ExpandPartially(Tokens) -> Tokens
Fatal(string, string, string)
GetKeyVal(?, string) -> string
GetKeyVals(string) -> map
Info(string, string, string)
InputDefinitions(string)
InputDefinitions(string, map)
IsDefined(string) -> bool
LaTeXMLVersion() -> string
Let(string, string)
LoadClass(string)
LoadClass(string, array)
LookupBool(string) -> bool
LookupCatcode(string) -> int
LookupDefinition(string) -> Definition?
LookupMapping(string, string) -> string
LookupMeaning(string) -> string
LookupNumber(string) -> int
LookupString(string) -> string
LookupTokens(string) -> Tokens
LookupValue(string) -> ?
MergeFont(map)
NewCounter(string)
NewCounter(string, string)
NoteLog(string)
NoteSTDERR(string)
ParseXML(string) -> array
PassOptions(string, string, array)
ProcessOptions()
ProcessOptions(bool)
ProgressSpindown(string)
ProgressSpinup(string)
ProgressStep(string)
RawTeX(string)
ReadArg() -> string
ReadOptional() -> string
ReadUntil(string) -> string
RefCurrentID(string) -> map
RefStepCounter(string) -> map
RefStepID(string) -> map
RegisterDocumentNamespace(string, string)
RegisterNamespace(string, string)
RelaxNGSchema(string)
RequirePackage(string)
RequirePackage(string, array)
RequireResource(string)
RequireResource(string, map)
ResetCounter(string)
Revert(Digested) -> Tokens
SkipSpaces()
StepCounter(string)
T_CS(string) -> Tokens
Tag(string, map)
TeX(string)
ToAttribute(Digested) -> string
ToString(Digested) -> string
Today() -> string
Tokenize(string) -> Tokens
TokenizeInternal(string) -> Tokens
UnTeX(Tokens) -> string
Warn(string, string, string)
XEquals(string, string) -> bool
assign_global(string, string)
assign_value(string, string)
lookup_value(string) -> string
neutralize_font()
whatsit() -> Whatsit
```

## `document` methods

Reached through the `document` handle a constructor body receives as its first argument — Perl's `$document->method`.

33 functions.

```text
absorb(Digested)
absorbProperty(string)
absorbString(string)
addClass(Node, string)
appendClone(Node, array)
closeElement(string)
closeElementAt(Node)
findnode(string) -> Node?
findnode(string, Node) -> Node?
findnodes(string) -> array
findnodes(string, Node) -> array
generateID(Node, string)
getElement() -> Node?
getNode() -> Node
insertElement(string) -> Node
insertElement(string, Digested) -> Node
insertElement(string, Digested, map) -> Node
insertElement(string, map) -> Node
insertXML(Node)
insertXML(array)
insertXML(string)
maybeCloseElement(string)
openElement(string)
openElementAt(Node, string) -> Node
openElementAt(Node, string, map) -> Node
removeNode(Node)
renameNode(Node, string) -> Node
renameNode(Node, string, bool) -> Node
replaceNode(Node, array)
setAttribute(string, string)
setNode(Node)
unwrapNodes(Node)
wrapNodes(string, array) -> Node?
```

## `Node` methods

An XML node: one returned by `ParseXML`, or one handed to a rewrite / ligature-matcher body.

15 functions.

```text
children() -> array
content() -> string
firstChild() -> Node?
getAttribute(string) -> string
hasAttribute(string) -> bool
name() -> string
nextSibling() -> Node?
parent() -> Node?
prevSibling() -> Node?
qname() -> string
removeAttribute(string)
setAttribute(string, string)
setContent(string)
toString() -> string
unlink()
```

## `whatsit()` methods

The whatsit under construction, inside a digest hook.

3 functions.

```text
argString(int) -> string
propertyString(string) -> string
setProperty(string, string)
```

## `LookupDefinition()` methods

An already-installed definition, for pushing hooks onto it.

10 functions.

```text
pushAfterConstruct(Fn)
pushAfterDigest(Fn)
pushAfterDigestBody(Fn)
pushBeforeConstruct(Fn)
pushBeforeDigest(Fn)
unshiftAfterConstruct(Fn)
unshiftAfterDigest(Fn)
unshiftAfterDigestBody(Fn)
unshiftBeforeConstruct(Fn)
unshiftBeforeDigest(Fn)
```

## `Command` methods

The `std::process::Command` mirror, for a binding that shells out.

5 functions.

```text
arg(string)
args(array)
current_dir(string)
env(string, string)
output() -> ?
```
