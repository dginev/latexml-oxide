use rtx_package::*;

LoadDefinitions!(state, {

  // DefParameterType!(Foo, sub[gullet, inner, _extra, state] {
  //   gullet.skip_one_space(state);
  // });

  DefMacro!("\\hw", "hello world!");
});
