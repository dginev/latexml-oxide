use glob::glob;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Lit, Meta};

/// The only purpose of doing the glob for "*.tex" tests at compile-time is to
/// make sure each TeX entry gets a dedicated #[test] header, and respectively
/// increments the counter for the number of tests which have been run.
/// In addition, this allows running more tests in parallel.
pub fn compile_tests_at(input: DeriveInput) -> TokenStream {
  let directory: String = match input.attrs[0].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value(),
      _ => {
        panic!("only accepts #[directory = \"value\"] attribute, mandatory double-quotes (Lit)")
      },
    },
    _ => panic!(
      "only accepts #[directory = \"value\"] attribute, mandatory double-quotes (parse_meta)"
    ),
  };
  // TODO: How do we best manage the relative directories changing from compile-time to test-time?
  let test_functions: Vec<_> = glob(&format!("rtx/{directory}/*.tex"))
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
      let file = pb.strip_prefix("rtx/").unwrap().to_string_lossy();
      let fn_filename = filebase.replace(['-', ' ', '.'], "_");
      let fn_name = format_ident!("{fn_filename}_test");
      quote!(
        #[test]
        fn #fn_name() {
          rtx::util::test::rtx_test_single(#file, #filebase, #directory,
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
