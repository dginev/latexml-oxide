//! thumbs.sty — page-edge thumb tabs.
//!
//! The tabs are drawn at shipout (`\AtBeginShipout`, thumbs.sty:1076) at
//! absolute page positions; LaTeXML never ships out, so the reset the
//! handler performs (:1151 `\gdef\th@mbtoprint{0}`) never runs and the
//! package's own state machine raises "\thumbnewcolumn after \addthumb"
//! (:573) on the second column (thumbs-example; Perl identical). Thumb tabs
//! have no reflowable-HTML meaning, so the real package is loaded and then
//! put in the author's own off-mode — the `hidethumbs` branch, thumbs.sty:
//! 1531-1538 (`\addthumb`/`\thumbnewcolumn` → `\relax`), applied after the
//! load because kvoptions' `\DeclareBoolOption` (:130) re-initialises the
//! flag at load time. Everything else stays the real thumbs.sty.
//! Guard: `perfect_kernel_batch56::thumbs_loads_in_its_own_hide_mode`.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("thumbs", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // thumbs.sty:1531-1538, the `\ifthumbs@hidethumbs` branch.
  RawTeX!(
    r"\thumbs@hidethumbstrue\renewcommand{\addthumb}[4]{\relax}\renewcommand{\thumbnewcolumn}{\relax}"
  );
});
