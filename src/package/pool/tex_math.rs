use package::*;
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  //======================================================================
  // Math mode stuff
  // See TeXBook Ch.26
  //======================================================================
  // Decide whether we're going into or out of math, inline or display.
  Tag!("ltx:XMText", auto_open => true, auto_close => true);
  DefPrimitiveII!(T_MATH!(), None, |stomach : &mut Stomach, tokens: Vec<Tokens>, state: &mut State| {
    let mut op        = "\\@@BEGININLINEMATH";
  {
    let mut gullet = stomach.get_gullet_mut();
    let mode      = LookupString!("MODE", state);
    if mode == "display_math" {
      if try!(gullet.if_next(T_MATH!(), state)) {
        gullet.read_token(state);
        op = "\\@@ENDDISPLAYMATH"; }
      else {
        // Avoid a Fatal, but we're likely in trouble.
        // Should we switch to text mode? (LaTeX normally wouldn't)
        // Did we miss something and would should have already been in text mode? Possibly...
        error!(target: "expected:$",
          "Missing $ closing display math.\nIgnoring; expect to be in wrong math/text mode.");
        op = ""
      }
    }
    else if mode == "inline_math" {
      op = "\\@@ENDINLINEMATH";
    }
    else if try!(gullet.if_next(T_MATH!(), state)) {
      gullet.read_token(state);
      op = "\\@@BEGINDISPLAYMATH";
    }
  }
    if !op.is_empty() {
      try!(stomach.invoke_token(T_CS!(op), state));
    }
    Ok(Vec::new())
  }, PrimitiveOptions::default());
  // Let this be the default, conventional $
  LetI!(&T_CS!("\\@dollar@in@normalmode"), T_MATH!());

  // Effectively these are the math hooks, redefine these to do what you want with math?
  DefConstructor!("\\@@BEGINDISPLAYMATH",
  "<ltx:equation>
    <ltx:Math mode=\"display\">
    <ltx:XMath>
    #body
    </ltx:XMath>
    </ltx:Math>
  </ltx:equation>",
    alias         => Some("$$".to_string()),
    before_digest => sub!(|stomach, state| { try!(stomach.begin_mode("display_math", state)); Ok(Vec::new()) }),
    capture_body  => true
  );

  DefConstructorI!(T_CS!("\\@@ENDDISPLAYMATH"), None, |doc,whatsit,props,state|{}, alias => Some("$$".to_string()),
    before_digest => sub!(|stomach, state|{ try!(stomach.end_mode("display_math", state)); Ok(Vec::new()) }));

  DefConstructor!("\\@@BEGININLINEMATH",
  "<ltx:Math mode=\"inline\">
    <ltx:XMath>
    #body
    </ltx:XMath>
  </ltx:Math>",
    alias => Some("$".to_string()),
    before_digest => sub!(|stomach, state| {try!(stomach.begin_mode("inline_math", state)); Ok(Vec::new())}),
    capture_body => true);

  DefConstructorI!(T_CS!("\\@@ENDINLINEMATH"), None, |doc,whatsit,props,state|{}, alias => Some("$".to_string()),
    before_digest => sub!(|stomach, state| { try!(stomach.end_mode("inline_math", state)); Ok(Vec::new()) }));

  Ok(())
}
