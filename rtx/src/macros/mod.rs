#[macro_export]
/// Macro for compiling string construction replacements into closures
/// that execute the needed ops in libxml.
/// Approach borrowed from diesel-codegen
macro_rules! compile_replacement {
  ($var:ident, $replacement:expr) => {{
    use libxml::tree::Node;
    use rtx_core::definition::ReplacementClosure;
    use rtx_core::document::Document;
    use rtx_core::Digested;
    #[derive(CompileReplacement)]
    #[compile_replacement_options(replacement=$replacement)]
    struct _Dummy;
    $var = _Dummy::replacement();
  }};
}

#[macro_export]
/// Macro for compiling string macro expansions into closures
/// Approach borrowed from diesel-codegen
macro_rules! compile_expansion {
  ($var:ident, $expansion:expr) => {{
    use rtx_core::definition::ExpansionClosure;
    #[allow(unused_imports)]
    use rtx_core::token::Catcode;
    #[derive(CompileExpansion)]
    #[compile_expansion_options(expansion=$expansion)]
    struct _DummyE;
    $var = _DummyE::expansion();
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
    #[allow(unused_attributes)]
    #[derive(LoadModel)]
    #[load_model_options(name=$name)]
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
