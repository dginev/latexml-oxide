# Intermediate files

Besides the output you ask for (`--dest`, the images and stylesheets beside it, and the log), a conversion writes
the files below. **Caches** are kept between runs and are always safe to delete. **Scratch** files are removed when
a run ends, including on a timeout or a memory stop. A run that is killed or interrupted (Ctrl+C) can leave scratch
behind; those files are dot-named or sit in the system temp directory, and are safe to delete. None of them is ever
packed into a `.zip` or EPUB output.

`$TMPDIR` below is the system temp directory; `$XDG_CACHE_HOME` defaults to `~/.cache`. The variables that move or
turn off a cache are described in [ENVIRONMENT.md](https://github.com/dginev/latexml-oxide/blob/main/docs/ENVIRONMENT.md).

## Caches (kept between runs)

| File | Where | What it holds |
|---|---|---|
| `LaTeXML-oxide.cache` (and, while a run has it open, `-wal`/`-shm`) | the destination directory | The names given to the document's images (Perl latexmlpost's `LaTeXML.cache`, as SQLite), so a reconversion reuses them and documents converted into one folder never share a name. Not written under `--nocache`, for a `.zip` or EPUB destination, nor by `cortex_worker` or the editor server; deleting it only means names are given afresh, past the files already in the folder. |
| `graphics/<aa>/<hash>.<ext>`, its `.dims` sidecar, `.prune.lock` | `$XDG_CACHE_HOME/latexml-oxide/` (`LATEXML_GRAPHICS_CACHE_DIR` moves it) | Converted images keyed by their content, reused by later conversions. Capped at `LATEXML_GRAPHICS_CACHE_MAX_MB` (2048 MiB); `LATEXML_GRAPHICS_CACHE_OFF=1` turns it off. |
| `formats/<version>-<executable>-…/` with `.lock`, `.partial`, `.attempts.*`, `.failed.*` files | `$LATEXML_FORMAT_CACHE`, else `$XDG_CACHE_HOME/latexml-oxide/`, else `$TMPDIR` | Format dumps built for a TeX Live tree that the binary has no embedded dump for. One set per binary build; not pruned. |
| `latexml-oxide-dumps-<hash>-<uid>/<kind>.<year>.dump.txt` | `$TMPDIR` | The binary's embedded dumps, unpacked once so concurrent processes share them. Not pruned. |
| `latexml_lua_prelude_<uid>_<hash>.lua` | `$TMPDIR` | The sandbox prelude of `\directlua`, shared by concurrent processes. Not pruned. |

## Scratch (one run)

| File | Where | When |
|---|---|---|
| `.tmpXXXXXX/` | `$TMPDIR` | The source is a `.zip` or `.tar(.gz)` archive: it is unpacked here. |
| `.latexml-spill-<pid>-<seq>/segment-*.xml` | the source directory, else `$TMPDIR` when that is not writable | A very large document is converted in streaming mode. |
| `.latexml-writable-<pid>` | the source directory | Checks that the directory is writable; removed at once. |
| `latexml-post-handoff-<pid>.xml` | beside the destination, else `$TMPDIR` | The core result, handed to post-processing. |
| `.latexml-post-pages-XXXXXX/` (`page-*.xml`, and for a parallel render `LaTeXML-render.db`, `render-manifest-<n>.json`) | beside the destination, else `$TMPDIR` | Post-processing: the pages of a split document, and the shared data of parallel render workers. |
| `.<stem>.tmp.<pid>.<seq>.<ext>` | the destination directory | Every output file is written under this name, then renamed into place. |
| `.<stem>.<pid>-<seq>.mutool_svg/`, `.mutool.png`, `.pdftocairo.png`, Ghostscript and `ps2pdf` temporaries, `.<stem>.<n>.fonts.eps` | the destination directory | Converting a PDF, PostScript or EPS image. ImageMagick and Ghostscript also keep their own temporaries in `$TMPDIR`. |
| `.tmpXXXXXX/` (`mimetype`, `META-INF/`, `OPS/`, …) | `$TMPDIR` | Assembling a `.zip` or EPUB destination. |

The log, `<name>.latexml.log` (`latexml.log` for literal input), is written to the current directory unless `--log`
names another file; each run replaces it.

## Written by the document itself

`\openout`, `\write` and `filecontents` write to an in-memory file store, never to disk, and `\write18` runs nothing.
One exception: Lua code run by `\directlua` may write files with `io.open`, but only under relative paths in the
source directory (no `..`, no dot-files). These files are never removed.

## cortex_worker

| File | Where | What it holds |
|---|---|---|
| `.tmpXXXXXX/` | `$TMPDIR`, or the harness slot | One paper's source and outputs. |
| `cortex_output_<pid>_<seq>.zip` | `$TMPDIR`, or the harness slot | One paper's result, before it is sent to the dispatcher. |
| `cortex_harness_XXXX/` (`harness.lock`, `child-*.claim`, a log mirror of at most 8 MiB, `slot-NN/`) | `$TMPDIR` | The harness's state while it runs; a stale one is removed when the next harness starts. |
