# Supported packages and classes

**Generated** from the binding tables — `latexml_package::BINDINGS` (`latexml_package/src/lib.rs`) and
`latexml_contrib::BINDINGS` (`latexml_contrib/src/lib.rs`), the lists `dispatch` reads — and checked on every test
run (`supported_packages_doc_is_up_to_date`); regenerate with
`UPDATE_SUPPORTED_PACKAGES=1 cargo test -p latexml_contrib --lib supported_packages_doc_is_up_to_date`.

- **ported** — a compiled binding in `latexml_package` for a package Perl LaTeXML binds too: Perl ships a `.ltxml`
  for the name, or for the generic name its versioned-name fallback strips it to (`docs/parity/perl_bindings.txt`,
  the snapshot of Perl's binding names, refreshed with each upstream sync).
- **beyond Perl** — a compiled binding in `latexml_package` for a name Perl LaTeXML has no binding for.
- **contributed** — a compiled binding in `latexml_contrib`, the contributed bindings beyond Perl LaTeXML's tree.
- **raw + overrides** — the binding reads the package's own TeX file raw (`InputDefinitions!("<name>", noltxml …)`
  or `input_definitions(…, noltxml: true …)`) and defines only what carries semantics around it.
- *alias of* — the row runs the same loader as the one named (a version or a renamed copy).
- A note quotes the binding's own module doc where it calls itself a stub or a partial binding.

**Name matching.** A request matches a row by exact name first, then case-insensitively (`ieeetran` finds `IEEEtran.cls`). A
versioned name with no row of its own falls back to the generic row, as Perl's `FindFile_fallback` does
(`Package.pm:2191-2226`): version suffixes (`aastex701` → `aastex`, `neurips_2026` → `neurips`, `-arxiv`, `_final`,
`_v2`…) and the prefixes `rw`, `my`, `preprint` (`mysvjour3` → `svjour3`) are stripped.

