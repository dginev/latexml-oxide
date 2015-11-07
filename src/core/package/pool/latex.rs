use state::State;
use core::package::*;
use core::token::*;
use core::definition::constructor::ConstructorOptions;

pub fn load_definitions(state : &mut State) {
  println!("If you are seeing this, someone invoked latex::load_definitions !!! ");

  DefConstructor("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []".to_string(),
    "<?latexml class='#2' ?#1(options='#1')?>".to_string(), ConstructorOptions{
      afterDigest: Some(|stomach, whatsit, state| {
      let options = whatsit.get_arg(1);
      let opts_regex = regex!(r"/,\s*/");
      let class_opts = match options {
        Some(opts) => opts_regex.split(opts.to_string()).collect(),
        None => Vec::new()
      };
      load_class(state, whatsit.get_arg(2).to_string(),
        class_opts,
        vec![T_CS("\\AtBeginDocument".to_string()), T_CS("\\warn@unusedclassoptions".to_string())]);
      return; }),
      ..ConstructorOptions::default() }, state );
}