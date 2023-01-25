use rtx_package::*;

LoadDefinitions!(state, {

  TypedMacro!("\\sampler" number:Number, token:Token, dimension:Dimension => sub[gullet, _state] {
    Tokens!()
  });

  DefMacro!("\\classic Number Token Dimension", sub[gullet, args, _state] {
    Tokens!()
  });
});
