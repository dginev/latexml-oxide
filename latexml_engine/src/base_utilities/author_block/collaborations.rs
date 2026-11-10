//! A collaboration or team named in an author line: a `<contact role="collaboration">` on the names it covers, no
//! creator of its own and no part of a name (FIDELITY_CHARTER §4, user ruling 2026-11-01; OXIDIZED_DESIGN_DIVERGENCES
//! #946). The contact is the one Perl's aastex binding makes of `\collaboration{n}{name}`: `\lx@add@contact[role=
//! collaboration,annotate=new]` (aas_support.sty.ltxml:126), on the authors before it that carry none — here those of
//! the author call's `\and` group, `annotate=new:N`.

use super::*;
use crate::base_utilities::defs::frontmatter_fields::collaboration_line_reading;

/// Whether a line of an author block, a line of its own, names a collaboration ([`reads_as_collaboration`], a team word
/// framed): `(XYZ Collaboration)` under the names (2410.11607), `on behalf of the LHCb collaboration` (1905.11825).
pub(in crate::base_utilities) fn collaboration_line(line: &Tokens) -> bool {
  collaboration_line_reading(line).is_some()
}
