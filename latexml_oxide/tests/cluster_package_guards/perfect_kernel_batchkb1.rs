//! Guards for KB1, a printed mark glued from an address label and a note's number under `LATEXML_FM_EMIT=2`
//! (`a_glued_address_and_note_mark_parts_at_emit_2`).

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

/// ws-p8's `Maarten Golterman$^{a1}$ and Santi Peris$^b$` over `\address{{}$^a$… \\E-mail: … \\{}$^b$… \\E-mail: …}`
/// (hep-ph0011101): "a1" is address a and the body's footnote 1. At `=2` the reader parts it (`parted_label`'s note
/// number, entries labelled by letters alone), so address a is Golterman's, not the call's `annotate=` default (Peris,
/// KEY1's "label:unrequested owner:annotate", on KB1base); the call, written by its two lines, keeps each inner e-mail
/// its own item (`items_to_emit`'s `separate`, `by_pieces`). Off stays today's output: both addresses on Peris, one
/// contact. Distinct from `perfect_kernel_batchfx5` (an entry not written at all) and from the names' digit parting
/// (`12` under entries 1, 2): a number piece no entry holds, a call cut by the emitter, not the queue.
#[test]
fn a_glued_address_and_note_mark_parts_at_emit_2() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ws_p8_glued_mark_address_and_note.tex"
  );
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    vec![
      "Maarten Golterman a1",
      "address=a Department of Physics, Washington University, St. Louis MO 63130, USA",
      "email=maarten@aapje.wustl.edu"
    ],
    vec![
      "Santi Peris b",
      "address=b IFAE, Univ. Autonoma de Barcelona, Spain",
      "email=peris@ifae.es"
    ],
  ]);
  assert_eq!(xml.matches("maarten@aapje.wustl.edu").count(), 1);
  assert_eq!(xml.matches("peris@ifae.es").count(), 1);
}
