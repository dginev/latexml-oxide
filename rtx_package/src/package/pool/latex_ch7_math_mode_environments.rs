use std::rc::Rc;

use crate::package::*;
//======================================================================
// C.7.1 Math Mode Environments
//======================================================================

// # This provides {equation} with the capabilities for tags, nonumber, etc
// # even though stock LaTeX provides no means to override them.
// #   preset => boolean
// #   postset => boolean
// #   deferretract=>boolean
// fn prepare_equation_counter(options: Stored, state: &mut State) { state.assign_value("EQUATION_NUMBERING", options, Some(Scope::Global)); }

// fn before_equation(state: &mut State) {
//   let numbering = state.lookup_hash("EQUATION_NUMBERING");
//   let numbered = numbering.get("numbered").unwrap_or_default();
//   let ctr = numbering.get("counter").uwnrap_or("equation");
//   numbering.insert("in_equation", true);
//   if numbering.get("preset").unwrap_or(false) {
//     let props = if numbered { ref_step_counter(ctr, state) } else { ref_step_id(ctr) };
//     props.insert("preset", true);
//     state.assign_value("EQUATIONROW_TAGS", props, Some(Scope::Global));
//   } else {
//     state.assign_value("EQUATIONROW_TAGS", HashMap::new(), Some(Scope::Global));
//   }
// }

// sub retractEquation {
//   let numbering = LookupValue('EQUATION_NUMBERING');
//   # What about scopes? Is that handled automagically?
//   # What about \@currentID....
//   let tags = LookupValue('EQUATIONROW_TAGS');
//   let ctr = $$tags{counter} || $$numbering{counter} || 'equation';
//   if ($$tags{preset}) {    # ONLY if the number was preset!
//                            # counter (or ID counter) was stepped, so decrement it.
//     AddToCounter(($$numbering{numbered} ? $ctr : 'UN' . $ctr), Number(-1)); }
//   AssignValue(EQUATIONROW_TAGS => { RefStepID($ctr), reset => 1 }, 'global');
//   return; }

// sub afterEquation {
//   my ($whatsit) = @_;
//   let numbering = LookupValue('EQUATION_NUMBERING');
//   let tags      = LookupValue('EQUATIONROW_TAGS');
//   let ctr = $$tags{counter} || $$numbering{counter} || 'equation';
//   if (!$$tags{noretract}
//     && ($$tags{retract} || ($$numbering{retract} && $$numbering{preset} && $$tags{preset}))) {
//     retractEquation(); }
//   elsif ($$numbering{postset} && !$$tags{reset}) {
//     # my %props = ();
//     # if ($$numbering{numbered}) {
//     #   %props = RefStepCounter($ctr); }
//     # else {
//     #   %props = RefStepID($ctr); }
//     # AssignValue(EQUATIONROW_TAGS => {%props}, 'global'); }
//     AssignValue(EQUATIONROW_TAGS => {
//         ($$numbering{numbered} ? RefStepCounter($ctr) : RefStepID($ctr)) }, 'global'); }
//   elsif (!$$tags{reset} && $$numbering{numbered}) {
//     $$tags{tags} = Digest(Invocation(T_CS('\lx@make@tags'), $ctr)); }

//   # Now install the tags in $whatsit or current Row, as appropriate.
//   let props = LookupValue('EQUATIONROW_TAGS');
//   if ($$numbering{aligned}) {
//     if (let alignment = LookupValue('Alignment')) {
//       let row = $alignment->currentRow;
//       $$row{id}   = $$props{id};
//       $$row{tags} = $$props{tags}; } }
//   elsif ($whatsit) {
//     $whatsit->setProperties(%{ LookupValue('EQUATIONROW_TAGS') }); }
//   $$numbering{in_equation} = 0;
//   return; }

LoadDefinitions!(state, {
  // TODO: Implement environment modes properly, some work still to go
  // TODO: Re-add ltx: namespace when compiler can parse it
  DefEnvironment!("{math}",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    mode => Some(s!("inline_math"))
  );
  // My first inclination is to Lock {math}, but it is surprisingly common to redefine it in silly
  // ways... So...?
  DefEnvironment!(
    "{equation}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>"
  );
  // TODO: Caution -- very strange 30+ min infinite loop in rustc if used as-is
  // mode => Some(s!("display_math")),
  // before_digest => before_digest!(stomach, state, {
  //   prepare_equation_counter(map!(numbered => true, preset => true).into(), state);
  //   before_equation(state);
  // }),
  // after_digest_body => after_digest!(stomach, args, state, {
  //   after_equation(args, state);
  // }),
  // locked => true);

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
  before_digest => before_digest!(stomach, state, {stomach.begin_mode("display_math", state)?; }),
  capture_body  => true,
  properties   => properties!(sub[stomach, args, state] { ref_step_id("equation", stomach, state) })
  );

  DefConstructor!("\\]", "", before_digest => before_digest!(stomach, state, { stomach.end_mode("display_math", state)?; }));
});
