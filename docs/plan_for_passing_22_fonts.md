# Plan for Passing 22_fonts Test Suite in Rust

> **Revision note (2026-03-18):** Items marked [x] below were implemented by Gemini 3 but most code changes in the math parser, dialect.rs, and document.rs were **reverted** because they regressed esint (257→476 diffs) and mathbbol (168→199 diffs). Only two changes were kept: `scriptpos`/`mathstyle` propagation in `decode_math_char_for_stomach` (mathchar.rs) and `IN_MATH_DISPLAY` state in `set_mode` (stomach.rs). The items marked [x] represent completed analysis, not merged code.

This document outlines the detailed steps required to achieve full parity with Perl LaTeXML for the `22_fonts` test suite, strictly following original Perl semantics.

## 1. Fix Math Tokens & Properties
**Affected Tests**: `mathbbol_test`, `plainfonts_test`, `esint_test`

### 1.1. Propagate Missing Attributes in `def_math_dual`
- [x] **Location**: `latexml_core/src/binding/def/dialect.rs`
- [x] **Issue**: When `DefMath` creates an `XMDual` (via `def_math_dual`), it uses a `content_closure` to generate the content token. This closure currently ignores `name`, `meaning`, `omcd`, and `mathstyle` attributes.
- [x] **Change**: Update the `content_closure` in `def_math_dual` to extract and inject the following attributes from the `props` map:
    *   `name`
    *   `meaning`
    *   `omcd`
    *   `decl_id`
    *   `mathstyle`
- [x] **Perl Parity**: Matches `$cons_attr` in `LaTeXML::Package::defmath_dual`.

### 1.2. Preserve Semantic Properties for Decoded Math Characters
- [x] **Location**: `latexml_core/src/common/mathchar.rs` and `latexml_core/src/stomach.rs`
- [x] **Issue**: Symbols decoded via `\fam` (e.g., `\cal a` -> `⊣`) are losing their `meaning` and `role` during digestion or serialization.
- [x] **Change**: 
    1.  Ensure `decode_math_char_for_stomach` correctly places `meaning` and `role` into the `properties` map of the `Tbox`.
    2.  Verify `stomach::invoke_token_simple` does not overwrite these properties if they are already present on a digested math character.
    3.  Ensure `Tbox` serialization in `document.rs` or equivalent correctly maps the `"meaning"` property to the XML attribute.
- [x] **Perl Parity**: Matches property preservation in `LaTeXML::Core::Stomach::invokeToken_simple` and `LaTeXML::Package::DecodeMathChar`.

### 1.3. Fix `mathstyle` Attribute Propagation
- [x] **Location**: `latexml_core/src/common/mathchar.rs`
- [x] **Issue**: `\landupint` and other variable-sized operators are missing `mathstyle="display"`.
- [x] **Change**: Ensure `resolve_style_props()` correctly populates the `mathstyle` field in `MathCharProps` and that this is serialized as an attribute on the resulting `XMTok`.
- [x] **Perl Parity**: Matches `dynamic_mathstyle` logic in `LaTeXML::Package::defmath_cons`.

---

## 2. Fix Math Parser Semantics
**Affected Tests**: `mathaccents_test`, `fonts_test`

### 2.1. Map `delimited-` to Meaning
- [x] **Location**: `latexml_math_parser/src/semantics.rs`
- [x] **Issue**: The parser creates tokens like `delimited-[]` as text content (`<XMTok>delimited-[]</XMTok>`). In Perl, these are semantic meanings (`<XMTok meaning="delimited-[]"/>`).
- [x] **Change**: Update `xnew(text: String)` in `semantics.rs`. If `text` starts with `"delimited-"`, assign it to `XProps.meaning` and leave `XProps.content` as `None`.
- [x] **Perl Parity**: Matches `MathParser.pm` behavior where delimiters generate virtual operators with `meaning`.

