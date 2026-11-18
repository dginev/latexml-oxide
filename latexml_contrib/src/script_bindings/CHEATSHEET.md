# Perl | Rust | Rhai: the binding dialects side by side

One row per call: its Perl LaTeXML spelling (a `Package.pm` function, or the Perl method or expression that does the
same), the Rust a compiled binding uses for it (the macro where there is one, then the function, method, type or
trait item the call reaches — the one the Rhai call itself calls), and the runtime `.rhai` call. **Generated** from
`api_doc.rs` with [`API.md`](https://github.com/dginev/latexml-oxide/blob/main/latexml_contrib/src/script_bindings/API.md), and checked with it on every test run (`api_reference_is_up_to_date`); regenerate
with `UPDATE_API_DOC=1 cargo test -p latexml_contrib --lib api_reference_is_up_to_date`.

Every Rust name is a rustdoc intra-doc link, so `cargo doc` fails on a renamed one; a private item is named with its
file. A Rhai helper Perl has no single call for shows the Perl that does the same, as "(Rhai helper; Perl: …)". A
deprecated alias follows the call that replaces it, marked `(deprecated: …)` (the aliases of `mod.rs`'s
`DEPRECATED_ALIASES`). An empty Rhai cell is a Perl call the dialect lacks, listed at the end. Each Rhai call's
arguments and meaning are in [`API.md`](https://github.com/dginev/latexml-oxide/blob/main/latexml_contrib/src/script_bindings/API.md); the
newcomer guide is [`RHAI_BINDINGS_GUIDE.md`](https://github.com/dginev/latexml-oxide/blob/main/docs/parity/RHAI_BINDINGS_GUIDE.md).

| Perl | Rust | Rhai |
|---|---|---|
| `!$v` | `perl_truthy` (`latexml_contrib/src/script_bindings/mod.rs`) | `!` |
| `$a ne $b` | [`impl PartialEq for str`](str) / `register_string_tests` (`latexml_contrib/src/script_bindings/engine.rs`) | `!=` |
| `$n->valueOf % $m` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) / `register_number_int_transition` (`latexml_contrib/src/script_bindings/values.rs`) | `%` |
| `$n->valueOf & $m` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) / `register_number_int_transition` (`latexml_contrib/src/script_bindings/values.rs`) | `&` |
| `$v->multiply($n)` | [`NumericOps::multiply`](latexml_core::common::numeric_ops::NumericOps::multiply) | `*` |
| `$n->valueOf ** $m` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) / `register_number_int_transition` (`latexml_contrib/src/script_bindings/values.rs`) | `**` |
| `$v->add($w)` | [`NumericOps::add`](latexml_core::common::numeric_ops::NumericOps::add) / [`Glue::add_glue`](latexml_core::common::glue::Glue::add_glue) | `+` |
| `$v->subtract($w)` | [`NumericOps::subtract`](latexml_core::common::numeric_ops::NumericOps::subtract) / [`NumericOps::negate`](latexml_core::common::numeric_ops::NumericOps::negate) | `-` |
| `$v->divide($n)` | [`NumericOps::divide`](latexml_core::common::numeric_ops::NumericOps::divide) | `/` |
| `$v->valueOf < $w->valueOf` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) | `<` |
| `$n->valueOf << $m` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) / `register_number_int_transition` (`latexml_contrib/src/script_bindings/values.rs`) | `<<` |
| `$v->valueOf <= $w->valueOf` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) | `<=` |
| `$a eq $b` | [`impl PartialEq for str`](str) / `register_string_tests` (`latexml_contrib/src/script_bindings/engine.rs`) | `==` |
| `$v->valueOf > $w->valueOf` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) | `>` |
| `$v->valueOf >= $w->valueOf` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) | `>=` |
| `$n->valueOf >> $m` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) / `register_number_int_transition` (`latexml_contrib/src/script_bindings/values.rs`) | `>>` |
| `AddToCounter` | [`AddToCounter!`](latexml_engine::AddToCounter!) / [`add_to_counter`](latexml_core::binding::counter::dialect::add_to_counter) | `AddToCounter` |
| `AddToMacro` | [`AddToMacro!`](latexml_engine::AddToMacro!) / [`add_to_macro`](latexml_core::binding::content::add_to_macro) | `AddToMacro` |
| `AfterAssignment` | [`after_assignment`](latexml_core::state::after_assignment) | `AfterAssignment` |
| (Rhai helper; Perl: `PushValue(SEARCHPATHS => $dir)`) | [`add_search_path`](latexml_core::state::add_search_path) | `AppendSearchPath` |
| `AssignCatcode` | [`AssignCatcode!`](latexml_engine::AssignCatcode!) / [`assign_catcode`](latexml_core::state::assign_catcode) | `AssignCatcode` |
| `AssignDelcode` | [`assign_delcode`](latexml_core::state::assign_delcode) | `AssignDelcode` |
| `AssignValue` | [`AssignValue!`](latexml_engine::AssignValue!) / [`assign_value`](latexml_core::state::assign_value) | `AssignFloat` |
| `AssignLCcode` | [`assign_lccode`](latexml_core::state::assign_lccode) | `AssignLCcode` |
| `AssignMapping` | [`AssignMapping!`](latexml_engine::AssignMapping!) / [`assign_mapping`](latexml_core::state::assign_mapping) | `AssignMapping` |
| `AssignMathcode` | [`assign_mathcode`](latexml_core::state::assign_mathcode) | `AssignMathcode` |
| `$STATE->assignMeaning` | [`AssignMeaning!`](latexml_engine::AssignMeaning!) / [`assign_meaning`](latexml_core::state::assign_meaning) | `AssignMeaning` |
| `AssignRegister` | [`AssignRegister!`](latexml_core::AssignRegister!) / [`assign_register_token`](latexml_core::state::assign_register_token) | `AssignRegister` |
| `AssignSFcode` | [`assign_sfcode`](latexml_core::state::assign_sfcode) | `AssignSFcode` |
| `AssignUCcode` | [`assign_uccode`](latexml_core::state::assign_uccode) | `AssignUCcode` |
| `AssignValue` | [`AssignValue!`](latexml_engine::AssignValue!) / [`assign_value`](latexml_core::state::assign_value) | `AssignValue` |
| `AssignValue` | [`AssignValue!`](latexml_engine::AssignValue!) / [`assign_value`](latexml_core::state::assign_value) | `AssignBool` (deprecated: `AssignValue(key, value[, scope])`) |
| `AssignValue` | [`AssignValue!`](latexml_engine::AssignValue!) / [`assign_value`](latexml_core::state::assign_value) | `AssignNumber` (deprecated: `AssignValue(key, value[, scope])`) |
| `AssignValue` | [`AssignValue!`](latexml_engine::AssignValue!) / [`assign_value`](latexml_core::state::assign_value) | `AssignString` (deprecated: `AssignValue(key, value[, scope])`) |
| `AssignValue` | [`AssignValue!`](latexml_engine::AssignValue!) / [`assign_value`](latexml_core::state::assign_value) | `assign_global` (deprecated: `AssignValue(key, value, "global")`) |
| `AssignValue` | [`AssignValue!`](latexml_engine::AssignValue!) / [`assign_value`](latexml_core::state::assign_value) | `assign_value` (deprecated: `AssignValue(key, value, "local")`) |
| `AtBeginDocument` | [`at_begin_document`](latexml_package::prelude::at_begin_document) | `AtBeginDocument` |
| `AtEndDocument` | [`at_end_document`](latexml_package::prelude::at_end_document) | `AtEndDocument` |
| `beginItemize` | [`BeginItemize!`](latexml_engine::BeginItemize!) / [`begin_itemize`](latexml_core::binding::counter::dialect::begin_itemize) | `BeginItemize` |
| `Box` | [`Tbox::new`](latexml_core::tbox::Tbox::new) | `Box` |
| `CC_ACTIVE` | [`Catcode::ACTIVE`](latexml_core::token::Catcode::ACTIVE) | `CC_ACTIVE` |
| `CC_ALIGN` | [`Catcode::ALIGN`](latexml_core::token::Catcode::ALIGN) | `CC_ALIGN` |
| `CC_ARG` | [`Catcode::ARG`](latexml_core::token::Catcode::ARG) | `CC_ARG` |
| `CC_BEGIN` | [`Catcode::BEGIN`](latexml_core::token::Catcode::BEGIN) | `CC_BEGIN` |
| `CC_COMMENT` | [`Catcode::COMMENT`](latexml_core::token::Catcode::COMMENT) | `CC_COMMENT` |
| `CC_CS` | [`Catcode::CS`](latexml_core::token::Catcode::CS) | `CC_CS` |
| `CC_END` | [`Catcode::END`](latexml_core::token::Catcode::END) | `CC_END` |
| `CC_EOL` | [`Catcode::EOL`](latexml_core::token::Catcode::EOL) | `CC_EOL` |
| `CC_ESCAPE` | [`Catcode::ESCAPE`](latexml_core::token::Catcode::ESCAPE) | `CC_ESCAPE` |
| `CC_IGNORE` | [`Catcode::IGNORE`](latexml_core::token::Catcode::IGNORE) | `CC_IGNORE` |
| `CC_INVALID` | [`Catcode::INVALID`](latexml_core::token::Catcode::INVALID) | `CC_INVALID` |
| `CC_LETTER` | [`Catcode::LETTER`](latexml_core::token::Catcode::LETTER) | `CC_LETTER` |
| `CC_MARKER` | [`Catcode::MARKER`](latexml_core::token::Catcode::MARKER) | `CC_MARKER` |
| `CC_MATH` | [`Catcode::MATH`](latexml_core::token::Catcode::MATH) | `CC_MATH` |
| `CC_OTHER` | [`Catcode::OTHER`](latexml_core::token::Catcode::OTHER) | `CC_OTHER` |
| `CC_PARAM` | [`Catcode::PARAM`](latexml_core::token::Catcode::PARAM) | `CC_PARAM` |
| `CC_SPACE` | [`Catcode::SPACE`](latexml_core::token::Catcode::SPACE) | `CC_SPACE` |
| `CC_SUB` | [`Catcode::SUB`](latexml_core::token::Catcode::SUB) | `CC_SUB` |
| `CC_SUPER` | [`Catcode::SUPER`](latexml_core::token::Catcode::SUPER) | `CC_SUPER` |
| `CheckOptions` | [`check_options`](latexml_core::binding::def::options::check_options) | `CheckOptions` |
| `CleanBibKey` | [`clean_bib_key`](latexml_core::common::cleaners::clean_bib_key) | `CleanBibKey` |
| `CleanClassName` | [`clean_class_name`](latexml_core::common::cleaners::clean_class_name) | `CleanClassName` |
| `CleanID` | [`clean_id`](latexml_core::common::cleaners::clean_id) | `CleanID` |
| `CleanIndexKey` | [`clean_index_key`](latexml_core::common::cleaners::clean_index_key) | `CleanIndexKey` |
| `CleanLabel` | [`clean_label`](latexml_core::common::cleaners::clean_label) | `CleanLabel` |
| `CleanURL` | [`clean_url`](latexml_core::common::cleaners::clean_url) | `CleanURL` |
| (Rhai helper; Perl: `system($program, @args)`) | [`Command::new`](std::process::Command::new) | `Command` |
| `ComposeURL` | [`compose_url`](latexml_core::common::cleaners::compose_url) | `ComposeURL` |
| `CounterValue` | [`CounterValue!`](latexml_engine::CounterValue!) / [`counter_value`](latexml_core::binding::counter::dialect::counter_value) | `CounterValue` |
| `Debug` | [`Debug!`](latexml_core::Debug!) / [`emit_record`](latexml_core::common::error::emit_record) | `Debug` |
| `DeclareFontMap` | [`DeclareFontMap!`](latexml_engine::DeclareFontMap!) | `DeclareFontMap` |
| (Rhai helper; Perl: `DeclareFontMap($name, [..., "ffi", ...])`) | [`DeclareFontMapMultichar!`](latexml_engine::DeclareFontMapMultichar!) / [`load_font_map`](latexml_core::binding::content::load_font_map) | `DeclareFontMapMultichar` |
| `DeclareOption` | [`DeclareOption!`](latexml_engine::DeclareOption!) | `DeclareOption` |
| `DefAccent` | [`DefAccent!`](latexml_engine::DefAccent!) | `DefAccent` |
| `DefAutoload` | [`def_autoload_file`](latexml_engine::tex::def_autoload_file) | `DefAutoload` |
| `DefColumnType` | [`DefColumnType!`](latexml_engine::DefColumnType!) / [`def_macro`](latexml_core::binding::def::dialect::def_macro) | `DefColumnType` |
| `DefConditional` | [`DefConditional!`](latexml_engine::DefConditional!) / [`def_conditional`](latexml_core::binding::def::dialect::def_conditional) | `DefConditional` |
| `DefConditionalI` | [`def_conditional`](latexml_core::binding::def::dialect::def_conditional) | `DefConditionalI` |
| `DefConstructor` | [`DefConstructor!`](latexml_engine::DefConstructor!) / [`def_constructor`](latexml_core::binding::def::dialect::def_constructor) | `DefConstructor` |
| `DefConstructorI` | [`def_constructor`](latexml_core::binding::def::dialect::def_constructor) | `DefConstructorI` |
| `DefEnvironment` | [`DefEnvironment!`](latexml_engine::DefEnvironment!) / [`def_environment`](latexml_core::binding::def::dialect::def_environment) | `DefEnvironment` |
| `DefEnvironmentI` | [`def_environment`](latexml_core::binding::def::dialect::def_environment) | `DefEnvironmentI` |
| `DefKeyVal` | [`DefKeyVal!`](latexml_engine::DefKeyVal!) / [`define`](latexml_core::keyval::define) | `DefKeyVal` |
| `DefLigature` | [`DefLigature!`](latexml_engine::DefLigature!) | `DefLigature` |
| `DefMacro` | [`DefMacro!`](latexml_engine::DefMacro!) / [`def_macro`](latexml_core::binding::def::dialect::def_macro) | `DefMacro` |
| `DefMacroI` | [`def_macro`](latexml_core::binding::def::dialect::def_macro) | `DefMacroI` |
| `DefMath` | [`DefMath!`](latexml_engine::DefMath!) / [`def_math`](latexml_core::binding::def::dialect::def_math) | `DefMath` |
| `DefMathI` | [`def_math`](latexml_core::binding::def::dialect::def_math) | `DefMathI` |
| `DefMathLigature` | `DefMathLigature!` (`latexml_engine/src/setup_binding_language.rs`) | `DefMathLigature` |
| `DefMathRewrite` | [`DefMathRewrite!`](latexml_engine::DefMathRewrite!) | `DefMathRewrite` |
| `DefParameterType` | [`DefParameterType!`](latexml_engine::DefParameterType!) / [`Parameter`](latexml_core::parameter::Parameter) | `DefParameterType` |
| `DefPrimitive` | [`DefPrimitive!`](latexml_engine::DefPrimitive!) / [`def_primitive`](latexml_core::binding::def::dialect::def_primitive) | `DefPrimitive` |
| `DefPrimitiveI` | [`def_primitive`](latexml_core::binding::def::dialect::def_primitive) | `DefPrimitiveI` |
| `DefRegister` | [`DefRegister!`](latexml_engine::DefRegister!) / [`def_register`](latexml_core::binding::def::dialect::def_register) | `DefRegister` |
| `DefRegisterI` | [`def_register`](latexml_core::binding::def::dialect::def_register) | `DefRegisterI` |
| `DefRewrite` | [`DefRewrite!`](latexml_engine::DefRewrite!) | `DefRewrite` |
| `Deprecated` | [`Warn!`](latexml_core::Warn!) | `Deprecated` |
| `Digest` | [`Digest!`](latexml_engine::Digest!) / [`digest`](latexml_core::stomach::digest) | `Digest` |
| `DigestIf` | [`DigestIf!`](latexml_engine::DigestIf!) / [`digest_if`](latexml_core::binding::content::digest_if) | `DigestIf` |
| `DigestLiteral` | [`digest_literal`](latexml_core::binding::content::digest_literal) | `DigestLiteral` |
| `DigestText` | [`DigestText!`](latexml_engine::DigestText!) / [`digest_text`](latexml_core::binding::content::digest_text) | `DigestText` |
| `Dimension` | [`Dimension!`](latexml_engine::Dimension!) | `Dimension` |
| `DisableKeyVal` | [`disable_keyval`](latexml_core::keyval::disable_keyval) | `DisableKeyVal` |
| `DocType` | [`RegisterDocumentNamespaces!`](latexml_engine::RegisterDocumentNamespaces!) / [`register_document_namespace`](latexml_core::common::model::register_document_namespace) | `DocType` |
| `EndSemiverbatim` | [`end_semiverbatim`](latexml_core::state::end_semiverbatim) | `EndSemiverbatim` |
| `Equals` | [`Tokens::equals`](latexml_core::tokens::Tokens::equals) / `equals` (`latexml_contrib/src/script_bindings/engine.rs`) | `Equals` |
| `Error` | [`Error!`](latexml_core::Error!) | `Error` |
| `ExecuteOptions` | [`execute_options`](latexml_core::binding::content::execute_options) | `ExecuteOptions` |
| `Expand` | [`Expand!`](latexml_engine::Expand!) / [`do_expand`](latexml_core::gullet::do_expand) | `Expand` |
| `ExpandPartially` | [`do_expand_partially`](latexml_core::gullet::do_expand_partially) | `ExpandPartially` |
| `Explode` | [`Explode!`](latexml_core::Explode!) | `Explode` |
| `ExplodeText` | [`ExplodeText!`](latexml_core::ExplodeText!) | `ExplodeText` |
| `Fatal` | [`Fatal!`](latexml_core::Fatal!) | `Fatal` |
| `FindFile` | [`FindFile!`](latexml_engine::FindFile!) / [`find_file`](latexml_core::binding::content::find_file) | `FindFile` |
| `FontDecode` | [`font_decode`](latexml_core::binding::content::font_decode) | `FontDecode` |
| `FontDecodeString` | [`font_decode_string`](latexml_core::binding::content::font_decode_string) | `FontDecodeString` |
| `GetKeyVal` | [`GetKeyVal!`](latexml_engine::GetKeyVal!) | `GetKeyVal` |
| `GetKeyVals` | [`GetKeyVals!`](latexml_engine::GetKeyVals!) | `GetKeyVals` |
| `Glue` | [`Glue!`](latexml_engine::Glue!) | `Glue` |
| `HasKeyVal` | [`has_keyval`](latexml_core::keyval::has_keyval) | `HasKeyVal` |
| `IfCondition` | [`if_condition_args`](latexml_core::binding::content::if_condition_args) | `IfCondition` |
| `Info` | [`Info!`](latexml_core::Info!) | `Info` |
| `Input` | [`Input!`](latexml_engine::Input!) / [`input`](latexml_core::binding::content::input) | `Input` |
| `InputDefinitions` | [`InputDefinitions!`](latexml_engine::InputDefinitions!) / [`input_definitions`](latexml_core::binding::content::input_definitions) | `InputDefinitions` |
| `Invocation` | [`Invocation!`](latexml_engine::Invocation!) / [`build_invocation`](latexml_core::binding::content::build_invocation) | `Invocation` |
| `IsDefined` | [`IsDefined!`](latexml_engine::IsDefined!) / [`is_defined_token`](latexml_core::binding::def::dialect::is_defined_token) | `IsDefined` |
| `IsEmpty` | `is_empty_thing` (`latexml_contrib/src/script_bindings/engine.rs`) | `IsEmpty` |
| `$LaTeXML::VERSION` | [`lookup_string`](latexml_core::state::lookup_string) | `LaTeXMLVersion` |
| `Let` | [`Let!`](latexml_engine::Let!) / [`let_definition`](latexml_core::state::let_definition) | `Let` |
| `List` | [`List`](latexml_core::list::List) / `list_of` (`latexml_contrib/src/script_bindings/engine.rs`) | `List` |
| `LoadClass` | [`LoadClass!`](latexml_engine::LoadClass!) / [`load_class`](latexml_core::binding::content::load_class) | `LoadClass` |
| `LoadFontMap` | [`load_font_map`](latexml_core::binding::content::load_font_map) | `LoadFontMap` |
| `LoadFormat` | `plain_dump_available` (`latexml_engine/src/tex.rs`) / `latex_dump_available` (`latexml_engine/src/latex.rs`) | `LoadFormat` |
| `LoadPool` | [`LoadPool!`](latexml_engine::LoadPool!) / [`input_definitions`](latexml_core::binding::content::input_definitions) | `LoadPool` |
| `LookupValue` | [`LookupBool!`](latexml_engine::LookupBool!) / [`lookup_bool`](latexml_core::state::lookup_bool) | `LookupBool` |
| `LookupCatcode` | [`LookupCatcode!`](latexml_engine::LookupCatcode!) / [`lookup_catcode`](latexml_core::state::lookup_catcode) | `LookupCatcode` |
| `LookupDefinition` | [`LookupDefinition!`](latexml_engine::LookupDefinition!) | `LookupDefinition` |
| `LookupDelcode` | [`lookup_delcode`](latexml_core::state::lookup_delcode) | `LookupDelcode` |
| `LookupDimension` | [`lookup_dimension_cs`](latexml_core::state::lookup_dimension_cs) | `LookupDimension` |
| `LookupValue` | [`lookup_float`](latexml_core::state::lookup_float) | `LookupFloat` |
| `LookupLCcode` | [`lookup_lccode`](latexml_core::state::lookup_lccode) | `LookupLCcode` |
| `LookupMapping` | [`lookup_mapping`](latexml_core::state::lookup_mapping) | `LookupMapping` |
| `LookupMappingKeys` | [`with_mapping_keys`](latexml_core::state::with_mapping_keys) | `LookupMappingKeys` |
| `LookupMathcode` | [`lookup_mathcode`](latexml_core::state::lookup_mathcode) | `LookupMathcode` |
| `LookupMeaning` | [`LookupMeaning!`](latexml_engine::LookupMeaning!) / [`lookup_meaning`](latexml_core::state::lookup_meaning) | `LookupMeaning` |
| `LookupValue` | [`LookupNumber!`](latexml_engine::LookupNumber!) / [`lookup_number`](latexml_core::state::lookup_number) | `LookupNumber` |
| `LookupRegister` | [`LookupRegister!`](latexml_engine::LookupRegister!) / [`lookup_register_definition`](latexml_core::state::lookup_register_definition) | `LookupRegister` |
| (Rhai helper; Perl: `IsDefined($cs) ? LookupRegister($cs) : Number(0)`) | [`LookupRegisterOrDefault!`](latexml_engine::LookupRegisterOrDefault!) / [`lookup_register`](latexml_core::state::lookup_register) | `LookupRegisterOrDefault` |
| `LookupSFcode` | [`lookup_sfcode`](latexml_core::state::lookup_sfcode) | `LookupSFcode` |
| `LookupValue` | [`LookupString!`](latexml_engine::LookupString!) / [`lookup_string`](latexml_core::state::lookup_string) | `LookupString` |
| `LookupValue` | [`LookupTokens!`](latexml_engine::LookupTokens!) / [`lookup_tokens`](latexml_core::state::lookup_tokens) | `LookupTokens` |
| `LookupUCcode` | [`lookup_uccode`](latexml_core::state::lookup_uccode) | `LookupUCcode` |
| `LookupValue` | [`lookup_value`](latexml_core::state::lookup_value) | `LookupValue` |
| `LookupValue` | [`lookup_value`](latexml_core::state::lookup_value) | `lookup_value` (deprecated: `LookupValue(key)`) |
| `MaybeNoteLabel` | [`maybe_note_label`](latexml_core::binding::counter::dialect::maybe_note_label) | `MaybeNoteLabel` |
| `MaybePeekLabel` | [`maybe_peek_label`](latexml_core::binding::counter::dialect::maybe_peek_label) | `MaybePeekLabel` |
| `MergeFont` | [`MergeFont!`](latexml_engine::MergeFont!) / [`merge_font`](latexml_core::binding::content::merge_font) | `MergeFont` |
| `MuGlue` | [`MuGlue!`](latexml_engine::MuGlue!) | `MuGlue` |
| `NewCounter` | [`NewCounter!`](latexml_engine::NewCounter!) / [`new_counter`](latexml_core::binding::counter::dialect::new_counter) | `NewCounter` |
| `NormalizeBibKey` | [`normalize_bib_key`](latexml_core::common::cleaners::normalize_bib_key) | `NormalizeBibKey` |
| `Note` | [`Note!`](latexml_core::Note!) | `Note` |
| `NoteLog` | [`NoteLog!`](latexml_core::NoteLog!) | `NoteLog` |
| `NoteSTDERR` | [`NoteSTDERR!`](latexml_core::NoteSTDERR!) | `NoteSTDERR` |
| `Number` | [`Number!`](latexml_core::Number!) | `Number` |
| (Rhai helper; Perl: `LaTeXML::Common::XML::Parser->new->parseChunk($xml)`) | [`parse_fragment`](latexml_core::common::xml::parse_fragment) | `ParseXML` |
| `PassOptions` | [`pass_options`](latexml_core::binding::content::pass_options) | `PassOptions` |
| `PopValue` | [`pop_value`](latexml_core::state::pop_value) | `PopValue` |
| (Rhai helper; Perl: `UnshiftValue(SEARCHPATHS => $dir)`) | [`search_paths_push_front`](latexml_core::state::search_paths_push_front) | `PrependSearchPath` |
| (Rhai helper; Perl: `ProcessOptions(keysets => [$keyset])`) | [`ProcessKeyOptions!`](latexml_engine::ProcessKeyOptions!) / [`process_key_options`](latexml_core::binding::content::process_key_options) | `ProcessKeyOptions` |
| `ProcessOptions` | [`ProcessOptions!`](latexml_engine::ProcessOptions!) / [`process_options`](latexml_core::binding::content::process_options) | `ProcessOptions` |
| `ProgressSpindown` | [`note_end`](latexml_core::common::error::note_end) | `ProgressSpindown` |
| `ProgressSpinup` | [`note_begin`](latexml_core::common::error::note_begin) | `ProgressSpinup` |
| `ProgressStep` | [`progress_step`](latexml_core::common::error::progress_step) | `ProgressStep` |
| `PushValue` | [`push_value`](latexml_core::state::push_value) | `PushValue` |
| `RawTeX` | [`RawTeX!`](latexml_engine::RawTeX!) / [`raw_tex`](latexml_core::stomach::raw_tex) | `RawTeX` |
| `$gullet->readArg` | [`read_arg`](latexml_core::gullet::read_arg) | `ReadArg` |
| `$gullet->readOptional` | [`read_optional`](latexml_core::gullet::read_optional) | `ReadOptional` |
| `$gullet->readToken` | [`read_token`](latexml_core::gullet::read_token) | `ReadToken` |
| `$gullet->readUntil` | [`read_until`](latexml_core::gullet::read_until) | `ReadUntil` |
| `RefCurrentID` | [`RefCurrentID!`](latexml_engine::RefCurrentID!) / [`ref_current_id`](latexml_core::binding::counter::dialect::ref_current_id) | `RefCurrentID` |
| `RefStepCounter` | [`RefStepCounter!`](latexml_engine::RefStepCounter!) / [`ref_step_counter_by_meaning`](latexml_core::binding::counter::dialect::ref_step_counter_by_meaning) | `RefStepCounter` |
| `RefStepID` | [`RefStepID!`](latexml_engine::RefStepID!) / [`ref_step_id`](latexml_core::binding::counter::dialect::ref_step_id) | `RefStepID` |
| `RegisterDocumentNamespace` | [`RegisterDocumentNamespace!`](latexml_engine::RegisterDocumentNamespace!) / [`register_document_namespace`](latexml_core::common::model::register_document_namespace) | `RegisterDocumentNamespace` |
| `RegisterNamespace` | [`RegisterNamespace!`](latexml_engine::RegisterNamespace!) / [`register_namespace`](latexml_core::common::model::register_namespace) | `RegisterNamespace` |
| `RelaxNGSchema` | [`RelaxNGSchema!`](latexml_engine::RelaxNGSchema!) / [`select_relaxng_schema`](latexml_core::binding::content::select_relaxng_schema) | `RelaxNGSchema` |
| `RequirePackage` | [`RequirePackage!`](latexml_engine::RequirePackage!) / [`require_package`](latexml_core::binding::content::require_package) | `RequirePackage` |
| `RequireResource` | [`RequireResource!`](latexml_engine::RequireResource!) / [`require_resource`](latexml_core::binding::content::require_resource) | `RequireResource` |
| `ResetCounter` | [`ResetCounter!`](latexml_engine::ResetCounter!) / [`reset_counter`](latexml_core::binding::counter::dialect::reset_counter) | `ResetCounter` |
| `Revert` | [`Revert!`](latexml_engine::Revert!) / [`Digested::revert`](latexml_core::digested::Digested::revert) | `Revert` |
| `Roman` | [`Roman!`](latexml_core::Roman!) | `Roman` |
| `SetCondition` | [`set_condition`](latexml_core::binding::content::set_condition) | `SetCondition` |
| `SetCounter` | [`SetCounter!`](latexml_core::SetCounter!) / [`set_counter`](latexml_core::binding::counter::dialect::set_counter) | `SetCounter` |
| `ShiftValue` | [`shift_value`](latexml_core::state::shift_value) | `ShiftValue` |
| `$gullet->skipSpaces` | [`skip_spaces`](latexml_core::gullet::skip_spaces) | `SkipSpaces` |
| `StartSemiverbatim` | [`begin_semiverbatim`](latexml_core::state::begin_semiverbatim) | `StartSemiverbatim` |
| `StepCounter` | [`StepCounter!`](latexml_engine::StepCounter!) / [`step_counter`](latexml_core::binding::counter::dialect::step_counter) | `StepCounter` |
| `Stringify` | [`Object::stringify`](latexml_core::common::object::Object::stringify) | `Stringify` |
| `T_ACTIVE` | [`T_ACTIVE!`](latexml_core::T_ACTIVE!) | `T_ACTIVE` |
| `T_ALIGN` | [`T_ALIGN!`](latexml_core::T_ALIGN!) | `T_ALIGN` |
| `T_ARG` | [`T_ARG!`](latexml_core::T_ARG!) | `T_ARG` |
| `T_BEGIN` | [`T_BEGIN!`](latexml_core::T_BEGIN!) | `T_BEGIN` |
| `T_COMMENT` | [`T_COMMENT!`](latexml_core::T_COMMENT!) | `T_COMMENT` |
| `T_CR` | [`T_CR!`](latexml_core::T_CR!) | `T_CR` |
| `T_CS` | [`T_CS!`](latexml_core::T_CS!) | `T_CS` |
| `T_END` | [`T_END!`](latexml_core::T_END!) | `T_END` |
| `T_LETTER` | [`T_LETTER!`](latexml_core::T_LETTER!) | `T_LETTER` |
| `T_MARKER` | [`T_MARKER!`](latexml_core::T_MARKER!) | `T_MARKER` |
| `T_MATH` | [`T_MATH!`](latexml_core::T_MATH!) | `T_MATH` |
| `T_OTHER` | [`T_OTHER!`](latexml_core::T_OTHER!) | `T_OTHER` |
| `T_PARAM` | [`T_PARAM!`](latexml_core::T_PARAM!) | `T_PARAM` |
| `T_SPACE` | [`T_SPACE!`](latexml_core::T_SPACE!) | `T_SPACE` |
| `T_SUB` | [`T_SUB!`](latexml_core::T_SUB!) | `T_SUB` |
| `T_SUPER` | [`T_SUPER!`](latexml_core::T_SUPER!) | `T_SUPER` |
| `Tag` | [`Tag!`](latexml_engine::Tag!) / [`install_tag`](latexml_core::binding::content::install_tag) | `Tag` |
| `Digest(TokenizeInternal($tex))` | [`TeX!`](latexml_engine::TeX!) / [`digest`](latexml_core::stomach::digest) | `TeX` |
| `$box->toAttribute` | [`Digested::to_attribute`](latexml_core::digested::Digested::to_attribute) | `ToAttribute` |
| `ToString` | [`impl Display for Digested`](latexml_core::digested::Digested) / `dynamic_to_string` (`latexml_contrib/src/script_bindings/mod.rs`) | `ToString` |
| (Rhai helper; Perl: `Expand(T_CS('\\today'))`) | [`Today!`](latexml_engine::Today!) / [`today`](latexml_engine::base_utilities::today) | `Today` |
| `Token` | [`Token!`](latexml_core::Token!) | `Token` |
| `Tokenize` | [`Tokenize!`](latexml_engine::Tokenize!) / [`tokenize`](latexml_core::mouth::tokenize) | `Tokenize` |
| `TokenizeInternal` | [`TokenizeInternal!`](latexml_engine::TokenizeInternal!) / [`tokenize_internal`](latexml_core::mouth::tokenize_internal) | `TokenizeInternal` |
| `Tokens` | [`Tokens!`](latexml_core::Tokens!) | `Tokens` |
| `TrimmedCommaList` | [`trimmed_comma_list`](latexml_core::common::cleaners::trimmed_comma_list) | `TrimmedCommaList` |
| `UTF` | [`from_u32`](char::from_u32) | `UTF` |
| `UnTeX` | [`Tokens::untex`](latexml_core::tokens::Tokens::untex) | `UnTeX` |
| `UnshiftValue` | [`unshift_value`](latexml_core::state::unshift_value) | `UnshiftValue` |
| `Warn` | [`Warn!`](latexml_core::Warn!) | `Warn` |
| `XEquals` | [`XEquals!`](latexml_engine::XEquals!) / [`x_equals`](latexml_core::state::x_equals) | `XEquals` |
| `$n->valueOf ^ $m` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) / `register_number_int_transition` (`latexml_contrib/src/script_bindings/values.rs`) | `^` |
| `$v->absolute` | [`NumericOps::absolute`](latexml_core::common::numeric_ops::NumericOps::absolute) | `absolute` |
| `$v->add` | [`NumericOps::add`](latexml_core::common::numeric_ops::NumericOps::add) / [`Glue::add_glue`](latexml_core::common::glue::Glue::add_glue) | `add` |
| `defined $v` | [`Dynamic::is_unit`](rhai::Dynamic::is_unit) | `defined` |
| `$v->divide` | [`NumericOps::divide`](latexml_core::common::numeric_ops::NumericOps::divide) | `divide` |
| `$v->divideround` | [`NumericOps::divideround`](latexml_core::common::numeric_ops::NumericOps::divideround) | `divideround` |
| `$v->getShrink` | [`Glue::minus`](latexml_core::common::glue::Glue::minus) | `getShrink` |
| `$v->getShrinkOrder` | [`Glue::mfill`](latexml_core::common::glue::Glue::mfill) | `getShrinkOrder` |
| `$v->getStretch` | [`Glue::plus`](latexml_core::common::glue::Glue::plus) | `getStretch` |
| `$v->getStretchOrder` | [`Glue::pfill`](latexml_core::common::glue::Glue::pfill) | `getStretchOrder` |
| `$v->larger` | [`NumericOps::larger`](latexml_core::common::numeric_ops::NumericOps::larger) | `larger` |
| `$v->multiply` | [`NumericOps::multiply`](latexml_core::common::numeric_ops::NumericOps::multiply) | `multiply` |
| `$v->negate` | [`NumericOps::negate`](latexml_core::common::numeric_ops::NumericOps::negate) | `negate` |
| (Rhai helper; Perl: `neutralizeFont()`) | [`neutralize_font`](latexml_engine::base_utilities::neutralize_font) | `neutralize_font` |
| `$v->ptValue` | [`NumericOps::pt_value`](latexml_core::common::numeric_ops::NumericOps::pt_value) | `ptValue` |
| `$v->pxValue` | [`NumericOps::px_value`](latexml_core::common::numeric_ops::NumericOps::px_value) | `pxValue` |
| `roman` | [`roman!`](latexml_core::roman!) | `roman` |
| `$v->sign` | [`NumericOps::sign`](latexml_core::common::numeric_ops::NumericOps::sign) | `sign` |
| `$v->smaller` | [`NumericOps::smaller`](latexml_core::common::numeric_ops::NumericOps::smaller) | `smaller` |
| `$v->spValue` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) | `spValue` |
| `$v->subtract` | [`NumericOps::subtract`](latexml_core::common::numeric_ops::NumericOps::subtract) | `subtract` |
| `$v->toAttribute` | [`NumericOps::to_attribute`](latexml_core::common::numeric_ops::NumericOps::to_attribute) | `toAttribute` |
| `$v->valueOf` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) | `valueOf` |
| `$whatsit` | [`Whatsit`](latexml_core::whatsit::Whatsit) | `whatsit` (deprecated: the hook's whatsit argument) |
| `$n->valueOf \| $m` | [`NumericOps::value_of`](latexml_core::common::numeric_ops::NumericOps::value_of) / `register_number_int_transition` (`latexml_contrib/src/script_bindings/values.rs`) | `\|` |
| `$document->absorb` | [`Document::absorb`](latexml_core::document::Document::absorb) | `document.absorb` |
| (Rhai helper; Perl: `$document->absorb($props{$key})`) | [`Document::absorb`](latexml_core::document::Document::absorb) | `document.absorbProperty` |
| `$document->absorb` | [`Document::absorb_string`](latexml_core::document::Document::absorb_string) | `document.absorbString` |
| `$document->addClass` | [`Document::add_class`](latexml_core::document::Document::add_class) | `document.addClass` |
| `$document->appendClone` | [`Document::append_clone`](latexml_core::document::Document::append_clone) | `document.appendClone` |
| `$document->canContain` | [`can_contain`](latexml_core::document::can_contain) | `document.canContain` |
| `$document->closeElement` | [`Document::close_element`](latexml_core::document::Document::close_element) | `document.closeElement` |
| `$document->closeElementAt` | [`Document::close_element_at`](latexml_core::document::Document::close_element_at) | `document.closeElementAt` |
| `$document->findnode` | [`Document::findnode`](latexml_core::document::Document::findnode) | `document.findnode` |
| `$document->findnodes` | [`Document::findnodes`](latexml_core::document::Document::findnodes) | `document.findnodes` |
| `$document->floatToElement` | [`Document::float_to_element`](latexml_core::document::Document::float_to_element) | `document.floatToElement` |
| `GenerateID($document, $node, undef, $prefix)` | [`Document::generate_id`](latexml_core::document::Document::generate_id) | `document.generateID` |
| `$document->getElement` | [`Document::get_element`](latexml_core::document::Document::get_element) | `document.getElement` |
| `$document->getNode` | [`Document::get_node`](latexml_core::document::Document::get_node) | `document.getNode` |
| `$whatsit->getProperties` | [`BoxOps::get_properties`](latexml_core::BoxOps::get_properties) | `document.getProperties` (deprecated: the constructor body's trailing properties map) |
| `$whatsit->getProperty` | [`BoxOps::get_property`](latexml_core::BoxOps::get_property) | `document.getProperty` (deprecated: the constructor body's trailing properties map) |
| `$document->insertElement` | [`Document::insert_element`](latexml_core::document::Document::insert_element) | `document.insertElement` |
| `$document->insertPI` | [`Document::insert_pi`](latexml_core::document::Document::insert_pi) | `document.insertPI` |
| (Rhai helper; Perl: `$document->appendClone($document->getNode, @nodes)`) | [`Document::insert_nodes`](latexml_core::document::Document::insert_nodes) | `document.insertXML` |
| `$document->maybeCloseElement` | [`Document::maybe_close_element`](latexml_core::document::Document::maybe_close_element) | `document.maybeCloseElement` |
| `$document->openElement` | [`Document::open_element`](latexml_core::document::Document::open_element) | `document.openElement` |
| `$document->openElementAt` | [`Document::open_element_at`](latexml_core::document::Document::open_element_at) | `document.openElementAt` |
| `$document->removeNode` | [`Document::remove_node`](latexml_core::document::Document::remove_node) | `document.removeNode` |
| `$document->renameNode` | [`Document::rename_node`](latexml_core::document::Document::rename_node) | `document.renameNode` |
| `$document->replaceNode` | [`Document::replace_node`](latexml_core::document::Document::replace_node) | `document.replaceNode` |
| `$document->setAttribute` | [`Document::set_attribute`](latexml_core::document::Document::set_attribute) | `document.setAttribute` |
| `$document->setNode` | [`Document::set_node`](latexml_core::document::Document::set_node) | `document.setNode` |
| `$document->unwrapNodes` | [`Document::unwrap_nodes`](latexml_core::document::Document::unwrap_nodes) | `document.unwrapNodes` |
| `$document->wrapNodes` | [`Document::wrap_nodes`](latexml_core::document::Document::wrap_nodes) | `document.wrapNodes` |
| `$node->childNodes` | [`Node::get_child_nodes`](libxml::tree::Node::get_child_nodes) | `node.children` |
| `$node->textContent` | [`Node::get_content`](libxml::tree::Node::get_content) | `node.content` |
| `$node->firstChild` | [`Node::get_first_child`](libxml::tree::Node::get_first_child) | `node.firstChild` |
| `$node->getAttribute` | [`get_node_attribute`](latexml_core::common::model::get_node_attribute) | `node.getAttribute` |
| `$node->hasAttribute` | [`get_node_attribute`](latexml_core::common::model::get_node_attribute) | `node.hasAttribute` |
| `$node->localname` | [`Node::get_name`](libxml::tree::Node::get_name) | `node.name` |
| `$node->nextSibling` | [`Node::get_next_sibling`](libxml::tree::Node::get_next_sibling) | `node.nextSibling` |
| `$node->parentNode` | [`Node::get_parent`](libxml::tree::Node::get_parent) | `node.parent` |
| `$node->previousSibling` | [`Node::get_prev_sibling`](libxml::tree::Node::get_prev_sibling) | `node.prevSibling` |
| `$document->getNodeQName($node)` | [`with_node_qname`](latexml_core::common::model::with_node_qname) | `node.qname` |
| `$node->removeAttribute` | [`remove_node_attribute`](latexml_core::common::model::remove_node_attribute) | `node.removeAttribute` |
| `$node->setAttribute` | [`Node::set_attribute`](libxml::tree::Node::set_attribute) | `node.setAttribute` |
| (Rhai helper; Perl: `$node->removeChildNodes; $node->appendText($text)`) | [`Node::set_content`](libxml::tree::Node::set_content) | `node.setContent` |
| `$node->toString` | [`Document::node_to_string`](libxml::tree::Document::node_to_string) | `node.toString` |
| `$node->unbindNode` | [`Node::unlink`](libxml::tree::Node::unlink) | `node.unlink` |
| `$whatsit->getArg` | [`Whatsit::get_arg`](latexml_core::whatsit::Whatsit::get_arg) | `whatsit.getArg` |
| `$whatsit->getArg` | [`Whatsit::get_arg`](latexml_core::whatsit::Whatsit::get_arg) / [`Digested::untex`](latexml_core::digested::Digested::untex) | `whatsit.argString` (deprecated: `whatsit.getArg(n)`) |
| `$whatsit->getFont` | [`BoxOps::get_font`](latexml_core::BoxOps::get_font) | `whatsit.getFont` |
| `$whatsit->getProperties` | [`BoxOps::get_properties`](latexml_core::BoxOps::get_properties) | `whatsit.getProperties` |
| `$whatsit->getProperty` | [`BoxOps::get_property`](latexml_core::BoxOps::get_property) | `whatsit.getProperty` |
| `$whatsit->getProperty` | [`BoxOps::get_property_string`](latexml_core::BoxOps::get_property_string) | `whatsit.propertyString` (deprecated: `whatsit.getProperty(key)`) |
| `$whatsit->setProperty` | [`BoxOps::set_property`](latexml_core::BoxOps::set_property) | `whatsit.setProperty` |
| `ToString($whatsit)` | [`Whatsit::revert`](latexml_core::whatsit::Whatsit::revert) / `whatsit_string` (`latexml_contrib/src/script_bindings/mod.rs`) | `whatsit.to_string` |
| (Rhai helper; Perl: `ToString($definition) eq ''`) | `register_string_length` (`latexml_contrib/src/script_bindings/engine.rs`) | `definition.is_empty` |
| (Rhai helper; Perl: `length(ToString($definition))`) | `register_string_length` (`latexml_contrib/src/script_bindings/engine.rs`) | `definition.len` |
| `push(@{$$defn{afterConstruct}}, …)` | [`Constructor::after_construct`](latexml_core::definition::constructor::Constructor::after_construct) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.pushAfterConstruct` |
| `push(@{$$defn{afterDigest}}, …)` | [`Constructor::after_digest`](latexml_core::definition::constructor::Constructor::after_digest) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.pushAfterDigest` |
| `push(@{$$defn{afterDigestBody}}, …)` | [`Constructor::after_digest_body`](latexml_core::definition::constructor::Constructor::after_digest_body) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.pushAfterDigestBody` |
| `push(@{$$defn{beforeConstruct}}, …)` | [`Constructor::before_construct`](latexml_core::definition::constructor::Constructor::before_construct) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.pushBeforeConstruct` |
| `push(@{$$defn{beforeDigest}}, …)` | [`Constructor::before_digest`](latexml_core::definition::constructor::Constructor::before_digest) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.pushBeforeDigest` |
| (Rhai helper; Perl: `ToString($definition)`) | `definition_string` (`latexml_contrib/src/script_bindings/mod.rs`) | `definition.to_debug` |
| `ToString($definition)` | `definition_string` (`latexml_contrib/src/script_bindings/mod.rs`) | `definition.to_string` |
| `unshift(@{$$defn{afterConstruct}}, …)` | [`Constructor::after_construct`](latexml_core::definition::constructor::Constructor::after_construct) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.unshiftAfterConstruct` |
| `unshift(@{$$defn{afterDigest}}, …)` | [`Constructor::after_digest`](latexml_core::definition::constructor::Constructor::after_digest) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.unshiftAfterDigest` |
| `unshift(@{$$defn{afterDigestBody}}, …)` | [`Constructor::after_digest_body`](latexml_core::definition::constructor::Constructor::after_digest_body) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.unshiftAfterDigestBody` |
| `unshift(@{$$defn{beforeConstruct}}, …)` | [`Constructor::before_construct`](latexml_core::definition::constructor::Constructor::before_construct) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.unshiftBeforeConstruct` |
| `unshift(@{$$defn{beforeDigest}}, …)` | [`Constructor::before_digest`](latexml_core::definition::constructor::Constructor::before_digest) / `push_definition_hook` (`latexml_contrib/src/script_bindings/wire.rs`) | `definition.unshiftBeforeDigest` |
| `ReadParameters($gullet, $spec)` | [`parse_parameters`](latexml_core::common::def_parser::parse_parameters) / [`Parameters::read_arguments`](latexml_core::parameter::Parameters::read_arguments) | `gullet.ReadParameters` |
| `$gullet->readArg` | [`read_arg`](latexml_core::gullet::read_arg) | `gullet.readArg` |
| `$gullet->readOptional` | [`read_optional`](latexml_core::gullet::read_optional) | `gullet.readOptional` |
| `$gullet->readToken` | [`read_token`](latexml_core::gullet::read_token) | `gullet.readToken` |
| `$gullet->readUntil` | [`read_until`](latexml_core::gullet::read_until) | `gullet.readUntil` |
| `$gullet->skipSpaces` | [`skip_spaces`](latexml_core::gullet::skip_spaces) | `gullet.skipSpaces` |
| `$stomach->beginMode` | [`begin_mode_opt`](latexml_core::stomach::begin_mode_opt) | `stomach.beginMode` |
| `$stomach->begingroup` | [`begingroup`](latexml_core::stomach::begingroup) | `stomach.begingroup` |
| `$stomach->bgroup` | [`bgroup`](latexml_core::stomach::bgroup) | `stomach.bgroup` |
| `$stomach->egroup` | [`egroup`](latexml_core::stomach::egroup) | `stomach.egroup` |
| `$stomach->endMode` | [`end_mode_opt`](latexml_core::stomach::end_mode_opt) | `stomach.endMode` |
| `$stomach->endgroup` | [`endgroup`](latexml_core::stomach::endgroup) | `stomach.endgroup` |
| `$stomach->enterHorizontal` | [`enter_horizontal`](latexml_core::stomach::enter_horizontal) | `stomach.enterHorizontal` |
| `$stomach->getGullet` | [`gullet`](latexml_core::gullet) | `stomach.getGullet` |
| `$stomach->leaveHorizontal` | [`leave_horizontal`](latexml_core::stomach::leave_horizontal) | `stomach.leaveHorizontal` |
| `$font->getEncoding` | [`Font::get_encoding`](latexml_core::common::font::Font::get_encoding) | `font.getEncoding` |
| `$font->getFamily` | [`Font::get_family`](latexml_core::common::font::Font::get_family) | `font.getFamily` |
| `$font->getLanguage` | [`Font::get_language`](latexml_core::common::font::Font::get_language) | `font.getLanguage` |
| `$font->getSeries` | [`Font::get_series`](latexml_core::common::font::Font::get_series) | `font.getSeries` |
| `$font->getShape` | [`Font::get_shape`](latexml_core::common::font::Font::get_shape) | `font.getShape` |
| `$font->getSize` | [`Font::get_size`](latexml_core::common::font::Font::get_size) | `font.getSize` |
| `$v->toString` | [`impl Display for Number`](latexml_core::common::number::Number) | `number.toString` |
| (Rhai helper; Perl: `$v->toString`) | [`impl Display for Number`](latexml_core::common::number::Number) | `number.to_debug` |
| `$v->toString` | [`impl Display for Number`](latexml_core::common::number::Number) | `number.to_string` |
| `$v->toString` | [`impl Display for Dimension`](latexml_core::common::dimension::Dimension) | `dimension.toString` |
| (Rhai helper; Perl: `$v->toString`) | [`impl Display for Dimension`](latexml_core::common::dimension::Dimension) | `dimension.to_debug` |
| `$v->toString` | [`impl Display for Dimension`](latexml_core::common::dimension::Dimension) | `dimension.to_string` |
| `$v->toString` | [`impl Display for Glue`](latexml_core::common::glue::Glue) | `glue.toString` |
| (Rhai helper; Perl: `$v->toString`) | [`impl Display for Glue`](latexml_core::common::glue::Glue) | `glue.to_debug` |
| `$v->toString` | [`impl Display for Glue`](latexml_core::common::glue::Glue) | `glue.to_string` |
| `$v->toString` | [`impl Display for MuGlue`](latexml_core::common::muglue::MuGlue) | `muglue.toString` |
| (Rhai helper; Perl: `$v->toString`) | [`impl Display for MuGlue`](latexml_core::common::muglue::MuGlue) | `muglue.to_debug` |
| `$v->toString` | [`impl Display for MuGlue`](latexml_core::common::muglue::MuGlue) | `muglue.to_string` |
| (Rhai helper; Perl: `push(@command, $arg)`) | [`Command::arg`](std::process::Command::arg) | `command.arg` |
| (Rhai helper; Perl: `push(@command, @args)`) | [`Command::args`](std::process::Command::args) | `command.args` |
| (Rhai helper; Perl: `chdir($dir)`) | [`Command::current_dir`](std::process::Command::current_dir) | `command.current_dir` |
| (Rhai helper; Perl: `local $ENV{$key} = $value`) | [`Command::env`](std::process::Command::env) | `command.env` |
| (Rhai helper; Perl: `qx($command)`) | [`Command::output`](std::process::Command::output) | `command.output` |
| `Color` | [`Color`](latexml_core::common::color::Color) |  |
| `DefColor` | [`def_color`](latexml_core::binding::content::def_color) |  |
| `DefColorModel` | [`def_color_model`](latexml_core::binding::content::def_color_model) |  |
| `DefExpandable` | [`DefMacro!`](latexml_engine::DefMacro!) / [`def_macro`](latexml_core::binding::def::dialect::def_macro) | `DefMacro(proto, expansion, opts)` |
| `Float` | [`Float`](latexml_core::common::float::Float) | a Rhai float (`1.5`) |
| `GenerateID` | [`Document::generate_id`](latexml_core::document::Document::generate_id) | `document.generateID(node, prefix)` |
| `InputContent` | [`input_content`](latexml_core::binding::content::input_content) |  |
| `InstallDefinition` | [`InstallDefinition!`](latexml_engine::InstallDefinition!) / [`install_definition`](latexml_core::state::install_definition) |  |
| `LookupColor` | [`LookupColor!`](latexml_engine::LookupColor!) / [`lookup_color`](latexml_package::package::color_sty::lookup_color) |  |
| `MuDimension` | [`MuDimension`](latexml_core::common::mudimension::MuDimension) |  |
| `Pair` | [`Pair::new`](latexml_core::common::pair::Pair::new) |  |
| `PairList` | [`Vec<Pair>`](latexml_core::common::pair::Pair) |  |
| `ProcessPendingResources` | [`Document::process_pending_resources`](latexml_core::document::Document::process_pending_resources) |  |
| `ReadAlignmentTemplate` | [`read_alignment_template`](latexml_core::alignment::read_alignment_template) |  |
| `TokensI` | [`Tokens::new`](latexml_core::tokens::Tokens::new) | `Tokens(t1, t2, …)` |
| `convertLaTeXArgs` | [`convert_latex_args`](latexml_core::binding::content::convert_latex_args) |  |
| `createXMRefs` | [`i_xmref`](latexml_package::xmath_helpers::i_xmref) / `rearrange_ams_split` (`latexml_package/src/package/amsmath_sty.rs`) |  |
| `decodeMathChar` | [`decode_math_char`](latexml_core::common::mathchar::decode_math_char) |  |
| `dualize_arglist` | [`dualize_arglist`](latexml_core::binding::def::dialect::dualize_arglist) |  |

## Perl calls the dialect lacks (an empty Rhai cell)

15 calls, each with the reason:

- `Color`: a Color value handle the dialect lacks.
- `DefColor`: takes a Color value handle the dialect lacks.
- `DefColorModel`: its conversion closures take Color value handles the dialect lacks.
- `InputContent`: not yet exposed; a binding reaches it as `RawTeX("\\input{file}")`.
- `InstallDefinition`: needs a Definition object; the dialect's definers install their own.
- `LookupColor`: returns a Color value handle the dialect lacks.
- `MuDimension`: a MuDimension value handle the dialect lacks.
- `Pair`: a Pair value handle the dialect lacks.
- `PairList`: a PairList value handle the dialect lacks.
- `ProcessPendingResources`: run by the schema as `ltx:document` opens (Base_Schema.pool.ltxml:61, `afterOpen`); no binding calls it.
- `ReadAlignmentTemplate`: returns an alignment template, a handle the dialect lacks.
- `convertLaTeXArgs`: internal to `\newcommand`'s definers; a binding writes its parameter spec itself.
- `createXMRefs`: an XMRef per math node or argument, inside `DefMath`'s XMDual (Package.pm:1544), which the dialect's `DefMath` builds.
- `decodeMathChar`: returns Perl's list (glyph, font, reversion, properties); not yet exposed as a Rhai value.
- `dualize_arglist`: internal to `DefMath`'s XMDual, which the dialect's `DefMath` builds.