**Anything not listed.** A `<name>.sty.rhai` / `<name>.cls.rhai` runtime binding beside the document (or under
`--path`) is read first and overrides any row here; one on the TeX tree fills a name no row has
([`parity/RHAI_BINDINGS_GUIDE.md`](parity/RHAI_BINDINGS_GUIDE.md)). Otherwise a package with no binding is read raw
from TeX Live only when raw styles are allowed (`INCLUDE_STYLES`: `--includestyles`, or `latexml.sty`'s `rawstyles`,
`localrawstyles` for the document's own directory; `--preload=ar5iv.sty` turns on `rawstyles` and `rawclasses`); when
they are not, it is reported missing, its dependencies on bound packages still loaded. A class with no binding is read
raw under `INCLUDE_CLASSES` (`rawclasses`, `localrawclasses`, `--includestyles`), else it falls back to OmniBus. A
compiled binding is never displaced by raw mode.

## Classes (226)

| Name | Kind | Support | Note |
|---|---|---|---|
| `a0poster` | `.cls` | ported |  |
| `aa` | `.cls` | ported |  |
| `aamas` | `.cls` | contributed | Stub for AAMAS conference class. |
| `aastex` | `.cls` | ported |  |
| `aastex62` | `.cls` | ported | *alias of* `aastex.cls` |
| `aastex63` | `.cls` | ported | *alias of* `aastex.cls` |
| `aastex631` | `.cls` | ported | *alias of* `aastex.cls` |
| `aastex7` | `.cls` | ported | *alias of* `aastex.cls` |
| `aastex70` | `.cls` | ported | *alias of* `aastex.cls` |
| `achemso` | `.cls` | contributed | Stub for achemso.cls (ACS chemistry journals). |
| `acmart` | `.cls` | ported |  |
| `aero` | `.cls` | contributed, raw + overrides |  |
| `agujournal2019` | `.cls` | contributed | Stub for agujournal2019.cls (AGU journal template). |
| `aguplus` | `.cls` | contributed, raw + overrides |  |
| `agutexSI2019` | `.cls` | contributed | *alias of* `agujournal2019.cls`; Stub for agujournal2019.cls (AGU journal template). |
| `aipproc` | `.cls` | ported |  |
| `alggeom` | `.cls` | contributed |  |
| `ams_core` | `.cls` | ported |  |
| `amsart` | `.cls` | ported |  |
| `amsbook` | `.cls` | ported |  |
| `amsdtx` | `.cls` | contributed, raw + overrides |  |
| `amsldoc` | `.cls` | contributed, raw + overrides |  |
| `amsppt` | `.cls` | beyond Perl | *alias of* `amsppt.sty` |
| `amsproc` | `.cls` | ported |  |
| `an` | `.cls` | contributed |  |
| `aomart` | `.cls` | contributed | Stub for aomart.cls (Annals of Mathematics). |
| `apa` | `.cls` | contributed |  |
| `apa6` | `.cls` | contributed |  |
| `apa7` | `.cls` | contributed, raw + overrides |  |
| `appolb` | `.cls` | contributed |  |
| `article` | `.cls` | ported |  |
| `arxbj` | `.cls` | contributed |  |
| `arximspdf` | `.cls` | beyond Perl |  |
| `arxstspdf` | `.cls` | beyond Perl |  |
| `asme2ej` | `.cls` | contributed | Stub for asme2ej.cls (ASME journal class). |
| `asmejour` | `.cls` | beyond Perl |  |
| `atlasdoc` | `.cls` | contributed |  |
| `autart` | `.cls` | contributed | Stub for Elsevier's autart.cls (Automatica journal). |
| `beamer` | `.cls` | ported | beamer.cls — Minimal stubs for beamer presentation class Perl: beamer.cls.ltxml (1364 lines) |
| `beilstein` | `.cls` | contributed, raw + overrides |  |
| `bmcart` | `.cls` | contributed |  |
| `bmvc2k` | `.cls` | contributed | Stub for bmvc2k.cls (BMVC British Machine Vision Conference). |
| `book` | `.cls` | ported |  |
| `brief` | `.cls` | contributed, raw + overrides |  |
| `bytedance_seed` | `.cls` | contributed | Stub for bytedance_seed.cls (ByteDance Seed paper template). |
| `cas-dc` | `.cls` | contributed | Stub for Elsevier cas-dc.cls / cas-sc.cls (CAS journals double-column). |
| `cas-sc` | `.cls` | contributed | *alias of* `cas-dc.cls`; Stub for Elsevier cas-dc.cls / cas-sc.cls (CAS journals double-column). |
| `CECS` | `.cls` | contributed |  |
| `ceurart` | `.cls` | contributed | Stub for CEUR-WS ceurart.cls. |
| `cimart` | `.cls` | contributed | Stub for `cimart` class (CiM = Communications in Mathematics). |
| `cimento` | `.cls` | contributed |  |
| `cjs-rcs-article` | `.cls` | contributed, raw + overrides |  |
| `clear2025` | `.cls` | contributed |  |
| `colt2024` | `.cls` | contributed | Stub for colt2024.cls (COLT 2024 — Conference on Learning Theory); also serves the sibling PMLR proceedings classes via the registry — colt2025, colt2026, l4dc2026 (Learning for Dynamics & Control), … |
| `colt2025` | `.cls` | contributed | *alias of* `colt2024.cls`; Stub for colt2024.cls (COLT 2024 — Conference on Learning Theory); also serves the sibling PMLR proceedings classes via the registry — colt2025, colt2026, l4dc2026 (Learning for Dynamics & Control), … |
| `colt2026` | `.cls` | contributed | *alias of* `colt2024.cls`; Stub for colt2024.cls (COLT 2024 — Conference on Learning Theory); also serves the sibling PMLR proceedings classes via the registry — colt2025, colt2026, l4dc2026 (Learning for Dynamics & Control), … |
| `combine` | `.cls` | contributed | Stub for combine.cls (multi-document combiner). |
| `compositio` | `.cls` | contributed |  |
| `czipreprint` | `.cls` | contributed |  |
| `czjphys` | `.cls` | contributed, raw + overrides |  |
| `daj` | `.cls` | contributed | Stub for daj.cls (Discrete Analysis journal class). |
| `Definitions/mdpi` | `.cls` | contributed | *alias of* `mdpi.cls`; Stub for MDPI journal class (Definitions/mdpi.cls, bundled by users). |
| `dmtcs-episciences` | `.cls` | contributed | Stub for dmtcs-episciences.cls (Discrete Mathematics & Theoretical Computer Science journal class, Episciences platform). |
| `ecai` | `.cls` | contributed |  |
| `egpubl` | `.cls` | contributed | Stub for egpubl.cls (Eurographics conference proceedings). |
| `ejpecp` | `.cls` | contributed | Stub for ejpecp.cls (Electronic Journal of Probability / Communications in Probability). |
| `elife` | `.cls` | contributed | Stub for elife.cls (eLife journal class). |
| `elsart` | `.cls` | ported |  |
| `elsart1p` | `.cls` | beyond Perl | *alias of* `elsart.cls` |
| `elsart3p` | `.cls` | beyond Perl | *alias of* `elsart.cls` |
| `elsart5p` | `.cls` | beyond Perl | *alias of* `elsart.cls` |
| `elsarticle` | `.cls` | ported |  |
| `emulateapj` | `.cls` | ported |  |
| `entcs` | `.cls` | contributed | *alias of* `entics.cls` |
| `entics` | `.cls` | contributed |  |
| `epl` | `.cls` | contributed | *alias of* `epl2.cls` |
| `epl2` | `.cls` | contributed |  |
| `eptcs` | `.cls` | contributed |  |
| `examdesign` | `.cls` | contributed, raw + overrides |  |
| `extarticle` | `.cls` | beyond Perl |  |
| `extbook` | `.cls` | beyond Perl |  |
| `extreport` | `.cls` | beyond Perl |  |
| `fairmeta` | `.cls` | contributed |  |
| `fcs` | `.cls` | contributed | Stub for fcs.cls (Frontiers of Computer Science). |
| `g-brief` | `.cls` | contributed, raw + overrides |  |
| `g-brief2` | `.cls` | contributed, raw + overrides |  |
| `gen-j-l` | `.cls` | ported |  |
| `gen-m-l` | `.cls` | ported |  |
| `gen-p-l` | `.cls` | ported |  |
| `gretsi` | `.cls` | contributed |  |
| `hha` | `.cls` | contributed |  |
| `hld` | `.cls` | contributed | Stub for hld.cls (inferred from the colt/midl pattern — no host `hld.cls` was on hand to confirm the exact wrapper; harmless when `\documentclass{hld}` never appears). |
| `ieeeaccess` | `.cls` | contributed | Stub for IEEE Access journal class (ieeeaccess.cls). |
| `IEEEAerospaceCLS` | `.cls` | contributed | *alias of* `IEEEapm.cls`; Stub for IEEEAerospaceCLS (IEEE Aerospace conference template). |
| `IEEEapm` | `.cls` | contributed | Stub for IEEEAerospaceCLS (IEEE Aerospace conference template). |
| `ieeecolor` | `.cls` | contributed | Stub for ieeecolor.cls (IEEE colored journal class). |
| `ieeeconf` | `.cls` | ported |  |
| `IEEEoj` | `.cls` | contributed | Stub for IEEEoj.cls (IEEE Open Journal template). |
| `IEEEojcsys` | `.cls` | contributed | Stub for IEEEojcsys.cls (IEEE Open Journal of Control Systems). |
| `IEEEtaes` | `.cls` | contributed | Stub for IEEEtaes.cls (IEEE Trans on Aerospace and Electronic Systems). |
| `IEEEtai` | `.cls` | contributed | *alias of* `IEEEapm.cls`; Stub for IEEEAerospaceCLS (IEEE Aerospace conference template). |
| `ieeetj` | `.cls` | contributed |  |
| `IEEEtran` | `.cls` | ported |  |
| `ijsra` | `.cls` | contributed, raw + overrides |  |
| `imsart` | `.cls` | contributed, raw + overrides | Stub for IMS (Institute of Mathematical Statistics) `imsart` class. |
| `informs` | `.cls` | contributed |  |
| `interact` | `.cls` | contributed | Stub for interact.cls (Taylor & Francis interact class). |
| `Interspeech` | `.cls` | contributed |  |
| `iopart` | `.cls` | ported |  |
| `iucr` | `.cls` | contributed, raw + overrides |  |
| `jcm` | `.cls` | contributed |  |
| `jfm` | `.cls` | contributed | Stub for jfm.cls (Journal of Fluid Mechanics). |
| `jgcc` | `.cls` | contributed | Stub for jgcc.cls (JGCC journal class). |
| `JHEP` | `.cls` | ported |  |
| `JHEP2` | `.cls` | ported |  |
| `JHEP3` | `.cls` | ported |  |
| `jhrs` | `.cls` | contributed |  |
| `JINST` | `.cls` | contributed |  |
| `jmlr` | `.cls` | contributed | Stub for jmlr.cls and clear2025.cls family. |
| `jpconf` | `.cls` | contributed |  |
| `just` | `.cls` | beyond Perl |  |
| `kluwer` | `.cls` | contributed |  |
| `knittingpattern` | `.cls` | contributed, raw + overrides |  |
| `l-aa` | `.cls` | contributed |  |
| `l-aa-ps` | `.cls` | contributed | *alias of* `l-aa.cls` |
| `l4dc2026` | `.cls` | contributed | *alias of* `colt2024.cls`; Stub for colt2024.cls (COLT 2024 — Conference on Learning Theory); also serves the sibling PMLR proceedings classes via the registry — colt2025, colt2026, l4dc2026 (Learning for Dynamics & Control), … |
| `laa` | `.cls` | contributed | *alias of* `l-aa.cls` |
| `lipics` | `.cls` | contributed | Stub for LIPIcs class (Dagstuhl Leibniz International Proceedings). |
| `lipics-v2019` | `.cls` | contributed | *alias of* `lipics.cls`; Stub for LIPIcs class (Dagstuhl Leibniz International Proceedings). |
| `lipics-v2021` | `.cls` | contributed | *alias of* `lipics.cls`; Stub for LIPIcs class (Dagstuhl Leibniz International Proceedings). |
| `lipics-v2024` | `.cls` | contributed | *alias of* `lipics.cls`; Stub for LIPIcs class (Dagstuhl Leibniz International Proceedings). |
| `llncs` | `.cls` | ported |  |
| `lmcs` | `.cls` | contributed | Stub for lmcs.cls (Logical Methods in Computer Science journal class). |
| `lni` | `.cls` | contributed, raw + overrides |  |
| `ltnews` | `.cls` | contributed, raw + overrides |  |
| `ltxdockit` | `.cls` | contributed |  |
| `mdpi` | `.cls` | contributed | Stub for MDPI journal class (Definitions/mdpi.cls, bundled by users). |
| `memoir` | `.cls` | contributed, raw + overrides |  |
| `midl` | `.cls` | contributed | Stub for midl.cls (MIDL — Medical Imaging with Deep Learning), a jmlr derivative. |
| `minimal` | `.cls` | ported |  |
| `mn` | `.cls` | ported |  |
| `mn2e` | `.cls` | ported |  |
| `mnras` | `.cls` | ported |  |
| `moderncv` | `.cls` | ported |  |
| `modernposter` | `.cls` | contributed |  |
| `moduli` | `.cls` | contributed |  |
| `MRM` | `.cls` | contributed |  |
| `nature-pre` | `.cls` | contributed |  |
| `nature_mod` | `.cls` | contributed | *alias of* `nature-pre.cls` |
| `neus2025` | `.cls` | contributed | *alias of* `colt2024.cls`; Stub for colt2024.cls (COLT 2024 — Conference on Learning Theory); also serves the sibling PMLR proceedings classes via the registry — colt2025, colt2026, l4dc2026 (Learning for Dynamics & Control), … |
| `nic-series` | `.cls` | contributed |  |
| `OmniBus` | `.cls` | ported |  |
| `openmoss` | `.cls` | contributed |  |
| `optica-article` | `.cls` | contributed | Stub for optica-article.cls (Optica/OSA journal class). |
| `oup-authoring-template` | `.cls` | contributed | Stub for oup-authoring-template.cls (Oxford University Press journal template). |
| `pasa` | `.cls` | contributed |  |
| `pasj00` | `.cls` | contributed, raw + overrides |  |
| `pasj01` | `.cls` | contributed |  |
| `pasj02` | `.cls` | contributed |  |
| `pnas-new` | `.cls` | contributed | Stub for pnas-new.cls (PNAS journal class). |
| `PoS` | `.cls` | ported |  |
| `ptephy` | `.cls` | contributed | *alias of* `ptephy_v1.cls`; Stub for ptephy.cls (Progress of Theoretical and Experimental Physics). |
| `ptephy_v1` | `.cls` | contributed | Stub for ptephy.cls (Progress of Theoretical and Experimental Physics). |
| `ptephy_v2` | `.cls` | contributed | *alias of* `ptephy_v1.cls`; Stub for ptephy.cls (Progress of Theoretical and Experimental Physics). |
| `quantumarticle` | `.cls` | ported |  |
| `quantumview` | `.cls` | beyond Perl, raw + overrides |  |
| `rasti` | `.cls` | contributed |  |
| `report` | `.cls` | ported |  |
| `resphilosophica` | `.cls` | contributed, raw + overrides |  |
| `revtex` | `.cls` | ported |  |
| `revtex4` | `.cls` | ported |  |
| `revtex4-1` | `.cls` | ported |  |
| `revtex4-2` | `.cls` | ported | *alias of* `revtex4.cls` |
| `rmaa` | `.cls` | contributed |  |
| `sagej` | `.cls` | contributed | Stub for sagej.cls (SAGE journals). |
| `SCIS2024` | `.cls` | contributed | Stub for SCIS2024.cls (Science China Information Sciences 2024). |
| `scrartcl` | `.cls` | contributed, raw + overrides |  |
| `scrbook` | `.cls` | contributed, raw + overrides |  |
| `scrreprt` | `.cls` | contributed, raw + overrides |  |
| `selfevolagent` | `.cls` | contributed |  |
| `siamart` | `.cls` | contributed | Stub binding for SIAM siamart-family classes (siamart, siamart220329, ...). |
| `siamltex` | `.cls` | contributed |  |
| `siamonline` | `.cls` | contributed | *alias of* `siamart.cls`; Stub binding for SIAM siamart-family classes (siamart, siamart220329, ...). |
| `sigma` | `.cls` | contributed | Stub for sigma.cls (SIGMA journal — Symmetry, Integrability, Geometry). |
| `slides` | `.cls` | ported |  |
| `smc_ieeeconf` | `.cls` | contributed | Stub for smc_ieeeconf.cls (IEEE SMC conference class, a IEEEconf variant). |
| `smfart` | `.cls` | contributed, raw + overrides |  |
| `sn-jnl` | `.cls` | contributed | Stub for sn-jnl.cls (Springer Nature journal class). |
| `SNmult` | `.cls` | beyond Perl | *alias of* `svmono.cls` |
| `spie` | `.cls` | contributed | Stub for spie.cls (SPIE conference proceedings). |
| `standalone` | `.cls` | ported, raw + overrides |  |
| `subfiles` | `.cls` | ported |  |
| `svjour` | `.cls` | ported |  |
| `svjour1` | `.cls` | ported |  |
| `svjour2` | `.cls` | ported | *alias of* `svjour1.cls` |
| `svjour3` | `.cls` | ported | *alias of* `svjour1.cls` |
| `svmono` | `.cls` | beyond Perl |  |
| `svmult` | `.cls` | ported | *alias of* `svmono.cls` |
| `svproc` | `.cls` | contributed |  |
| `tac` | `.cls` | contributed | Stub for tac.cls (Theory and Applications of Categories journal). |
| `tlp` | `.cls` | contributed, raw + overrides |  |
| `uai2025` | `.cls` | contributed | Stub for uai2025.cls (UAI conference class). |
| `uiucthesis` | `.cls` | contributed, raw + overrides |  |
| `uspatent` | `.cls` | contributed, raw + overrides |  |
| `ut-thesis` | `.cls` | contributed |  |
| `vmsta` | `.cls` | contributed |  |
| `webofc` | `.cls` | contributed |  |
| `WileyASNA-v1` | `.cls` | contributed |  |
| `WileyMSP-template` | `.cls` | contributed | Stub for WileyMSP-template.cls (Wiley Mathematical Sciences Publishers). |
| `WileyNJD-v1` | `.cls` | contributed |  |
| `WileyNJD-v2` | `.cls` | contributed | *alias of* `WileyNJD-v1.cls` |
| `WileyNJDv5` | `.cls` | contributed | *alias of* `WileyNJD-v1.cls` |
| `wileyNJDv5` | `.cls` | contributed | *alias of* `WileyNJD-v1.cls` |
| `wlpeerj` | `.cls` | contributed | Stub for wlpeerj.cls (Wiley PeerJ template). |
| `ws-ijgmmp` | `.cls` | contributed | Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.). |
| `ws-ijmpa` | `.cls` | contributed | *alias of* `ws-ijgmmp.cls`; Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.). |
| `ws-ijmpb` | `.cls` | contributed | *alias of* `ws-ijgmmp.cls`; Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.). |
| `ws-ijmpc` | `.cls` | contributed | *alias of* `ws-ijgmmp.cls`; Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.). |
| `ws-ijmpd` | `.cls` | contributed | *alias of* `ws-ijgmmp.cls`; Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.). |
| `ws-ijmpe` | `.cls` | contributed | *alias of* `ws-ijgmmp.cls`; Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.). |
| `ws-mpla` | `.cls` | contributed | *alias of* `ws-ijgmmp.cls`; Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.). |
| `ws-mplb` | `.cls` | contributed | *alias of* `ws-ijgmmp.cls`; Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.). |
| `ws-p8-50x6-00` | `.cls` | contributed |  |
| `ws-procs961x669` | `.cls` | contributed | *alias of* `ws-rv9x6.cls` |
| `ws-procs9x6` | `.cls` | contributed | *alias of* `ws-rv9x6.cls` |
| `ws-rv9x6` | `.cls` | contributed |  |

