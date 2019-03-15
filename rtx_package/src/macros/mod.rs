#[macro_export]
/// Macro for compiling string construction replacements into closures
/// that execute the needed ops in libxml.
/// Approach borrowed from diesel-codegen
macro_rules! compile_replacement {
  ($var:ident, $replacement:expr) => {{
    use rtx_core::definition::ReplacementClosure;
    #[derive(CompileReplacement)]
    #[replacement=$replacement]
    struct _Dummy;
    let tmp: Option<ReplacementClosure> = this_replacement!();
    $var = tmp;
  }};
}

#[macro_export]
/// Macro for compiling string macro expansions into closures
/// Approach borrowed from diesel-codegen
macro_rules! compile_expansion {
  ($var:ident, $expansion:expr) => {{
    use rtx_core::definition::ExpansionBody;
    #[allow(unused_imports)]
    use rtx_core::token::Catcode;
    #[derive(CompileExpansion)]
    #[expansion=$expansion]
    struct _DummyE;
    let tmp: Option<ExpansionBody> = this_expansion!();
    $var = tmp;
  }};
}

#[macro_export]
macro_rules! load_model {
  ($var:expr, $name:expr) => {{
    use rtx_core::common::model::Model;
    use rtx_core::common::relaxng::Relaxng;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    // use rtx_core::common::model::IndirectModel;
    #[derive(LoadModel)]
    #[name=$name]
    struct _ModelLoader;
    {
      // compute the model
      _ModelLoader::model(&mut $var.model);
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
