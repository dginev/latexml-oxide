# Third-party license texts

**These are NOT latexml-oxide's license.** latexml-oxide is dedicated to the
public domain under [CC0 1.0 Universal](../LICENSE).

The files here are verbatim, unmodified copies of licenses belonging to
**third-party libraries** that the distributed binary links. They live in the
repo so [`tools/gen_notices.sh`](../tools/gen_notices.sh) can append them as
section 6 ("COPYLEFT LICENSE TEXTS") of the [`THIRD-PARTY-NOTICES`](../THIRD-PARTY-NOTICES)
shipped with every release — those licenses require the text to travel with the
binary.

| File | Applies to | Why it's here |
|---|---|---|
| `LGPL-2.1.txt` | `libkpathsea`; libmarpa's `marpa_obs.c` (from the GNU obstack) | Statically linked → the license text must ship. |
| `LGPL-3.0.txt` | libmarpa's `marpa_avl.c` / `marpa_tavl.c` (from Ben Pfaff's libavl) | Same. |
| `GPL-3.0.txt` | — (nothing here is GPL-licensed) | LGPL-3.0 is a set of *additional permissions* layered on GPL-3.0 and is not self-contained, so its text is meaningless without this one. Present **only** as LGPL-3.0's base text. |

To be explicit, since a directory of GPL files invites the wrong conclusion: no
GPL-licensed code is linked into latexml-oxide, and none of these terms apply to
latexml-oxide's own source. See `THIRD-PARTY-NOTICES` §3 for what each library is
and §3.5 for the relink note, or
[`docs/release/LICENSE_INVENTORY.md`](../docs/release/LICENSE_INVENTORY.md) for the
full audit.

Sourced from the canonical FSF texts as distributed in Debian's
`/usr/share/common-licenses/`. Do not edit them — they are verbatim by
requirement.
