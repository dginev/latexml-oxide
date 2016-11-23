#[macro_export]
/// Macro for compiling string construction replacements into closures
/// that execute the needed ops in libxml.
/// Approach borrowed from diesel-codegen
macro_rules! compile_replacement {
    ($var: ident, $replacement: expr) => {
        use rtx_core::BoxOps;

        #[allow(unused_attributes)]
        #[derive(CompileReplacement)]
        #[options(replacement=$replacement)]
        struct _Dummy;

        $var = _Dummy::replacement();
    }
}