### 2.2. Handle Empty Arguments with `absent()`
- [x] **Location**: `latexml_math_parser/src/semantics.rs`
- [x] **Issue**: `\underbrace{}` with an empty group produces `<XMArg/>` instead of `<XMTok meaning="absent"/>`.
- [x] **Change**: In `interpret_delimited` and other operator-application functions, check if the argument `XM` is "empty". If so, replace it with `XM::Token(absent())`.
- [x] **Perl Parity**: Matches structural simplification in Perl's `MathParser.pm`.

---

## 3. Fix Text-in-Math Handling
**Affected Tests**: `mixed_test`

### 3.1. Flatten `<XMText>` to `<XMTok>`
- [x] **Location**: `latexml_core/src/document.rs` (or Math Parser post-processing)
- [x] **Issue**: `\textbf{bf}` in math mode produces `<XMText><text font="bold">bf</text></XMText>`. Perl produces `<XMTok font="bold">bf</XMTok>`.
- [x] **Change**: Implement a simplification rule: if an `XMText` contains only a single `text` node with no complex internal structure (like spaces or nested nodes), convert it into an `XMTok` where the font and content are promoted.
- [x] **Perl Parity**: Matches math-mode text digestion in `LaTeXML::Core::Stomach`.

---

## 4. Resolve Math Parser Infrastructure
**Affected Tests**: `mixed_test`, `mathbbol_test`

### 4.1. Fix Precomputed Grammar Reset
- [x] **Location**: `latexml_math_parser/src/grammar.rs`
- [x] **Issue**: Repeated calls to the math parser trigger `RESET FAILED: This grammar is precomputed`. This causes the parser to fail and fall back to unparsed output, breaking almost all structural XML comparisons.
- [x] **Change**: Modify `Grammar::reset()` to be a no-op (returning `Ok(())`) if the grammar is precomputed and no actual changes to the grammar rules were requested. (Actually fixed by rebuilding from scratch in `parser.rs` if trivial parse fails).
- [x] **Perl Parity**: Perl's `MathParser` initializes the grammar once and reuses the state machine efficiently.

## 6. Analysis of `parser.rs` Deviations from `MathParser.pm`
- **Grammar Reset Strategy**: `MathParser.pm` generates a single grammar instance (`$LaTeXML::MathParser::GRAMMAR`), then compiles it into a `Marpa::R2::Scanless::R` recognizer for each equation. If it fails, it does not recompile the grammar. In Rust, `MarpaGrammar` gets locked when the recognizer starts. Because Rust Marpa bindings have different initialization lifecycle constraints, when `run_recognizer` encounters an unexpected token and errors out, the recognizer cannot just be reset cleanly; a fresh Engine (and potentially Grammar) needs to be instantiated. This causes performance overhead but maintains semantic equivalence.
- **`XMRef` Post-processing**: `MathParser.pm` uses `xml:id` immediately when creating virtual copies for the parser tree (using `$document->generateID`). However, it has an issue where nodes "replaced" by parsed structures lose track of XMRefs pointing to them, so it employs a `$LOSTNODES` hash to reconnect them. In `latexml-oxide`, we opted to map IDs through an ephemeral `_xmkey` attribute assigned during `create_xmrefs`. After parsing and reparenting, a post-processing pass translates these temporary keys into actual structural `xml:id` and `idref` attributes, correctly linking `XMRef` to `XMApp` bodies.
- **Text Form Fallback**: `MathParser.pm` calculates `$text` from the resulting parsed structure. If the parser fails (e.g. single unparsed token), it uses the fallback `tex` attribute or text-serialization. `latexml-oxide` matches this behavior precisely using the `text_form` function.
- **`def_math_dual` macro expansion mismatch**: `defmath_dual` in Perl successfully processes `\bbeta` by evaluating the presentation branch (`\lx@mbfont{\beta}`) into a fully detailed `XMTok` node (via `invokeToken_simple` parsing) which carries over to the math parser. In Rust, `def_math_dual` correctly processes the macro expansion, BUT it seems that `\lx@mbfont` evaluation or font application is swallowing attributes (or generating raw string text without meaning). This is the root cause for `mathbbol` not parsing.