use package::*;
use rtx_core::state::State;

pub fn load_definitions(mut state: &mut State) -> Result<()> {

  try!(pool::tex_setup::load_definitions(&mut state));
  try!(pool::tex_structure::load_definitions(&mut state));
  try!(pool::tex_math::load_definitions(&mut state));

  try!(pool::tex_appendix_b::load_definitions(&mut state));

  try!(pool::tex_chapter_24::load_definitions(&mut state));
  Ok(())
}
