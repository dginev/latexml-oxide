//! An author line that names a collaboration (`collaboration_line`), for authblk's gate
//! `rows::lines_under_names_read_as_names`. Moved unchanged by Z18m, which deleted `author_block/`, from `author_block/collaborations.rs`, whose collaboration contact
//! (`<contact role="collaboration">`, FIDELITY_CHARTER §4, OXIDIZED_DESIGN_DIVERGENCES #946; Perl's aastex
//! `\collaboration`, aas_support.sty.ltxml:126) Z18a moved with its line reading to `defs/frontmatter_fields/names.rs`
//! (`collaboration_line_reading`).

use super::*;
use crate::base_utilities::defs::frontmatter_fields::collaboration_line_reading;

/// Whether a line of an author block, a line of its own, names a collaboration ([`reads_as_collaboration`], a team word
/// framed): `(XYZ Collaboration)` under the names (2410.11607), `on behalf of the LHCb collaboration` (1905.11825).
pub(in crate::base_utilities) fn collaboration_line(line: &Tokens) -> bool {
  collaboration_line_reading(line).is_some()
}