## Packages (670)

| Name | Kind | Support | Note |
|---|---|---|---|
| `a0size` | `.sty` | ported |  |
| `a4` | `.sty` | ported |  |
| `a4wide` | `.sty` | ported |  |
| `aa_support` | `.sty` | ported |  |
| `aas2pp4` | `.sty` | beyond Perl | *alias of* `aaspp.sty` |
| `aas_macros` | `.sty` | ported |  |
| `aas_support` | `.sty` | ported |  |
| `aasms` | `.sty` | ported |  |
| `aaspp` | `.sty` | ported |  |
| `aaspptwo` | `.sty` | beyond Perl | *alias of* `aaspp.sty` |
| `aastex` | `.sty` | ported |  |
| `abntex2cite` | `.sty` | contributed, raw + overrides |  |
| `accents` | `.sty` | ported |  |
| `acronym` | `.sty` | ported |  |
| `actuarialangle` | `.sty` | ported, raw + overrides |  |
| `adjmulticol` | `.sty` | beyond Perl |  |
| `adjustbox` | `.sty` | ported, raw + overrides |  |
| `ae` | `.sty` | ported |  |
| `aecompl` | `.sty` | ported |  |
| `afterpage` | `.sty` | ported |  |
| `aipproc` | `.sty` | ported |  |
| `aistats` | `.sty` | contributed, raw + overrides |  |
| `aistats2026` | `.sty` | contributed, raw + overrides | *alias of* `aistats.sty` |
| `aistats2e` | `.sty` | contributed, raw + overrides | *alias of* `aistats.sty` |
| `aistats2eM` | `.sty` | contributed, raw + overrides | *alias of* `aistats.sty` |
| `aj_pt4` | `.sty` | beyond Perl | *alias of* `aaspp.sty` |
| `ajmacros` | `.sty` | beyond Perl |  |
| `AlegreyaSans` | `.sty` | contributed, raw + overrides |  |
| `algc` | `.sty` | ported, raw + overrides |  |
| `algcompatible` | `.sty` | ported, raw + overrides |  |
| `algmatlab` | `.sty` | ported, raw + overrides |  |
| `algorithm` | `.sty` | ported, raw + overrides |  |
| `algorithm2e` | `.sty` | ported, raw + overrides |  |
| `algorithmic` | `.sty` | ported, raw + overrides |  |
| `algorithmicx` | `.sty` | ported, raw + overrides |  |
| `algpascal` | `.sty` | ported, raw + overrides |  |
| `algpseudocode` | `.sty` | ported, raw + overrides |  |
| `algpseudocodex` | `.sty` | contributed |  |
| `aliascnt` | `.sty` | contributed, raw + overrides |  |
| `aligned-overset` | `.sty` | contributed |  |
| `alltt` | `.sty` | ported |  |
| `ams_support` | `.sty` | ported |  |
| `amsaddr` | `.sty` | ported |  |
| `amsbsy` | `.sty` | ported |  |
| `amscd` | `.sty` | ported |  |
| `amsfonts` | `.sty` | ported |  |
| `amsgen` | `.sty` | ported |  |
| `amsmath` | `.sty` | ported |  |
| `amsopn` | `.sty` | ported |  |
| `amsppt` | `.sty` | ported |  |
| `amsrefs` | `.sty` | ported |  |
| `amssymb` | `.sty` | ported |  |
| `amstex` | `.sty` | ported |  |
| `amstext` | `.sty` | ported |  |
| `amsthm` | `.sty` | ported |  |
| `amsxtra` | `.sty` | ported |  |
| `animate` | `.sty` | contributed |  |
| `apacite` | `.sty` | contributed |  |
| `apjfonts` | `.sty` | ported |  |
| `apjpt4` | `.sty` | beyond Perl | *alias of* `aaspp.sty` |
| `appendix` | `.sty` | ported |  |
| `apxproof` | `.sty` | contributed, raw + overrides |  |
| `ar5iv` | `.sty` | contributed |  |
| `array` | `.sty` | ported |  |
| `arxiv` | `.sty` | contributed, raw + overrides |  |
| `arydshln` | `.sty` | contributed |  |
| `ascmac` | `.sty` | contributed | Stub for ascmac.sty — a Japanese LaTeX2e add-on (part of `jsclasses` family) that provides boxed environments such as `{itembox}`, `{screen}`, `{shadebox}`, `{boxnote}` (TeX Live: … |
| `assoccnt` | `.sty` | contributed |  |
| `attachfile` | `.sty` | ported |  |
| `atveryend` | `.sty` | ported |  |
| `authblk` | `.sty` | ported |  |
| `autobreak` | `.sty` | contributed |  |
| `autofe` | `.sty` | contributed | No-op stub for autofe.sty (ucs's "Automatic switching of font encodings"). |
| `autonum` | `.sty` | contributed |  |
| `auxhook` | `.sty` | ported |  |
| `avant` | `.sty` | ported |  |
| `axessibility` | `.sty` | contributed |  |
| `axodraw` | `.sty` | beyond Perl |  |
| `axodraw2` | `.sty` | beyond Perl, raw + overrides |  |
| `babel` | `.sty` | ported, raw + overrides |  |
| `babel_support` | `.sty` | ported |  |
| `backref` | `.sty` | contributed |  |
| `balance` | `.sty` | ported |  |
| `bbding` | `.sty` | ported, raw + overrides |  |
| `bbm` | `.sty` | ported |  |
| `bbold` | `.sty` | ported |  |
| `beton` | `.sty` | ported |  |
| `bezier` | `.sty` | ported |  |
| `bfhlayout` | `.sty` | contributed, raw + overrides |  |
| `bibentry` | `.sty` | contributed |  |
| `biblatex` | `.sty` | contributed, raw + overrides |  |
| `bibunits` | `.sty` | ported |  |
| `bidi` | `.sty` | contributed |  |
| `bigintcalc` | `.sty` | ported, raw + overrides |  |
| `bigstrut` | `.sty` | beyond Perl |  |
| `bitset` | `.sty` | ported, raw + overrides |  |
| `blindtext` | `.sty` | ported, raw + overrides |  |
| `blkarray` | `.sty` | beyond Perl |  |
| `bm` | `.sty` | ported |  |
| `bmvc2k_natbib` | `.sty` | contributed |  |
| `bookman` | `.sty` | ported |  |
| `bookmark` | `.sty` | ported |  |
| `booktabs` | `.sty` | ported |  |
| `boxedminipage` | `.sty` | ported, raw + overrides |  |
| `braket` | `.sty` | ported |  |
| `breakurl` | `.sty` | ported |  |
| `breqn` | `.sty` | contributed |  |
| `bussproofs` | `.sty` | contributed, raw + overrides |  |
| `calc` | `.sty` | ported |  |
| `calrsfs` | `.sty` | ported |  |
| `cancel` | `.sty` | ported |  |
| `capt-of` | `.sty` | contributed |  |
| `caption` | `.sty` | ported |  |
| `caption2` | `.sty` | contributed |  |
| `cases` | `.sty` | ported |  |
| `catchfile` | `.sty` | contributed |  |
| `causets` | `.sty` | ported, raw + overrides |  |
| `ccaption` | `.sty` | contributed |  |
| `ccfonts` | `.sty` | ported |  |
| `cclayout` | `.sty` | contributed |  |
| `centernot` | `.sty` | contributed |  |
| `chancery` | `.sty` | ported |  |
| `changepage` | `.sty` | contributed, raw + overrides |  |
| `changes` | `.sty` | contributed |  |
| `chapterbib` | `.sty` | ported |  |
| `charter` | `.sty` | ported |  |
| `chemformula` | `.sty` | contributed, raw + overrides |  |
| `chemgreek` | `.sty` | beyond Perl, raw + overrides |  |
| `chemmacros` | `.sty` | beyond Perl, raw + overrides |  |
| `chemnum` | `.sty` | contributed |  |
| `chngcntr` | `.sty` | ported |  |
| `chngpage` | `.sty` | contributed, raw + overrides |  |
| `circuitikz` | `.sty` | ported, raw + overrides |  |
| `cite` | `.sty` | ported |  |
| `citesort` | `.sty` | ported |  |
| `CJK` | `.sty` | contributed |  |
| `CJKutf8` | `.sty` | contributed |  |
| `clefval` | `.sty` | contributed, raw + overrides |  |
| `cleveref` | `.sty` | ported, raw + overrides |  |
| `cmap` | `.sty` | ported |  |
| `cmbright` | `.sty` | ported |  |
| `cmcal` | `.sty` | contributed |  |
| `cmpj` | `.sty` | beyond Perl, raw + overrides |  |
| `cmpj2` | `.sty` | beyond Perl, raw + overrides |  |
| `cmpj3` | `.sty` | beyond Perl, raw + overrides |  |
| `codehigh` | `.sty` | beyond Perl, raw + overrides |  |
| `collcell` | `.sty` | contributed |  |
| `colm` | `.sty` | contributed | *alias of* `colm2025_conference.sty`; Stub for colm2025_conference.sty (COLM 2025 conference template), and the COLM family binding: registered also as `colm`, which the versioned fallback reaches from colm2024_conference, colm2024_arxiv … |
| `colm2025_conference` | `.sty` | contributed | Stub for colm2025_conference.sty (COLM 2025 conference template), and the COLM family binding: registered also as `colm`, which the versioned fallback reaches from colm2024_conference, colm2024_arxiv … |
| `color` | `.sty` | ported |  |
| `colordvi` | `.sty` | ported |  |
| `colortbl` | `.sty` | ported |  |
| `commath` | `.sty` | contributed, raw + overrides |  |
| `comment` | `.sty` | ported |  |
| `concmath` | `.sty` | ported |  |
| `constants` | `.sty` | beyond Perl | No-op stub for constants.sty. |
| `courier` | `.sty` | ported |  |
| `cprotect` | `.sty` | contributed |  |
| `crckapb` | `.sty` | contributed |  |
| `crop` | `.sty` | ported |  |
| `cropmark` | `.sty` | ported |  |
| `crossreftools` | `.sty` | contributed, raw + overrides |  |
| `csquotes` | `.sty` | ported, raw + overrides |  |
| `ctable` | `.sty` | beyond Perl |  |
| `currfile` | `.sty` | beyond Perl, raw + overrides |  |
| `curve2e` | `.sty` | contributed, raw + overrides |  |
| `cuted` | `.sty` | contributed, raw + overrides |  |
| `cvpr` | `.sty` | contributed, raw + overrides | *alias of* `wacv.sty`; Stub for cvpr.sty / iccv.sty / iccvw.sty (computer vision conference style). |
| `cvpr2023` | `.sty` | contributed, raw + overrides | *alias of* `wacv.sty`; Stub for cvpr.sty / iccv.sty / iccvw.sty (computer vision conference style). |
| `cvpr2024` | `.sty` | contributed, raw + overrides | *alias of* `wacv.sty`; Stub for cvpr.sty / iccv.sty / iccvw.sty (computer vision conference style). |
| `cvpr2025` | `.sty` | contributed, raw + overrides | *alias of* `wacv.sty`; Stub for cvpr.sty / iccv.sty / iccvw.sty (computer vision conference style). |
| `datatool` | `.sty` | beyond Perl, raw + overrides |  |
| `datatool-base` | `.sty` | beyond Perl, raw + overrides |  |
| `datetime` | `.sty` | contributed |  |
| `datetime2` | `.sty` | contributed, raw + overrides |  |
| `dblfloatfix` | `.sty` | contributed |  |
| `dcolumn` | `.sty` | ported |  |
| `delarray` | `.sty` | beyond Perl |  |
| `deluxe` | `.sty` | contributed |  |
| `deluxetable` | `.sty` | ported |  |
| `derivative` | `.sty` | contributed, raw + overrides |  |
| `dgruyter` | `.sty` | contributed |  |
| `dhucs` | `.sty` | contributed, raw + overrides |  |
| `diagbox` | `.sty` | ported |  |
| `diagrams` | `.sty` | contributed |  |
| `directory` | `.sty` | contributed, raw + overrides |  |
| `dirtytalk` | `.sty` | ported |  |
| `doclicense` | `.sty` | contributed, raw + overrides |  |
| `docmute` | `.sty` | beyond Perl |  |
| `doi` | `.sty` | ported, raw + overrides |  |
| `doublespace` | `.sty` | ported |  |
| `dsfont` | `.sty` | ported |  |
| `ed` | `.sty` | ported, raw + overrides |  |
| `ellipsis` | `.sty` | ported |  |
| `elsart` | `.sty` | ported |  |
| `elsart_support` | `.sty` | ported |  |
| `elsart_support_core` | `.sty` | ported |  |
| `emlines` | `.sty` | contributed |  |
| `empheq` | `.sty` | ported |  |
| `emulateapj` | `.sty` | ported |  |
| `emulateapj5` | `.sty` | ported |  |
| `endnotes` | `.sty` | ported |  |
| `english` | `.sty` | ported, raw + overrides |  |
| `enotez` | `.sty` | beyond Perl, raw + overrides |  |
| `enumerate` | `.sty` | ported |  |
| `enumitem` | `.sty` | ported |  |
| `envmath` | `.sty` | contributed |  |
| `epigraph` | `.sty` | ported |  |
| `epsf` | `.sty` | ported | *alias of* `epsf.tex` |
| `epsfig` | `.sty` | ported | *alias of* `psfig.tex` |
| `epstopdf` | `.sty` | ported |  |
| `eqnarray` | `.sty` | beyond Perl |  |
| `equations` | `.sty` | contributed |  |
| `esint` | `.sty` | ported |  |
| `eso-pic` | `.sty` | contributed |  |
| `espcrc` | `.sty` | ported |  |
| `etex` | `.sty` | ported |  |
| `etoc` | `.sty` | beyond Perl, raw + overrides |  |
| `etoolbox` | `.sty` | ported |  |
| `eucal` | `.sty` | ported |  |
| `eufrak` | `.sty` | ported |  |
| `euler` | `.sty` | ported |  |
| `eulervm` | `.sty` | ported |  |
| `eurosym` | `.sty` | ported |  |
| `euscript` | `.sty` | ported |  |
| `everyshi` | `.sty` | beyond Perl |  |
| `expl3` | `.sty` | ported, raw + overrides |  |
| `exscale` | `.sty` | ported |  |
| `fancybox` | `.sty` | ported, raw + overrides |  |
| `fancyhdr` | `.sty` | ported |  |
| `fancyheadings` | `.sty` | ported |  |
| `fancyvrb` | `.sty` | ported, raw + overrides |  |
| `fancyvrb-ex` | `.sty` | contributed, raw + overrides |  |
| `fdsymbol` | `.sty` | beyond Perl |  |
| `feynmf` | `.sty` | ported, raw + overrides |  |
| `feynmp` | `.sty` | beyond Perl, raw + overrides |  |
| `figbib` | `.sty` | contributed, raw + overrides |  |
| `figcaps` | `.sty` | contributed |  |
| `filehook` | `.sty` | ported, raw + overrides |  |
| `fix-cm` | `.sty` | ported |  |
| `fixltx2e` | `.sty` | ported |  |
| `fixme` | `.sty` | ported, raw + overrides |  |
| `flafter` | `.sty` | ported |  |
| `fleqn` | `.sty` | ported |  |
| `float` | `.sty` | ported |  |
| `floatfig` | `.sty` | ported |  |
| `floatflt` | `.sty` | ported |  |
| `floatpag` | `.sty` | ported |  |
| `floatrow` | `.sty` | beyond Perl, raw + overrides |  |
| `flowchart` | `.sty` | ported, raw + overrides |  |
| `flushend` | `.sty` | ported |  |
| `fontawesome` | `.sty` | contributed |  |
| `fontawesome5` | `.sty` | contributed |  |
| `fontenc` | `.sty` | ported |  |
| `fontspec` | `.sty` | ported |  |
| `footmisc` | `.sty` | ported |  |
| `footnote` | `.sty` | ported |  |
| `forest` | `.sty` | contributed |  |
| `fourier` | `.sty` | ported |  |
| `fp` | `.sty` | contributed, raw + overrides |  |
| `framed` | `.sty` | ported |  |
| `frontespizio` | `.sty` | beyond Perl, raw + overrides |  |
| `fullname` | `.sty` | contributed |  |
| `fullpage` | `.sty` | ported |  |
| `fvextra` | `.sty` | beyond Perl, raw + overrides |  |
| `gauss` | `.sty` | contributed |  |
| `gensymb` | `.sty` | ported |  |
| `geometry` | `.sty` | ported |  |
| `german` | `.sty` | ported |  |
| `getfiledate` | `.sty` | contributed | Stub for getfiledate.sty — prints a file's last-modification date (a boxed/inline date string). |
| `gettitlestring` | `.sty` | ported, raw + overrides |  |
| `glossaries` | `.sty` | ported, raw + overrides |  |
| `graphics` | `.sty` | ported |  |
| `graphicx` | `.sty` | ported |  |
| `grffile` | `.sty` | ported |  |
| `helvet` | `.sty` | ported |  |
| `hepnames` | `.sty` | contributed, raw + overrides |  |
| `hepparticles` | `.sty` | contributed, raw + overrides |  |
| `hepunits` | `.sty` | ported, raw + overrides |  |
| `here` | `.sty` | ported |  |
| `hhline` | `.sty` | ported |  |
| `hpstatement` | `.sty` | contributed, raw + overrides |  |
| `html` | `.sty` | ported |  |
| `hvfloat` | `.sty` | contributed |  |
| `hypcap` | `.sty` | ported |  |
| `hypdestopt` | `.sty` | contributed |  |
| `hyperref` | `.sty` | ported |  |
| `hyperxmp` | `.sty` | ported |  |
| `hyphenat` | `.sty` | contributed |  |
| `iccv` | `.sty` | contributed, raw + overrides | Stub for iccv.sty / iccvw.sty (ICCV conference style). |
| `iccvw` | `.sty` | contributed, raw + overrides | *alias of* `iccv.sty`; Stub for iccv.sty / iccvw.sty (ICCV conference style). |
| `icml` | `.sty` | ported |  |
| `icml2016` | `.sty` | ported |  |
| `icml2017` | `.sty` | ported | *alias of* `icml2016.sty` |
| `icml2018` | `.sty` | ported | *alias of* `icml2016.sty` |
| `icml2019` | `.sty` | ported, raw + overrides |  |
| `icml2020` | `.sty` | ported | *alias of* `icml2016.sty` |
| `icml2021` | `.sty` | ported | *alias of* `icml2016.sty` |
| `icml2022` | `.sty` | ported | *alias of* `icml2016.sty` |
| `icml2023` | `.sty` | ported | *alias of* `icml2016.sty` |
| `icml2024` | `.sty` | ported | *alias of* `icml2016.sty` |
| `icml2025` | `.sty` | ported | *alias of* `icml2016.sty` |
| `icml_support` | `.sty` | ported |  |
| `IEEEtrantools` | `.sty` | beyond Perl |  |
| `ifdraft` | `.sty` | ported |  |
| `ifetex` | `.sty` | ported |  |
| `ifluatex` | `.sty` | ported |  |
| `ifoddpage` | `.sty` | contributed, raw + overrides |  |
| `ifpdf` | `.sty` | ported |  |
| `ifplatform` | `.sty` | ported |  |
| `ifsym` | `.sty` | ported, raw + overrides |  |
| `iftex` | `.sty` | ported |  |
| `ifthen` | `.sty` | ported, raw + overrides |  |
| `ifvtex` | `.sty` | ported |  |
| `ifxetex` | `.sty` | ported |  |
| `ijcai` | `.sty` | ported |  |
| `imakeidx` | `.sty` | contributed, raw + overrides |  |
| `import` | `.sty` | ported |  |
| `ims_support` | `.sty` | beyond Perl |  |
| `imsart` | `.sty` | contributed |  |
| `indentfirst` | `.sty` | ported |  |
| `infwarerr` | `.sty` | ported, raw + overrides |  |
| `inputenc` | `.sty` | ported |  |
| `inst_support` | `.sty` | ported |  |
| `intcalc` | `.sty` | ported, raw + overrides |  |
| `INTERSPEECH2021` | `.sty` | contributed |  |
| `INTERSPEECH2022` | `.sty` | contributed | *alias of* `INTERSPEECH2021.sty` |
| `INTERSPEECH2023` | `.sty` | contributed | *alias of* `INTERSPEECH2021.sty` |
| `iopams` | `.sty` | ported |  |
| `iopart_support` | `.sty` | ported |  |
| `isorot` | `.sty` | beyond Perl |  |
| `jabbrv` | `.sty` | contributed | Stub for jabbrv.sty (journal-abbreviation package). |
| `jair` | `.sty` | contributed | Stub for jair.sty (Journal of AI Research style). |
| `jcappub` | `.sty` | contributed |  |
| `jheppub` | `.sty` | ported |  |
| `jinstpub` | `.sty` | contributed |  |
| `jmlr2e` | `.sty` | contributed | Stub for jmlr2e.sty / jmlr2e_preprint.sty (JMLR / PMLR author-block macros). |
| `jmlr2e_preprint` | `.sty` | contributed | *alias of* `jmlr2e.sty`; Stub for jmlr2e.sty / jmlr2e_preprint.sty (JMLR / PMLR author-block macros). |
| `jpc` | `.sty` | contributed |  |
| `jurabib` | `.sty` | contributed, raw + overrides |  |
| `keysetopt` | `.sty` | contributed |  |
| `keytheorems` | `.sty` | contributed, raw + overrides |  |
| `keyval` | `.sty` | ported, raw + overrides |  |
| `kotexutf` | `.sty` | contributed, raw + overrides |  |
| `kvdefinekeys` | `.sty` | ported, raw + overrides |  |
| `kvoptions` | `.sty` | ported, raw + overrides |  |
| `kvsetkeys` | `.sty` | ported, raw + overrides |  |
| `l3draw` | `.sty` | contributed |  |
| `l3keys2e` | `.sty` | ported, raw + overrides |  |
| `lastpage` | `.sty` | ported |  |
| `latex-lab-testphase-block` | `.sty` | beyond Perl |  |
| `latex-lab-testphase-minipage` | `.sty` | beyond Perl |  |
| `latexml` | `.sty` | ported |  |
| `latexmlman` | `.sty` | contributed |  |
| `latexrelease` | `.sty` | contributed | Stub for latexrelease.sty (LaTeX kernel rollback machinery). |
| `latexsym` | `.sty` | ported |  |
| `letltxmacro` | `.sty` | contributed, raw + overrides |  |
| `lettrine` | `.sty` | contributed |  |
| `libertine` | `.sty` | contributed |  |
| `libertinust1math` | `.sty` | beyond Perl |  |
| `lineno` | `.sty` | ported |  |
| `lipsum` | `.sty` | ported, raw + overrides |  |
| `listings` | `.sty` | ported |  |
| `listingsutf8` | `.sty` | ported |  |
| `lmodern` | `.sty` | ported |  |
| `longtable` | `.sty` | ported |  |
| `lscape` | `.sty` | ported |  |
| `ltablex` | `.sty` | contributed |  |
| `ltxcmds` | `.sty` | ported, raw + overrides |  |
| `ltxtable` | `.sty` | contributed |  |
| `lua-widow-control` | `.sty` | beyond Perl |  |
| `luacode` | `.sty` | contributed |  |
| `luatexbase` | `.sty` | contributed |  |
| `luximono` | `.sty` | ported |  |
| `lxRDFa` | `.sty` | ported |  |
| `lxtesttemplate` | `.sty` | contributed |  |
| `makecell` | `.sty` | ported, raw + overrides |  |
| `makeidx` | `.sty` | ported |  |
| `marginnote` | `.sty` | ported |  |
| `marvosym` | `.sty` | ported |  |
| `mathabx` | `.sty` | ported |  |
| `mathastext` | `.sty` | beyond Perl |  |
| `mathbbol` | `.sty` | ported |  |
| `mathdots` | `.sty` | ported |  |
| `mathenv` | `.sty` | beyond Perl, raw + overrides |  |
| `mathpartir` | `.sty` | contributed | Stub the public API as faithful enough wrappers — inference rules render as a vertical `numerator / line / denominator` frac-style — so the raw file is skipped and the document continues. |
| `mathpazo` | `.sty` | ported |  |
| `mathpple` | `.sty` | ported |  |
| `mathptm` | `.sty` | ported |  |
| `mathptmx` | `.sty` | ported |  |
| `mathrsfs` | `.sty` | ported |  |
| `mathscinet` | `.sty` | beyond Perl |  |
| `mathspec` | `.sty` | contributed |  |
| `mathtools` | `.sty` | ported |  |
| `mciteplus` | `.sty` | contributed |  |
| `mdframed` | `.sty` | contributed |  |
| `mdwtab` | `.sty` | beyond Perl |  |
| `mfirstuc` | `.sty` | beyond Perl, raw + overrides |  |
| `mhchem` | `.sty` | contributed, raw + overrides |  |
| `mhsetup` | `.sty` | beyond Perl, raw + overrides |  |
| `microtype` | `.sty` | ported |  |
| `minted` | `.sty` | contributed |  |
| `mleftright` | `.sty` | ported |  |
| `mn2e_support` | `.sty` | ported |  |
| `MnSymbol` | `.sty` | contributed |  |
| `morefloats` | `.sty` | contributed | Stub for morefloats.sty — raises the limit on the number of *unprocessed* floats LaTeX can hold in its internal queue. |
| `mprocl` | `.sty` | contributed |  |
| `multibib` | `.sty` | contributed, raw + overrides |  |
| `multicol` | `.sty` | ported |  |
| `multido` | `.sty` | ported |  |
| `multirow` | `.sty` | ported |  |
| `nag` | `.sty` | contributed |  |
| `nameref` | `.sty` | ported |  |
| `natbib` | `.sty` | ported |  |
| `needspace` | `.sty` | contributed, raw + overrides |  |
| `neurips` | `.sty` | ported |  |
| `neurips_2019` | `.sty` | ported | *alias of* `neurips.sty` |
| `neurips_2020` | `.sty` | ported | *alias of* `neurips.sty` |
| `neurips_2021` | `.sty` | ported | *alias of* `neurips.sty` |
| `neurips_2022` | `.sty` | ported | *alias of* `neurips.sty` |
| `neurips_2023` | `.sty` | ported | *alias of* `neurips.sty` |
| `neurips_2024` | `.sty` | ported | *alias of* `neurips.sty` |
| `neurips_2025` | `.sty` | ported | *alias of* `neurips.sty` |
| `neurips_2026` | `.sty` | ported | *alias of* `neurips.sty` |
| `newcent` | `.sty` | ported |  |
| `newclude` | `.sty` | beyond Perl |  |
| `newfloat` | `.sty` | ported |  |
| `newlfont` | `.sty` | ported |  |
| `newpxmath` | `.sty` | contributed |  |
| `newtxmath` | `.sty` | ported |  |
| `newtxtext` | `.sty` | ported |  |
| `newverbs` | `.sty` | beyond Perl |  |
| `ngerman` | `.sty` | ported | *alias of* `ngerman.ldf` |
| `nicefrac` | `.sty` | ported |  |
| `nicematrix` | `.sty` | contributed |  |
| `NJDnatbib` | `.sty` | contributed |  |
| `nlctuserguide` | `.sty` | contributed, raw + overrides |  |
| `nmbib` | `.sty` | contributed, raw + overrides |  |
| `nomencl` | `.sty` | ported, raw + overrides |  |
| `nomentbl` | `.sty` | contributed, raw + overrides |  |
| `nopageno` | `.sty` | ported |  |
| `ntheorem` | `.sty` | ported, raw + overrides |  |
| `numprint` | `.sty` | ported, raw + overrides |  |
| `ol2` | `.sty` | contributed |  |
| `oldgerm` | `.sty` | contributed |  |
| `oldlfont` | `.sty` | contributed |  |
| `opex3` | `.sty` | contributed |  |
| `orcidlink` | `.sty` | ported |  |
| `osajnl` | `.sty` | contributed |  |
| `osajnl2` | `.sty` | contributed |  |
| `overpic` | `.sty` | ported |  |
| `palatino` | `.sty` | ported |  |
| `paralist` | `.sty` | ported |  |
| `parskip` | `.sty` | ported, raw + overrides |  |
| `paspconf` | `.sty` | contributed |  |
| `pax` | `.sty` | contributed | Stub the package as a no-op shell. |
| `pb-diagram` | `.sty` | contributed, raw + overrides |  |
| `pdfcol` | `.sty` | beyond Perl |  |
| `pdfcomment` | `.sty` | contributed |  |
| `pdflscape` | `.sty` | ported |  |
| `pdfmanagement` | `.sty` | beyond Perl, raw + overrides |  |
| `pdfmanagement-testphase` | `.sty` | beyond Perl, raw + overrides |  |
| `pdfpages` | `.sty` | ported |  |
| `pdfsync` | `.sty` | ported |  |
| `pdftexcmds` | `.sty` | ported |  |
| `pdfx` | `.sty` | beyond Perl | Stub as no-op. |
| `pgf` | `.sty` | ported, raw + overrides |  |
| `pgfkeys` | `.sty` | ported |  |
| `pgfmath` | `.sty` | ported |  |
| `pgfplots` | `.sty` | ported, raw + overrides |  |
| `pgfplotstable` | `.sty` | ported, raw + overrides |  |
| `pgfrcs` | `.sty` | ported |  |
| `phy-ab.braket` | `.sty` | contributed, raw + overrides |  |
| `physics` | `.sty` | ported |  |
| `pict2e` | `.sty` | beyond Perl |  |
| `pifont` | `.sty` | ported |  |
| `pinlabel` | `.sty` | contributed, raw + overrides |  |
| `piton` | `.sty` | contributed |  |
| `placeins` | `.sty` | ported |  |
| `polyglossia` | `.sty` | beyond Perl, raw + overrides |  |
| `pos` | `.sty` | beyond Perl, raw + overrides |  |
| `prettyref` | `.sty` | ported, raw + overrides |  |
| `preview` | `.sty` | ported |  |
| `PRIMEarxiv` | `.sty` | contributed, raw + overrides |  |
| `program` | `.sty` | contributed, raw + overrides |  |
| `projlib-author` | `.sty` | contributed, raw + overrides |  |
| `proof` | `.sty` | ported |  |
| `proofwiki` | `.sty` | ported |  |
| `psfig` | `.sty` | ported |  |
| `psfrag` | `.sty` | ported |  |
| `pslatex` | `.sty` | ported |  |
| `pspicture` | `.sty` | ported |  |
| `pst-all` | `.sty` | contributed |  |
| `pst-grad` | `.sty` | ported, raw + overrides |  |
| `pst-node` | `.sty` | ported |  |
| `pst-plot` | `.sty` | contributed |  |
| `pstricks` | `.sty` | ported, raw + overrides |  |
| `pstricks_support` | `.sty` | ported |  |
| `pxfonts` | `.sty` | ported |  |
| `quotchap` | `.sty` | contributed, raw + overrides |  |
| `ragged2e` | `.sty` | ported |  |
| `rccol` | `.sty` | contributed, raw + overrides |  |
| `refcount` | `.sty` | ported, raw + overrides |  |
| `refstyle` | `.sty` | contributed, raw + overrides |  |
| `relsize` | `.sty` | ported |  |
| `remreset` | `.sty` | ported |  |
| `revsymb` | `.sty` | ported |  |
| `revsymb4-1` | `.sty` | ported | *alias of* `revsymb.sty` |
| `revtex` | `.sty` | ported |  |
| `revtex3_support` | `.sty` | ported |  |
| `revtex4` | `.sty` | ported |  |
| `revtex4_support` | `.sty` | ported |  |
| `robustindex` | `.sty` | contributed, raw + overrides |  |
| `rotate` | `.sty` | ported |  |
| `rotating` | `.sty` | ported |  |
| `rotfloat` | `.sty` | contributed | Stub the public API as no-ops. |
| `rsfs` | `.sty` | ported |  |
| `rsphrase` | `.sty` | contributed, raw + overrides |  |
| `savetrees` | `.sty` | contributed |  |
| `scalefnt` | `.sty` | ported |  |
| `scalerel` | `.sty` | beyond Perl |  |
| `schooldocs` | `.sty` | contributed, raw + overrides |  |
| `scicite` | `.sty` | contributed |  |
| `scrbase` | `.sty` | contributed, raw + overrides |  |
| `scrkbase` | `.sty` | contributed, raw + overrides |  |
| `scrlayer` | `.sty` | contributed, raw + overrides |  |
| `scrlayer-scrpage` | `.sty` | contributed, raw + overrides |  |
| `scrlfile` | `.sty` | contributed, raw + overrides |  |
| `scrlogo` | `.sty` | contributed, raw + overrides |  |
| `scrpage` | `.sty` | contributed |  |
| `scrpage2` | `.sty` | contributed |  |
| `sectsty` | `.sty` | ported |  |
| `semantic` | `.sty` | contributed, raw + overrides | Stub for semantic.sty (proof tree / inference rules + ligatures). |
| `setspace` | `.sty` | ported |  |
| `setstack` | `.sty` | contributed |  |
| `shapepar` | `.sty` | beyond Perl |  |
| `shellesc` | `.sty` | beyond Perl, raw + overrides |  |
| `showexpl` | `.sty` | beyond Perl |  |
| `showkeys` | `.sty` | ported |  |
| `sidecap` | `.sty` | ported |  |
| `silence` | `.sty` | contributed |  |
| `SIunits` | `.sty` | ported |  |
| `siunitx` | `.sty` | ported |  |
| `slashbox` | `.sty` | ported |  |
| `slashed` | `.sty` | ported |  |
| `soul` | `.sty` | ported |  |
| `sourcecodepro` | `.sty` | contributed, raw + overrides |  |
| `spconf` | `.sty` | contributed |  |
| `spectralsequences` | `.sty` | beyond Perl |  |
| `spotcolor` | `.sty` | contributed, raw + overrides |  |
| `sprocl` | `.sty` | contributed |  |
| `srcltx` | `.sty` | ported |  |
| `srdp-tables` | `.sty` | contributed |  |
| `standalone` | `.sty` | ported |  |
| `stfloats` | `.sty` | ported |  |
| `stix` | `.sty` | contributed |  |
| `stix2` | `.sty` | contributed |  |
| `stmaryrd` | `.sty` | ported |  |
| `stwol` | `.sty` | contributed |  |
| `subcaption` | `.sty` | ported |  |
| `subeqn` | `.sty` | ported |  |
| `subeqnarray` | `.sty` | ported, raw + overrides |  |
| `subfig` | `.sty` | ported |  |
| `subfigure` | `.sty` | ported |  |
| `subfiles` | `.sty` | ported |  |
| `subfloat` | `.sty` | ported |  |
| `substr` | `.sty` | beyond Perl, raw + overrides |  |
| `supertabular` | `.sty` | ported |  |
| `sv_support` | `.sty` | ported |  |
| `svg` | `.sty` | ported |  |
| `svg-extract` | `.sty` | contributed |  |
| `svn-multi` | `.sty` | contributed, raw + overrides |  |
| `svninfo` | `.sty` | contributed |  |
| `t1enc` | `.sty` | ported, raw + overrides |  |
| `tablefootnote` | `.sty` | ported |  |
| `tabls` | `.sty` | contributed | Stub: pre-define `\@unrecurse` as `\relax` so the cleanup is a no-op. |
| `tabto` | `.sty` | beyond Perl |  |
| `tabu` | `.sty` | contributed |  |
| `tabularray` | `.sty` | contributed |  |
| `tabularx` | `.sty` | ported |  |
| `tabulary` | `.sty` | ported |  |
| `tagpdf` | `.sty` | contributed |  |
| `tagpdf-base` | `.sty` | beyond Perl, raw + overrides |  |
| `tcolorbox` | `.sty` | ported, raw + overrides |  |
| `tex-label` | `.sty` | contributed, raw + overrides |  |
| `textcase` | `.sty` | ported |  |
| `textcomp` | `.sty` | ported |  |
| `textgreek` | `.sty` | beyond Perl |  |
| `textpos` | `.sty` | contributed, raw + overrides |  |
| `texvc` | `.sty` | ported |  |
| `tgpagella` | `.sty` | contributed, raw + overrides |  |
| `theorem` | `.sty` | ported |  |
| `theoremref` | `.sty` | contributed, raw + overrides |  |
| `thm-restate` | `.sty` | ported |  |
| `thmtools` | `.sty` | ported |  |
| `threeparttable` | `.sty` | ported |  |
| `thumbs` | `.sty` | beyond Perl, raw + overrides |  |
| `tikz` | `.sty` | ported, raw + overrides |  |
| `tikz-3dplot` | `.sty` | ported, raw + overrides |  |
| `tikz-cd` | `.sty` | ported, raw + overrides |  |
| `tikzbricks` | `.sty` | ported, raw + overrides |  |
| `times` | `.sty` | ported |  |
| `tipa` | `.sty` | contributed, raw + overrides |  |
| `titleps` | `.sty` | contributed |  |
| `titleref` | `.sty` | contributed |  |
| `titlesec` | `.sty` | ported |  |
| `titling` | `.sty` | ported |  |
| `tkz-euclide` | `.sty` | contributed, raw + overrides |  |
| `tocbasic` | `.sty` | contributed, raw + overrides |  |
| `tocbibind` | `.sty` | ported |  |
| `tocloft` | `.sty` | contributed, raw + overrides |  |
| `todonotes` | `.sty` | ported |  |
| `tone` | `.sty` | beyond Perl, raw + overrides |  |
| `trace` | `.sty` | beyond Perl | Stub the package as a no-op: define the user-visible knobs (`\traceon`, `\traceoff`, `\tracingall`, `\conditionally@traceoff`, `\conditionally@traceon`) as empty macros so user-defined wrappers that … |
| `tracefnt` | `.sty` | ported |  |
| `tracklang` | `.sty` | beyond Perl, raw + overrides |  |
| `translations` | `.sty` | beyond Perl, raw + overrides | Stub for translations.sty — i18n helper used by acro, fnpct, etc. |
| `translator` | `.sty` | beyond Perl, raw + overrides |  |
| `transparent` | `.sty` | ported |  |
| `turing` | `.sty` | ported |  |
| `twoopt` | `.sty` | ported |  |
| `txfonts` | `.sty` | ported |  |
| `type1cm` | `.sty` | ported |  |
| `typearea` | `.sty` | contributed, raw + overrides |  |
| `ucs` | `.sty` | contributed |  |
| `ulem` | `.sty` | ported |  |
| `underscore` | `.sty` | ported |  |
| `underscore-ltx` | `.sty` | contributed |  |
| `undertilde` | `.sty` | ported |  |
| `unicode-math` | `.sty` | contributed |  |
| `units` | `.sty` | ported |  |
| `updatemarks` | `.sty` | contributed |  |
| `upgreek` | `.sty` | ported |  |
| `upquote` | `.sty` | ported, raw + overrides |  |
| `upref` | `.sty` | ported |  |
| `url` | `.sty` | ported |  |
| `utopia` | `.sty` | ported |  |
| `varioref` | `.sty` | ported |  |
| `varwidth` | `.sty` | ported |  |
| `verbatim` | `.sty` | ported |  |
| `versonotes` | `.sty` | contributed, raw + overrides |  |
| `vmargin` | `.sty` | ported |  |
| `vntex` | `.sty` | beyond Perl |  |
| `wacv` | `.sty` | contributed, raw + overrides | Stub for cvpr.sty / iccv.sty / iccvw.sty (computer vision conference style). |
| `wasysym` | `.sty` | ported |  |
| `widetext` | `.sty` | contributed | Stub for widetext.sty (revtex-style wide-text environment). |
| `wiki` | `.sty` | ported, raw + overrides |  |
| `wrapfig` | `.sty` | ported |  |
| `wrapstuff` | `.sty` | beyond Perl |  |
| `ws-rv-har` | `.sty` | contributed |  |
| `xargs` | `.sty` | ported |  |
| `xcolor` | `.sty` | ported |  |
| `xeCJK` | `.sty` | contributed |  |
| `xfor` | `.sty` | beyond Perl, raw + overrides |  |
| `xfrac` | `.sty` | ported, raw + overrides |  |
| `xkeymask` | `.sty` | contributed |  |
| `xkeyval` | `.sty` | ported |  |
| `xkvview` | `.sty` | ported |  |
| `xltabular` | `.sty` | contributed |  |
| `xparse` | `.sty` | ported, raw + overrides |  |
| `xpatch` | `.sty` | beyond Perl |  |
| `xr` | `.sty` | contributed |  |
| `xr-hyper` | `.sty` | contributed | *alias of* `xr.sty` |
| `xspace` | `.sty` | ported |  |
| `xunicode` | `.sty` | ported |  |
| `xurl` | `.sty` | ported, raw + overrides |  |
| `xwatermark` | `.sty` | contributed | Stub for xwatermark.sty — page-watermark package (visual-only). |
| `xy` | `.sty` | ported, raw + overrides |  |
| `xypic` | `.sty` | ported |  |
| `yfonts` | `.sty` | ported |  |

## Other binding files (engine pools, encodings, language and font-map files, TeX code files) (108)

| Name | Kind | Support | Note |
|---|---|---|---|
| `aipcheck` | `.tex` | ported |  |
| `american` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `amsa` | `.fontmap` | ported |  |
| `amsb` | `.fontmap` | ported |  |
| `AmSTeX` | `.pool` | ported |  |
| `amstex` | `.tex` | ported |  |
| `applemac` | `.def` | ported, raw + overrides |  |
| `arabic` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `australian` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `babel` | `.def` | ported, raw + overrides |  |
| `BibTeX` | `.pool` | ported |  |
| `brazil` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `brazilian` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `british` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `canadian` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `cp852` | `.def` | ported, raw + overrides |  |
| `croatian` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `csquotes` | `.def` | ported, raw + overrides |  |
| `czech` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `diagrams` | `.tex` | contributed |  |
| `ding` | `.fontmap` | ported |  |
| `dutch` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `english` | `.ldf` | beyond Perl, raw + overrides | *alias of* `english.sty` |
| `epsf` | `.tex` | ported |  |
| `eTeX` | `.pool` | ported |  |
| `expl3` | `.ltx` | ported, raw + overrides |  |
| `expl3` | `.lua` | ported |  |
| `farsi` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `francais` | `.ldf` | beyond Perl | *alias of* `french.ldf` |
| `french` | `.ldf` | ported |  |
| `frenchb` | `.ldf` | ported | *alias of* `french.ldf` |
| `german` | `.ldf` | beyond Perl | *alias of* `german.sty` |
| `germanb` | `.ldf` | beyond Perl | *alias of* `german.sty` |
| `glyphtounicode` | `.tex` | contributed |  |
| `harvmac` | `.tex` | contributed, raw + overrides |  |
| `hindi` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `icelandic` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `ifblk` | `.fontmap` | ported |  |
| `ifclk` | `.fontmap` | ported |  |
| `ifgeo` | `.fontmap` | ported |  |
| `ifsym` | `.fontmap` | ported |  |
| `ifwea` | `.fontmap` | ported |  |
| `italian` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `lae` | `.fontmap` | beyond Perl |  |
| `lanlmac` | `.tex` | contributed |  |
| `LaTeX` | `.pool` | ported |  |
| `latin` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `latin10` | `.def` | ported, raw + overrides |  |
| `lcircle` | `.fontmap` | beyond Perl |  |
| `lfe` | `.fontmap` | beyond Perl |  |
| `lgr` | `.fontmap` | ported |  |
| `line` | `.fontmap` | beyond Perl |  |
| `lmc` | `.fontmap` | beyond Perl |  |
| `ltluatex` | `.tex` | contributed |  |
| `ly1` | `.fontmap` | ported |  |
| `mssymb` | `.tex` | contributed |  |
| `newzealand` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `ngerman` | `.ldf` | beyond Perl |  |
| `ngermanb` | `.ldf` | beyond Perl | *alias of* `ngerman.ldf` |
| `nil` | `.ldf` | ported, raw + overrides |  |
| `ot2` | `.fontmap` | beyond Perl |  |
| `ot4` | `.fontmap` | ported |  |
| `pdfTeX` | `.pool` | ported |  |
| `pgfcircutils` | `.tex` | ported, raw + overrides |  |
| `pgfkeys` | `.code.tex` | ported, raw + overrides |  |
| `pgfmath` | `.code.tex` | ported, raw + overrides |  |
| `pgfmathcalc` | `.code.tex` | ported, raw + overrides |  |
| `pgfsys-latexml` | `.def` | ported |  |
| `pgfutil-common` | `.tex` | ported, raw + overrides |  |
| `phyzzx` | `.plus` | contributed |  |
| `phyzzx` | `.tex` | contributed |  |
| `polish` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `portuges` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `portuguese` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `psfig` | `.tex` | ported |  |
| `pstricks` | `.tex` | ported, raw + overrides |  |
| `pzd` | `.fontmap` | ported |  |
| `romanian` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `slovene` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `spanish` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `t1` | `.fontmap` | ported |  |
| `t1enc` | `.def` | ported, raw + overrides |  |
| `t2a` | `.fontmap` | ported |  |
| `t2b` | `.fontmap` | ported |  |
| `t2c` | `.fontmap` | ported |  |
| `t3` | `.fontmap` | beyond Perl |  |
| `t3enc` | `.def` | beyond Perl, raw + overrides |  |
| `t4` | `.fontmap` | beyond Perl |  |
| `t5enc` | `.def` | ported, raw + overrides |  |
| `tcblistingscore` | `.code.tex` | beyond Perl, raw + overrides |  |
| `tcilatex` | `.tex` | ported |  |
| `TeX` | `.pool` | ported |  |
| `tikzlibraryexternal` | `.code.tex` | beyond Perl, raw + overrides |  |
| `tikzlibrarytrees` | `.code.tex` | beyond Perl, raw + overrides |  |
| `ts1` | `.fontmap` | ported |  |
| `tu` | `.fontmap` | beyond Perl |  |
| `turkish` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `txtbabel` | `.def` | ported, raw + overrides |  |
| `u` | `.fontmap` | beyond Perl |  |
| `UKenglish` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `USenglish` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `utf8` | `.def` | ported |  |
| `utf8x` | `.def` | ported | *alias of* `utf8.def` |
| `vietnamese` | `.ldf` | beyond Perl, raw + overrides | babel-language stubs for `.ldf` files that aren't installed in minimal TeXLive environments. |
| `x2` | `.fontmap` | beyond Perl |  |
| `xy` | `.tex` | ported, raw + overrides | *alias of* `xy.sty` |
| `xylatexml` | `.tex` | ported |  |
| `xypic` | `.tex` | ported |  |
