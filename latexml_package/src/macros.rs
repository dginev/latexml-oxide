#[macro_export]
/// Macro for compiling string construction replacements into closures
/// that execute the needed ops in libxml.
/// Approach borrowed from diesel-codegen
macro_rules! compile_replacement {
  ($var:ident, $replacement:literal) => {{
    use latexml_core::definition::ReplacementClosure;
    #[derive(CompileReplacement)]
    #[replacement=$replacement]
    struct _Dummy;
    let tmp: Option<ReplacementClosure> = this_replacement!();
    $var = tmp;
  }};
}

#[macro_export]
/// Macro for compiling string binding prototypes into Expandable closures
/// Approach borrowed from diesel-codegen
macro_rules! compile_prototype_for_typed_macro {
  ($prototype:literal, sub [ ( $($var:ident),* )] $body:block
    $($input:tt)*) => {{
    #[derive(CompilePrototypeFor)]
    #[prototype=$prototype]
    #[inner="TypedMacro"]
    struct _DummyP;
    this_prototype!(sub [ ( $($var),* ) ] $body $($input)*);
  }};
}

#[macro_export]
/// Macro for compiling string binding prototypes into Primitive closures
macro_rules! compile_prototype_for_typed_primitive {
  ($prototype:literal, sub [( $($var:ident),* )] $body:block
    $($input:tt)*) => {{
    #[derive(CompilePrototypeFor)]
    #[prototype=$prototype]
    #[inner="TypedPrimitive"]
    struct _DummyP;
    this_prototype!(sub [ ( $($var),* )] $body $($input)*);
  }};
}

#[macro_export]
/// Macro for compiling string binding prototypes into Conditional closures
macro_rules! compile_prototype_for_typed_conditional {
  ($prototype:literal, sub [( $($var:ident),* )] $body:block
    $($input:tt)*) => {{
    #[derive(CompilePrototypeFor)]
    #[prototype=$prototype]
    #[inner="TypedConditional"]
    struct _DummyP;
    this_prototype!(sub [( $($var),* )] $body $($input)*);
  }};
}

#[macro_export]
/// Macro for compiling string binding prototypes into ColumnType closures
/// Approach borrowed from diesel-codegen
macro_rules! compile_prototype_for_typed_columntype {
  ($prototype:literal, sub [ ( $($var:ident),* )] $body:block
    $($input:tt)*) => {{
    #[derive(CompilePrototypeFor)]
    #[prototype=$prototype]
    #[inner="TypedColumntype"]
    struct _DummyP;
    this_prototype!(sub [ ( $($var),* ) ] $body $($input)*);
  }};
}

#[macro_export]
/// Macro for compiling string literal prototypes into a Token and Parameters structs
macro_rules! compile_prototype {
  ($prototype:literal) => {{
    #[derive(CompilePrototype)]
    #[prototype=$prototype]
    struct _DummyPR;
    this_cs_and_parameters!()
  }};
}

#[macro_export]
/// Macro for compiling string macro expansions into closures
/// Approach borrowed from diesel-codegen
macro_rules! compile_expansion {
  ($var:ident, $expansion:literal) => {{
    use latexml_core::definition::ExpansionBody;
    #[derive(CompileExpansion)]
    #[expansion=$expansion]
    struct _DummyE;
    let tmp: Option<ExpansionBody> = this_expansion!();
    $var = tmp;
  }};
}

#[macro_export]
macro_rules! load_model {
  ($name:literal) => {{
    use latexml_core::common::model;
    use latexml_core::common::relaxng::Relaxng;
    use rustc_hash::FxHashSet as HashSet;
    use std::iter::FromIterator;
    // use latexml_core::common::model::IndirectModel;
    #[derive(LoadModel)]
    #[name=$name]
    struct _ModelLoader;
    {
      // compute the model
      _ModelLoader::build_model();
    }

    // TODO: It seems that properly computing the indirect hash requires all Tag!() definitions to
    // have been executed. as those definitions are currently applied at runtime, based
    // on the input document, it is unclear if the indirect math can be submerged to
    // compile-time, without altering the algorithm. compute the indirect model
    // let indirect_model;
    // {
    //   indirect_model = _ModelLoader::indirect_model();
    // }
    // $var.indirect_model = Some(indirect_model);
  }};
}

#[macro_export]
/// Macro for compiling string literal tokens into their usual Tokens representation
macro_rules! compile_tokenize {
  ($var:ident, $literal:expr) => {{
    #[derive(CompileTokens)]
    #[literal=$literal]
    struct _Dummy;
    let tmp: Tokens = these_tokens!();
    $var = tmp;
  }};
}

#[macro_export]
/// Macro for compiling string literal tokens into their preamble Tokens representation
macro_rules! compile_tokenize_internal {
  ($var:ident, $literal:expr) => {{
    #[derive(CompileTokensInternal)]
    #[literal=$literal]
    struct _Dummy;
    let tmp: Tokens = these_internal_tokens!();
    $var = tmp;
  }};
}

// TODO: ideally we can auto-infer this based on DefParameterType declarations but the **timing** is
// tricky? what to do? For now, hardcode.
#[macro_export]
macro_rules! parameter_rust_type {
  (GeneralText) => {Tokens};
  (XGeneralText) => {Tokens};
  (Semiverbatim) => {Tokens};
  (SanitizedVerbatim) => {Tokens};
  (Verbatim) => {Tokens};
  (HyperVerbatim) => {Tokens};
  (Digested) => {Tokens};
  (DigestedBody) => {Tokens};
  (DigestUntil) => {Tokens};
  (BalancedParen) => {Option<Tokens>};
  (TeXDelimiter) => {Tokens};
  (MoveableBox) => {Option<Tokens>};
  (Until) => {Tokens};
  (UntilBrace) => {Tokens};
  (TeXFileName) => {Tokens};
  (Match) => {Tokens};
  (AlignmentTemplate) => {Template};
  (DefPlain) => {Tokens};
  (DefExpanded) => {Tokens};
  (Plain) => {Tokens};
  (Optional) => {Option<Tokens>};
  (OptionalMatch) => {Option<Tokens>};
  (DefToken) => {Token};
  (Expanded) => {Tokens};
  (ExpandedPartially) => {Tokens};
  (XToken) => {Token};
  (Relation) => {Token};  // Perl: skip spaces + readXToken(0,1) — result is a Token
  (ExpandedIfToken) => {Token};
  (CSName) => {Token};
  (CSNameQuiet) => {Token};
  (Variable) => {ArgWrap};
  // For now return the raw Tokens for KeyVals, until we figure out how to
  // do TryInto with access to the current "state" object.
  (OptionalKeyVals) => {Option<KeyVals>};
  (RequiredKeyVals) => {KeyVals};
  (KeyVals) => {KeyVals};
  ($other:ident) => {$other};
}
