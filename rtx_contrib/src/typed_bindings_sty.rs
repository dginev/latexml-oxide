use rtx_package::*;

// This is a first demonstration of using the Rust codegen approach for inducing argument types from TeX ParameterTypes.
// More to be done: so far this is the only variant that has been implemented (and only in DefMacro)

LoadDefinitions!(state, {
  DefMacro!("\\sampler Number Token Dimension",
    sub[gullet, (number, token, dimension), _state] {
      number.value_of();
      dbg!(token);
      dbg!(dimension);
    });
});