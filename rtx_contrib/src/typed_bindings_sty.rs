use rtx_package::*;

LoadDefinitions!(state, {
  TypedMacro!("\\sampler Number Token Dimension", sub[gullet, (number, token, dimension), _state] {
    number.value_of();
    dbg!(token);
    dbg!(dimension);
    Tokens!()
  });

  // DefMacro!("\\classic Number Token Dimension", sub[gullet, args, _state] {
  //   Tokens!()
  // });
});
