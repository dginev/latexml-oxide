use std::rc::Rc;

use crate::package::*;
//======================================================================
// C.7.1 Math Mode Environments
//======================================================================
LoadDefinitions!(state, {
  // TODO: Implement environment modes properly, some work still to go
  // TODO: Re-add ltx: namespace when compiler can parse it
  DefEnvironment!("{math}",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    mode => Some(s!("inline_math"))
  );
  // My first inclination is to Lock {math}, but it is surprisingly common to redefine it in silly
  // ways... So...?

  // Define \( ..\) and \[ ... \] to act like environments.
  // I would have thought these should be locked, but it seems relatively common to
  // redefine them as \left[ \right] and \left( \right) !
  DefConstructor!("\\[",
  "<ltx:equation xml:id='#id'>\
    <ltx:Math mode='display'>\
    <ltx:XMath>\
    #body\
    </ltx:XMath>\
    </ltx:Math>\
    </ltx:equation>",
  before_digest => beforeproc!(gullet, state, {gullet.begin_mode("display_math", state)?; }),
  capture_body  => true,
  properties   => properties!(sub[stomach, args, state] { ref_step_id("equation", stomach, state) })
  );

  DefConstructor!("\\]", "", before_digest => beforeproc!(gullet, state, { gullet.end_mode("display_math", state)?; }));
});
