# Perl | Rust macro | Rhai: the binding dialects side by side

One row per call: its Perl LaTeXML spelling (a `Package.pm` function, or the Perl method or expression that does the
same), the compile-time Rust macro that lowers to the same core call, and the runtime `.rhai` call. **Generated** from
`api_doc.rs` with [`API.md`](API.md), and checked with it on every test run (`api_reference_is_up_to_date`); regenerate
with `UPDATE_API_DOC=1 cargo test -p latexml_contrib --lib api_reference_is_up_to_date`.

An empty cell is a gap, listed at the end: a Perl call the dialect lacks, a Rhai call Perl has no counterpart for (a
Rhai-only helper), or a Perl function the macros do not spell. Methods (`document.openElement`) have no macro: a
compiled binding calls the Rust method. A deprecated alias follows the call that replaces it, marked `(deprecated: …)`
(the aliases of `mod.rs`'s `DEPRECATED_ALIASES`). Each Rhai call's arguments and meaning are in [`API.md`](API.md); the
newcomer guide is [`docs/parity/RHAI_BINDINGS_GUIDE.md`](../../../docs/parity/RHAI_BINDINGS_GUIDE.md).

| Perl | Rust macro | Rhai |
|---|---|---|
| `!$v` |  | `!` |
| `$a ne $b` |  | `!=` |
| `$v->multiply($n)` |  | `*` |
| `$v->add($w)` |  | `+` |
| `$v->subtract($w)` |  | `-` |
| `$v->divide($n)` |  | `/` |
| `$v->valueOf < $w->valueOf` |  | `<` |
| `$v->valueOf <= $w->valueOf` |  | `<=` |
| `$a eq $b` |  | `==` |
| `$v->valueOf > $w->valueOf` |  | `>` |
| `$v->valueOf >= $w->valueOf` |  | `>=` |
| `AddToCounter` | `AddToCounter!` | `AddToCounter` |
| `AddToMacro` | `AddToMacro!` | `AddToMacro` |
|  |  | `AppendSearchPath` |
| `AssignCatcode` | `AssignCatcode!` | `AssignCatcode` |
| `AssignDelcode` |  | `AssignDelcode` |
| `AssignValue` |  | `AssignFloat` |
| `AssignLCcode` |  | `AssignLCcode` |
| `AssignMapping` | `AssignMapping!` | `AssignMapping` |
| `AssignMathcode` |  | `AssignMathcode` |
| `$STATE->assignMeaning` | `AssignMeaning!` | `AssignMeaning` |
| `AssignRegister` | `AssignRegister!` | `AssignRegister` |
| `AssignSFcode` |  | `AssignSFcode` |
| `AssignUCcode` |  | `AssignUCcode` |
| `AssignValue` | `AssignValue!` | `AssignValue` |
| `AssignValue` |  | `AssignBool` (deprecated: `AssignValue(key, value[, scope])`) |
| `AssignValue` |  | `AssignNumber` (deprecated: `AssignValue(key, value[, scope])`) |
| `AssignValue` |  | `AssignString` (deprecated: `AssignValue(key, value[, scope])`) |
| `AssignValue` |  | `assign_global` (deprecated: `AssignValue(key, value, "global")`) |
| `AssignValue` |  | `assign_value` (deprecated: `AssignValue(key, value, "local")`) |
| `AtBeginDocument` |  | `AtBeginDocument` |
| `AtEndDocument` |  | `AtEndDocument` |
| `beginItemize` | `BeginItemize!` | `BeginItemize` |
| `CC_ACTIVE` |  | `CC_ACTIVE` |
| `CC_ALIGN` |  | `CC_ALIGN` |
| `CC_ARG` |  | `CC_ARG` |
| `CC_BEGIN` |  | `CC_BEGIN` |
| `CC_COMMENT` |  | `CC_COMMENT` |
| `CC_CS` |  | `CC_CS` |
| `CC_END` |  | `CC_END` |
| `CC_EOL` |  | `CC_EOL` |
| `CC_ESCAPE` |  | `CC_ESCAPE` |
| `CC_IGNORE` |  | `CC_IGNORE` |
| `CC_INVALID` |  | `CC_INVALID` |
| `CC_LETTER` |  | `CC_LETTER` |
| `CC_MARKER` |  | `CC_MARKER` |
| `CC_MATH` |  | `CC_MATH` |
| `CC_OTHER` |  | `CC_OTHER` |
| `CC_PARAM` |  | `CC_PARAM` |
| `CC_SPACE` |  | `CC_SPACE` |
| `CC_SUB` |  | `CC_SUB` |
| `CC_SUPER` |  | `CC_SUPER` |
| `CleanID` |  | `CleanID` |
| `CleanLabel` |  | `CleanLabel` |
|  |  | `Command` |
| `CounterValue` | `CounterValue!` | `CounterValue` |
| `DeclareFontMap` | `DeclareFontMap!` | `DeclareFontMap` |
|  | `DeclareFontMapMultichar!` | `DeclareFontMapMultichar` |
| `DeclareOption` | `DeclareOption!` | `DeclareOption` |
| `DefAccent` | `DefAccent!` | `DefAccent` |
| `DefColumnType` | `DefColumnType!` | `DefColumnType` |
| `DefConditional` | `DefConditional!` | `DefConditional` |
| `DefConstructor` | `DefConstructor!` | `DefConstructor` |
| `DefEnvironment` | `DefEnvironment!` | `DefEnvironment` |
| `DefKeyVal` | `DefKeyVal!` | `DefKeyVal` |
| `DefLigature` | `DefLigature!` | `DefLigature` |
| `DefMacro` | `DefMacro!` | `DefMacro` |
| `DefMath` | `DefMath!` | `DefMath` |
| `DefMathLigature` | `DefMathLigature!` | `DefMathLigature` |
| `DefMathRewrite` | `DefMathRewrite!` | `DefMathRewrite` |
| `DefParameterType` | `DefParameterType!` | `DefParameterType` |
| `DefPrimitive` | `DefPrimitive!` | `DefPrimitive` |
| `DefRegister` | `DefRegister!` | `DefRegister` |
| `DefRewrite` | `DefRewrite!` | `DefRewrite` |
| `Digest` | `Digest!` | `Digest` |
| `DigestIf` | `DigestIf!` | `DigestIf` |
| `DigestText` | `DigestText!` | `DigestText` |
| `Dimension` | `Dimension!` | `Dimension` |
| `EndSemiverbatim` |  | `EndSemiverbatim` |
| `Equals` |  | `Equals` |
| `Error` | `Error!` | `Error` |
| `ExecuteOptions` |  | `ExecuteOptions` |
| `Expand` | `Expand!` | `Expand` |
| `ExpandPartially` |  | `ExpandPartially` |
| `Explode` | `Explode!` | `Explode` |
| `ExplodeText` | `ExplodeText!` | `ExplodeText` |
| `Fatal` | `Fatal!` | `Fatal` |
| `FindFile` | `FindFile!` | `FindFile` |
| `GetKeyVal` | `GetKeyVal!` | `GetKeyVal` |
| `GetKeyVals` | `GetKeyVals!` | `GetKeyVals` |
| `Glue` | `Glue!` | `Glue` |
| `Info` | `Info!` | `Info` |
| `Input` | `Input!` | `Input` |
| `InputDefinitions` | `InputDefinitions!` | `InputDefinitions` |
| `Invocation` | `Invocation!` | `Invocation` |
| `IsDefined` | `IsDefined!` | `IsDefined` |
| `IsEmpty` |  | `IsEmpty` |
| `$LaTeXML::VERSION` |  | `LaTeXMLVersion` |
| `Let` | `Let!` | `Let` |
| `LoadClass` | `LoadClass!` | `LoadClass` |
| `LoadPool` | `LoadPool!` | `LoadPool` |
| `LookupValue` | `LookupBool!` | `LookupBool` |
| `LookupCatcode` | `LookupCatcode!` | `LookupCatcode` |
| `LookupDefinition` | `LookupDefinition!` | `LookupDefinition` |
| `LookupDelcode` |  | `LookupDelcode` |
| `LookupDimension` |  | `LookupDimension` |
| `LookupValue` |  | `LookupFloat` |
| `LookupLCcode` |  | `LookupLCcode` |
| `LookupMapping` |  | `LookupMapping` |
| `LookupMappingKeys` |  | `LookupMappingKeys` |
| `LookupMathcode` |  | `LookupMathcode` |
| `LookupMeaning` | `LookupMeaning!` | `LookupMeaning` |
| `LookupValue` | `LookupNumber!` | `LookupNumber` |
| `LookupRegister` | `LookupRegister!` | `LookupRegister` |
|  | `LookupRegisterOrDefault!` | `LookupRegisterOrDefault` |
| `LookupSFcode` |  | `LookupSFcode` |
| `LookupValue` | `LookupString!` | `LookupString` |
| `LookupValue` | `LookupTokens!` | `LookupTokens` |
| `LookupUCcode` |  | `LookupUCcode` |
| `LookupValue` |  | `LookupValue` |
| `LookupValue` |  | `lookup_value` (deprecated: `LookupValue(key)`) |
| `MergeFont` | `MergeFont!` | `MergeFont` |
| `MuGlue` | `MuGlue!` | `MuGlue` |
| `NewCounter` | `NewCounter!` | `NewCounter` |
| `Note` | `Note!` | `Note` |
| `NoteLog` | `NoteLog!` | `NoteLog` |
| `NoteSTDERR` | `NoteSTDERR!` | `NoteSTDERR` |
| `Number` | `Number!` | `Number` |
|  |  | `ParseXML` |
| `PassOptions` |  | `PassOptions` |
| `PopValue` |  | `PopValue` |
|  |  | `PrependSearchPath` |
|  | `ProcessKeyOptions!` | `ProcessKeyOptions` |
| `ProcessOptions` | `ProcessOptions!` | `ProcessOptions` |
| `ProgressSpindown` |  | `ProgressSpindown` |
| `ProgressSpinup` |  | `ProgressSpinup` |
| `ProgressStep` |  | `ProgressStep` |
| `PushValue` |  | `PushValue` |
| `RawTeX` | `RawTeX!` | `RawTeX` |
| `$gullet->readArg` |  | `ReadArg` |
| `$gullet->readOptional` |  | `ReadOptional` |
| `$gullet->readToken` |  | `ReadToken` |
| `$gullet->readUntil` |  | `ReadUntil` |
| `RefCurrentID` | `RefCurrentID!` | `RefCurrentID` |
| `RefStepCounter` | `RefStepCounter!` | `RefStepCounter` |
| `RefStepID` | `RefStepID!` | `RefStepID` |
| `RegisterDocumentNamespace` | `RegisterDocumentNamespace!` | `RegisterDocumentNamespace` |
| `RegisterNamespace` | `RegisterNamespace!` | `RegisterNamespace` |
| `RelaxNGSchema` | `RelaxNGSchema!` | `RelaxNGSchema` |
| `RequirePackage` | `RequirePackage!` | `RequirePackage` |
| `RequireResource` | `RequireResource!` | `RequireResource` |
| `ResetCounter` | `ResetCounter!` | `ResetCounter` |
| `Revert` | `Revert!` | `Revert` |
| `Roman` | `Roman!` | `Roman` |
| `SetCondition` |  | `SetCondition` |
| `SetCounter` | `SetCounter!` | `SetCounter` |
| `ShiftValue` |  | `ShiftValue` |
| `$gullet->skipSpaces` |  | `SkipSpaces` |
| `StartSemiverbatim` |  | `StartSemiverbatim` |
| `StepCounter` | `StepCounter!` | `StepCounter` |
| `T_ACTIVE` | `T_ACTIVE!` | `T_ACTIVE` |
| `T_ALIGN` | `T_ALIGN!` | `T_ALIGN` |
| `T_ARG` | `T_ARG!` | `T_ARG` |
| `T_BEGIN` | `T_BEGIN!` | `T_BEGIN` |
| `T_COMMENT` | `T_COMMENT!` | `T_COMMENT` |
| `T_CR` | `T_CR!` | `T_CR` |
| `T_CS` | `T_CS!` | `T_CS` |
| `T_END` | `T_END!` | `T_END` |
| `T_LETTER` | `T_LETTER!` | `T_LETTER` |
| `T_MARKER` | `T_MARKER!` | `T_MARKER` |
| `T_MATH` | `T_MATH!` | `T_MATH` |
| `T_OTHER` | `T_OTHER!` | `T_OTHER` |
| `T_PARAM` | `T_PARAM!` | `T_PARAM` |
| `T_SPACE` | `T_SPACE!` | `T_SPACE` |
| `T_SUB` | `T_SUB!` | `T_SUB` |
| `T_SUPER` | `T_SUPER!` | `T_SUPER` |
| `Tag` | `Tag!` | `Tag` |
| `Digest(TokenizeInternal($tex))` | `TeX!` | `TeX` |
| `$box->toAttribute` |  | `ToAttribute` |
| `ToString` |  | `ToString` |
|  | `Today!` | `Today` |
| `Token` | `Token!` | `Token` |
| `Tokenize` | `Tokenize!` | `Tokenize` |
| `TokenizeInternal` | `TokenizeInternal!` | `TokenizeInternal` |
| `Tokens` | `Tokens!` | `Tokens` |
| `UTF` |  | `UTF` |
| `UnTeX` |  | `UnTeX` |
| `UnshiftValue` |  | `UnshiftValue` |
| `Warn` | `Warn!` | `Warn` |
| `XEquals` | `XEquals!` | `XEquals` |
| `$v->absolute` |  | `absolute` |
| `$v->add` |  | `add` |
| `defined $v` |  | `defined` |
| `$v->divide` |  | `divide` |
| `$v->divideround` |  | `divideround` |
| `$v->getShrink` |  | `getShrink` |
| `$v->getShrinkOrder` |  | `getShrinkOrder` |
| `$v->getStretch` |  | `getStretch` |
| `$v->getStretchOrder` |  | `getStretchOrder` |
| `$v->larger` |  | `larger` |
| `$v->multiply` |  | `multiply` |
| `$v->negate` |  | `negate` |
|  |  | `neutralize_font` |
| `$v->ptValue` |  | `ptValue` |
| `$v->pxValue` |  | `pxValue` |
| `roman` | `roman!` | `roman` |
| `$v->sign` |  | `sign` |
| `$v->smaller` |  | `smaller` |
| `$v->spValue` |  | `spValue` |
| `$v->subtract` |  | `subtract` |
| `$v->toAttribute` |  | `toAttribute` |
| `$v->valueOf` |  | `valueOf` |
| `$whatsit` |  | `whatsit` (deprecated: the hook's whatsit argument) |
| `$document->absorb` |  | `document.absorb` |
|  |  | `document.absorbProperty` |
| `$document->absorb` |  | `document.absorbString` |
| `$document->addClass` |  | `document.addClass` |
| `$document->appendClone` |  | `document.appendClone` |
| `$document->canContain` |  | `document.canContain` |
| `$document->closeElement` |  | `document.closeElement` |
| `$document->closeElementAt` |  | `document.closeElementAt` |
| `$document->findnode` |  | `document.findnode` |
| `$document->findnodes` |  | `document.findnodes` |
| `$document->floatToElement` |  | `document.floatToElement` |
|  |  | `document.generateID` |
| `$document->getElement` |  | `document.getElement` |
| `$document->getNode` |  | `document.getNode` |
| `$whatsit->getProperties` |  | `document.getProperties` (deprecated: the constructor body's trailing properties map) |
| `$whatsit->getProperty` |  | `document.getProperty` (deprecated: the constructor body's trailing properties map) |
| `$document->insertElement` |  | `document.insertElement` |
| `$document->insertPI` |  | `document.insertPI` |
|  |  | `document.insertXML` |
| `$document->maybeCloseElement` |  | `document.maybeCloseElement` |
| `$document->openElement` |  | `document.openElement` |
| `$document->openElementAt` |  | `document.openElementAt` |
| `$document->removeNode` |  | `document.removeNode` |
| `$document->renameNode` |  | `document.renameNode` |
| `$document->replaceNode` |  | `document.replaceNode` |
| `$document->setAttribute` |  | `document.setAttribute` |
| `$document->setNode` |  | `document.setNode` |
| `$document->unwrapNodes` |  | `document.unwrapNodes` |
| `$document->wrapNodes` |  | `document.wrapNodes` |
| `$node->childNodes` |  | `node.children` |
| `$node->textContent` |  | `node.content` |
| `$node->firstChild` |  | `node.firstChild` |
| `$node->getAttribute` |  | `node.getAttribute` |
| `$node->hasAttribute` |  | `node.hasAttribute` |
| `$node->localname` |  | `node.name` |
| `$node->nextSibling` |  | `node.nextSibling` |
| `$node->parentNode` |  | `node.parent` |
| `$node->previousSibling` |  | `node.prevSibling` |
| `$document->getNodeQName($node)` |  | `node.qname` |
| `$node->removeAttribute` |  | `node.removeAttribute` |
| `$node->setAttribute` |  | `node.setAttribute` |
|  |  | `node.setContent` |
| `$node->toString` |  | `node.toString` |
| `$node->unbindNode` |  | `node.unlink` |
| `$whatsit->getArg` |  | `whatsit.getArg` |
| `$whatsit->getArg` |  | `whatsit.argString` (deprecated: `whatsit.getArg(n)`) |
| `$whatsit->getProperties` |  | `whatsit.getProperties` |
| `$whatsit->getProperty` |  | `whatsit.getProperty` |
| `$whatsit->getProperty` |  | `whatsit.propertyString` (deprecated: `whatsit.getProperty(key)`) |
| `$whatsit->setProperty` |  | `whatsit.setProperty` |
| `ToString($whatsit)` |  | `whatsit.to_string` |
| `push(@{$$defn{afterConstruct}}, …)` |  | `definition.pushAfterConstruct` |
| `push(@{$$defn{afterDigest}}, …)` |  | `definition.pushAfterDigest` |
| `push(@{$$defn{afterDigestBody}}, …)` |  | `definition.pushAfterDigestBody` |
| `push(@{$$defn{beforeConstruct}}, …)` |  | `definition.pushBeforeConstruct` |
| `push(@{$$defn{beforeDigest}}, …)` |  | `definition.pushBeforeDigest` |
| `unshift(@{$$defn{afterConstruct}}, …)` |  | `definition.unshiftAfterConstruct` |
| `unshift(@{$$defn{afterDigest}}, …)` |  | `definition.unshiftAfterDigest` |
| `unshift(@{$$defn{afterDigestBody}}, …)` |  | `definition.unshiftAfterDigestBody` |
| `unshift(@{$$defn{beforeConstruct}}, …)` |  | `definition.unshiftBeforeConstruct` |
| `unshift(@{$$defn{beforeDigest}}, …)` |  | `definition.unshiftBeforeDigest` |
| `$gullet->readArg` |  | `gullet.readArg` |
| `$gullet->readOptional` |  | `gullet.readOptional` |
| `$gullet->readToken` |  | `gullet.readToken` |
| `$gullet->readUntil` |  | `gullet.readUntil` |
| `$gullet->skipSpaces` |  | `gullet.skipSpaces` |
| `$stomach->getGullet` |  | `stomach.getGullet` |
| `$font->getEncoding` |  | `font.getEncoding` |
| `$font->getFamily` |  | `font.getFamily` |
| `$font->getLanguage` |  | `font.getLanguage` |
| `$font->getSeries` |  | `font.getSeries` |
| `$font->getShape` |  | `font.getShape` |
| `$font->getSize` |  | `font.getSize` |
| `$v->toString` |  | `number.toString` |
| `$v->toString` |  | `number.to_string` |
| `$v->toString` |  | `dimension.toString` |
| `$v->toString` |  | `dimension.to_string` |
| `$v->toString` |  | `glue.toString` |
| `$v->toString` |  | `glue.to_string` |
| `$v->toString` |  | `muglue.toString` |
| `$v->toString` |  | `muglue.to_string` |
|  |  | `command.arg` |
|  |  | `command.args` |
|  |  | `command.current_dir` |
|  |  | `command.env` |
|  |  | `command.output` |
| `AfterAssignment` |  |  |
| `Box` |  |  |
| `CheckOptions` |  |  |
| `CleanBibKey` |  |  |
| `CleanClassName` |  |  |
| `CleanIndexKey` |  |  |
| `CleanURL` |  |  |
| `Color` |  |  |
| `ComposeURL` |  |  |
| `Debug` | `Debug!` |  |
| `DefAutoload` |  |  |
| `DefColor` |  |  |
| `DefColorModel` |  |  |
| `DefConditionalI` |  |  |
| `DefConstructorI` |  |  |
| `DefEnvironmentI` |  |  |
| `DefExpandable` |  |  |
| `DefMacroI` |  |  |
| `DefMathI` |  |  |
| `DefPrimitiveI` |  |  |
| `DefRegisterI` |  |  |
| `Deprecated` |  |  |
| `DigestLiteral` |  |  |
| `DisableKeyVal` |  |  |
| `DocType` |  |  |
| `Float` |  |  |
| `FontDecode` |  |  |
| `FontDecodeString` |  |  |
| `GenerateID` |  |  |
| `HasKeyVal` |  |  |
| `IfCondition` |  |  |
| `InputContent` |  |  |
| `InstallDefinition` | `InstallDefinition!` |  |
| `List` |  |  |
| `LoadFontMap` |  |  |
| `LoadFormat` |  |  |
| `LookupColor` | `LookupColor!` |  |
| `MaybeNoteLabel` |  |  |
| `MaybePeekLabel` |  |  |
| `MuDimension` |  |  |
| `NormalizeBibKey` |  |  |
| `Pair` |  |  |
| `PairList` |  |  |
| `ProcessPendingResources` |  |  |
| `ReadAlignmentTemplate` |  |  |
| `ReadParameters` |  |  |
| `Stringify` |  |  |
| `TokensI` |  |  |
| `TrimmedCommaList` |  |  |
| `convertLaTeXArgs` |  |  |
| `createXMRefs` |  |  |
| `decodeMathChar` |  |  |
| `dualize_arglist` |  |  |

## Perl calls the dialect lacks (an empty Rhai cell)

53 calls: `AfterAssignment`, `Box`, `CheckOptions`, `CleanBibKey`, `CleanClassName`, `CleanIndexKey`, `CleanURL`, `Color`, `ComposeURL`, `Debug`, `DefAutoload`, `DefColor`, `DefColorModel`, `DefConditionalI`, `DefConstructorI`, `DefEnvironmentI`, `DefExpandable`, `DefMacroI`, `DefMathI`, `DefPrimitiveI`, `DefRegisterI`, `Deprecated`, `DigestLiteral`, `DisableKeyVal`, `DocType`, `Float`, `FontDecode`, `FontDecodeString`, `GenerateID`, `HasKeyVal`, `IfCondition`, `InputContent`, `InstallDefinition`, `List`, `LoadFontMap`, `LoadFormat`, `LookupColor`, `MaybeNoteLabel`, `MaybePeekLabel`, `MuDimension`, `NormalizeBibKey`, `Pair`, `PairList`, `ProcessPendingResources`, `ReadAlignmentTemplate`, `ReadParameters`, `Stringify`, `TokensI`, `TrimmedCommaList`, `convertLaTeXArgs`, `createXMRefs`, `decodeMathChar`, `dualize_arglist`.

## Rhai calls with no Perl counterpart (an empty Perl cell)

18 calls: `AppendSearchPath`, `Command`, `DeclareFontMapMultichar`, `LookupRegisterOrDefault`, `ParseXML`, `PrependSearchPath`, `ProcessKeyOptions`, `Today`, `neutralize_font`, `document.absorbProperty`, `document.generateID`, `document.insertXML`, `node.setContent`, `command.arg`, `command.args`, `command.current_dir`, `command.env`, `command.output`.

## Perl functions with no macro (compiled bindings call the Rust function named)

57 calls: `AssignDelcode` (`assign_delcode`), `AssignFloat` (`assign_value`), `AssignLCcode` (`assign_lccode`), `AssignMathcode` (`assign_mathcode`), `AssignSFcode` (`assign_sfcode`), `AssignUCcode` (`assign_uccode`), `AtBeginDocument` (`push_value`), `AtEndDocument` (`push_value`), `CC_ACTIVE`, `CC_ALIGN`, `CC_ARG`, `CC_BEGIN`, `CC_COMMENT`, `CC_CS`, `CC_END`, `CC_EOL`, `CC_ESCAPE`, `CC_IGNORE`, `CC_INVALID`, `CC_LETTER`, `CC_MARKER`, `CC_MATH`, `CC_OTHER`, `CC_PARAM`, `CC_SPACE`, `CC_SUB`, `CC_SUPER`, `CleanID` (`clean_id`), `CleanLabel` (`clean_label`), `EndSemiverbatim` (`end_semiverbatim`), `Equals`, `ExecuteOptions` (`execute_options`), `ExpandPartially` (`do_expand_partially`), `IsEmpty`, `LookupDelcode` (`lookup_delcode`), `LookupDimension` (`lookup_dimension_cs`), `LookupFloat` (`lookup_float`), `LookupLCcode` (`lookup_lccode`), `LookupMapping` (`lookup_mapping`), `LookupMappingKeys` (`with_mapping_keys`), `LookupMathcode` (`lookup_mathcode`), `LookupSFcode` (`lookup_sfcode`), `LookupUCcode` (`lookup_uccode`), `LookupValue` (`lookup_value`), `PassOptions` (`pass_options`), `PopValue` (`pop_value`), `ProgressSpindown` (`note_end`), `ProgressSpinup` (`note_begin`), `ProgressStep` (`progress_step`), `PushValue` (`push_value`), `SetCondition` (`set_condition`), `ShiftValue` (`shift_value`), `StartSemiverbatim` (`begin_semiverbatim`), `ToString`, `UTF`, `UnTeX` (`Tokens::untex`), `UnshiftValue` (`unshift_value`).
