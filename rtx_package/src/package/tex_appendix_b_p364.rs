use crate::package::*;

//======================================================================
// TeX Book, Appendix B. p. 364
LoadDefinitions!(state, {
  // Let's hope nobody is messing with the output routine...
  DefPrimitive!("\\footnoterule", None);
});
