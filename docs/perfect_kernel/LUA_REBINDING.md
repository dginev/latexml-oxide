# Lua rebinding — can we "natively emulate" texlua code paths?

*2026-08-31. Answers the mission question: given that we are not running the
original TeX engine, can LuaTeX code paths ever be natively emulated, or must
the Lua code be rebound-as-we-emulate into forms that fit latexml-oxide's XML
output?*

## The answer

**Rebinding is not an alternative to native emulation — it is the only
mechanism that exists, and it was already the architecture before the question
was asked.** The empirical ground: `texlua` is Lua plus TeX's *support*
libraries (`kpse`, `lpeg`, `md5`, `pdfe`, …) minus the engine. Its built-in
`tex` table holds exactly `run`/`initialize`/`finish` — nothing an author's
`\directlua` chunk touches. Every `tex.print`, every `tex.count[255]`, every
`pdf.setmajorversion` a chunk calls resolves to a function **we** put there in
the bridge prelude. So "natively emulate the texlua code paths" and
"rebind-as-we-emulate" are the same activity; the only real design decision is
**how deep each shim goes**, and that decision is made per-API against what
our XML pipeline models.

## The three shim tiers

| Tier | Meaning | APIs | Fate of the intent |
|---|---|---|---|
| **Translate** | the Lua side's *output* re-enters our pipeline | `tex.print`/`sprint`/`write`/`tprint` | retokenized with current catcodes, digested normally |
| **Mirror** | state our engine genuinely has is served *live* | `tex.count`, `tex.dimen` (reads **and** writes, indexed or `\countdef`'d-name keys), `tex.getcount`/`setcount`/`getdimen`/`setdimen` (incl. `'global'`) | framed `Q`/`A` protocol over the child's pipes; `lua_bridge::service_query` reads/writes the Rust State mid-chunk |
| **Absorb** | intents with **no XML meaning** are accepted and dropped | `pdf.set*` (backend versions), `tex.primitives()` → `{}`, `texio.*`, `tex.error` | silently no-op — the document proceeds exactly as if the engine honored them |

Outside the tiers, deliberately: **`node.*` / `font.*` / `callback.*`** — the
typesetter-internals layer. These *cannot* be mirrored (we have no node lists)
and *must not* be absorbed silently (a chunk that builds nodes expects them to
appear); they error visibly and the chunk degrades to a no-op with an
`Info:lua` line. If a package's node-layer output matters for content, the
correct move is a **binding** that maps the package's *intent* to our
constructors — the same rule as for any engine feature (e.g. piton's LPEG
highlighter emitting listings-shaped output would be a Translate-tier project,
not a node-layer one).

`require` is itself rebound: a `package.searchers` entry backed by texlua's
built-in `kpse` resolves texmf-shipped Lua modules, and `lualibs` (89 ms,
loaded once per interpreter) supplies the `file`/`string`/`table` extensions
that lualatex code takes for granted via luaotfload. The child runs with
`SOURCEDIRECTORY` as cwd — LuaTeX job semantics for relative file access.

## Ground truth from the real engine (probe results, luatex 1.22 / TL2025)

- `\directlua{ local x = 1 \par @@@ }` → error `near '@'` at **line 1**: the
  chunk reaches Lua as a single line, and a `\par` token (blank lines inside
  chunks produce one) contributes **nothing** to the string.
- `\directlua{ tex.sprint(42) \relax }` → error `near '\'`: other
  unexpandable CSes keep their backslash form.
- `\directlua` performs ONE `\edef`-like partial expansion, honoring
  `\noexpand` (babel's `[[\noexpand\csname bbl@error\endcsname{]]` idiom).

## Red/green record (all TDD'd red-first, 2026-08-31)

| Test (guard) | Red symptom | Green mechanism |
|---|---|---|
| `cluster_package_guards::lua_state_mirror` | `\count255=7` read back as 0; Lua-side write invisible | `Q`/`A` mirror protocol + `service_query` |
| `lua_bridge::tests::rebound_engine_intents_absorb_and_resolve` | `pdf` nil, `tex.primitives` nil, `require("newpax")` module-not-found | absorb shims + kpse searcher + lualibs |
| `cluster_package_guards::luatex_babel_api` | `BNO` — `Babel.locale_props` never created; every profiled doc logged `attempt to index a nil value (field 'locale_props')` | three stacked fixes, below |

The babel chain needed three root causes, each found from first principles:

1. **Format parity** (`latexml_sty` `luatex` option): the lualatex *format*
   ships hyphenation patterns, so `\bbl@luapatterns` is already defined when
   babel.def loads — babel.def L1135 then skips the patterns-only first
   `\input luababel.def` (which `\endinput`s at L195 and whose loaded-flag
   would suppress the real second input at L2285). We predefine it as an
   absorb no-op (patterns have no XML meaning).
2. **Double expansion** (`\lx@directlua`): `XGeneralText` already does the
   partial expansion; a second `Expand!` re-expanded the no-longer-protected
   `\csname`, and the resulting macro call ate the Lua text mid-chunk
   ("unfinished long string"). Minimal repro preserved in the pdftex.rs
   comment.
3. **`\par` dropping + direction primitives**: blank-line `\par` tokens are
   filtered from the chunk (ground truth above); `\pagedir`/`\textdir` &co
   are keyword-eating absorbers, not `{TLT}` macros (which leaked `TLTTLT`
   text).

## Witness articles (TL2025 doc corpus, clean-lualatex slice)

| Witness | Lua construct | Tier exercised | State after |
|---|---|---|---|
| `derivative/derivative` | `pdf.setmajorversion(2)` | Absorb | chunk runs clean; babel Lua layer initializes |
| `abntexto/abntexto`, `abntexto-uece` | `tex.primitives()` → listings `texcs` list | Absorb | clean (empty highlight list, no error) |
| `newpax/newpax` | `require("newpax")` + `pdfe` reads of shipped PDFs | require-rebind + cwd | module loads, inputs found; residual: *writing* `.newpax` into the root-owned TL tree fails (as it would for lualatex run there) → `Info:lua` |
| every profiled doc (systemic) | babel `Babel.locale_props[...]` chunk sequence | Translate + format parity | zero `locale_props` failures across all five witnesses |
| `pythontex/pythontex_quickstart` | (Lua incidental) | — | unchanged, 1 unrelated error |

Error *counts* on these witnesses barely moved — expected and previously
documented (LEDGER sweep 10): dead Lua branches fail silently, live ones do
real work. The wins here are **content correctness** (chunks no longer branch
on stub zeros; babel's Lua API exists for everything downstream) and the
removal of a whole systemic failure class, not headline error deltas.

## Residuals / next levers

- `tex.toks`, `tex.attribute`, `tex.sp`, `tex.getmacro` — extend the mirror
  protocol as witnesses demand (same `Q`/`A` shape).
- `tkz_elements_main.lua` loads fully under the kpse searcher — the
  tkz-elements bundle (not in the golden-PDF corpus, but ~20 manuals) becomes
  reachable when wanted.
- Node/callback-layer packages stay binding territory; catalog per case in
  `CLUSTERS.md` before deciding absorb vs bind.
