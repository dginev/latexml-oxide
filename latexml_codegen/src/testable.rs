use glob::glob;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// The only purpose of doing the glob for "*.tex" tests at compile-time is to
/// make sure each TeX entry gets a dedicated #[test] header, and respectively
/// increments the counter for the number of tests which have been run.
/// In addition, this allows running more tests in parallel.
pub fn compile_tests_at(input: DeriveInput) -> TokenStream {
  let directory = crate::attr_name_value_str(&input.attrs[0], "directory");
  // TODO: How do we best manage the relative directories changing from compile-time to test-time?
  let test_functions: Vec<_> = glob(&format!("latexml_oxide/{directory}/*.tex"))
    .unwrap()
    .flatten()
    .filter(|pb| {
      // ensure there is an XML target
      let mut xml_pb = pb.to_owned();
      xml_pb.set_extension("xml");
      xml_pb.is_file()
    })
    .map(|pb| {
      let filebase = pb.file_stem().unwrap().to_string_lossy();
      let file = pb.strip_prefix("latexml_oxide/").unwrap().to_string_lossy();
      let fn_filename = filebase.replace(['-', ' ', '.'], "_");
      // Rust identifiers cannot start with a digit; prepend `t_` for
      // tex filenames like `3d-cone.tex` that would otherwise produce
      // `3d_cone_test` (an invalid ident).
      let fn_filename = if fn_filename
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
      {
        format!("t_{fn_filename}")
      } else {
        fn_filename
      };
      let fn_name = format_ident!("{fn_filename}_test");
      let attrs = quote!(#[test]);
      quote!(
        #attrs
        fn #fn_name() {
          latexml::util::test::latexml_test_single(#file, #filebase, #directory,
            this_test_requires!(), this_test_dispatch!())
        }
      )
    })
    .collect();

  quote!(
    #(#test_functions)*
  )
  .into()
}
