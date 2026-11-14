# Installing latexml-oxide

[← README](../README.md) · [Usage](USAGE.md) · [Building from source](BUILDING.md) · [docs index](README.md)

## Prebuilt binaries

The [Releases page](https://github.com/dginev/latexml-oxide/releases) has prebuilt binaries for Linux, macOS and Windows,
each a single, fully self-contained executable.

You still need a working TeX installation, **TeX Live or MiKTeX**, at runtime, to resolve TeX packages, classes and fonts.
A few external tools are called as subprocesses, mainly to convert graphics files.
The binaries embed TeX format dumps for the TeX Live releases they were built with; with any
other TeX tree (a Linux distribution's TeX Live, or one updated since), the first conversion
builds the formats once (about 10 s) into a per-user cache.

The steps below use a version variable — set it to the release you want:
```
$ VERSION=0.7.6
```

## Ubuntu / Debian

The `.deb` declares its runtime apt dependencies (libraries + TeX Live + the graphics tools), so it is the easiest way in.
Built on Ubuntu 22.04 (glibc 2.35), so it runs on Ubuntu 22.04+ and Debian 12+.

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide_${VERSION}-1_amd64.deb
$ sudo apt install ./latexml-oxide_${VERSION}-1_amd64.deb
```

If you prefer the portable tarball, the binary is statically linked against libxml2/libxslt/libkpathsea, so you only need the external tools + TeX Live:

```
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-x86_64-unknown-linux-gnu.tar.gz
$ tar xzf latexml-oxide-$VERSION-x86_64-unknown-linux-gnu.tar.gz
$ sudo cp latexml-oxide-$VERSION-x86_64-unknown-linux-gnu/latexml_oxide /usr/local/bin/
$ sudo apt install imagemagick mupdf-tools poppler-utils ghostscript dvipng dvisvgm \
                   texlive-latex-base texlive-latex-extra texlive-science
```

On **64-bit ARM** (AWS Graviton, Ampere, Raspberry Pi OS 64-bit) swap `amd64` → `arm64` in the `.deb` name
and `x86_64-unknown-linux-gnu` → `aarch64-unknown-linux-gnu` in the tarball name; everything else is identical.

## macOS (Apple Silicon / arm64 + Intel / x86_64)

**Recommended — Homebrew.** Installs the right binary for your Mac, and pulls the graphics tools it needs:

```
$ brew install dginev/tap/latexml-oxide
```

You still need a TeX distribution (`brew info latexml-oxide` repeats this):

```
$ brew install texlive          # Homebrew's full TeX Live (~5 GB)
# …or MacTeX / BasicTeX — https://tug.org/mactex/  (put /Library/TeX/texbin on PATH)
```

**Alternative — download the tarball directly.** Pick the one matching your Mac (`uname -m` prints `arm64` or `x86_64`):

```
# Apple Silicon
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-aarch64-apple-darwin.tar.gz
$ tar xzf latexml-oxide-$VERSION-aarch64-apple-darwin.tar.gz
$ sudo cp latexml-oxide-$VERSION-aarch64-apple-darwin/latexml_oxide /usr/local/bin/

# Intel (built with a macOS 10.13 deployment target, so it runs on older Intel Macs)
$ curl -LO https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-x86_64-apple-darwin.tar.gz
$ tar xzf latexml-oxide-$VERSION-x86_64-apple-darwin.tar.gz
$ sudo cp latexml-oxide-$VERSION-x86_64-apple-darwin/latexml_oxide /usr/local/bin/

$ brew install imagemagick mupdf-tools poppler ghostscript dvisvgm
$ brew install texlive          # or install MacTeX / BasicTeX (provides dvipng)
```

The prebuilt binary links `libkpathsea` statically, so it finds your TeX files in-process on either
Homebrew's `texlive` or MacTeX/BasicTeX — just make sure the TeX binaries are on `PATH`
(`/Library/TeX/texbin` for MacTeX).

> **Gatekeeper / "unidentified developer" (tarball, browser downloads).**
> The `curl` + `tar xzf` install above is warning-free: terminal downloads and command-line `tar`
> don't set macOS's `com.apple.quarantine` flag. But if you download the tarball in a **browser**
> and unpack it by double-clicking in Finder, macOS may refuse to run the binary as "from an
> unidentified developer" — the binaries are ad-hoc signed, not Apple-notarized. Clear it once with
> `xattr -d com.apple.quarantine /usr/local/bin/latexml_oxide`, or right-click the binary in
> Finder → **Open**. The **Homebrew install above avoids this entirely**, since `brew` strips the flag.

## Windows (x86_64)

*(From `0.7.4`.)* A single self-contained `latexml_oxide.exe` (fully static: no VC++ redistributable needed),
shipped in a `.zip`. In PowerShell run:

```
> $VERSION = "0.7.6"
> curl.exe -LO "https://github.com/dginev/latexml-oxide/releases/download/$VERSION/latexml-oxide-$VERSION-x86_64-pc-windows-msvc.zip"
> Expand-Archive "latexml-oxide-$VERSION-x86_64-pc-windows-msvc.zip" -DestinationPath .
> .\latexml-oxide-$VERSION-x86_64-pc-windows-msvc\latexml_oxide.exe --version
```

Put `latexml_oxide.exe` somewhere on your `PATH`.
A TeX distribution (**TeX Live for Windows or MiKTeX**) must be on `PATH` for host TeX resolution;
the binary auto-selects the fast in-process backend on TeX Live, and falls back to subprocess `kpsewhich` on MiKTeX.
ImageMagick (`magick`), Ghostscript (`gswin64c` / MiKTeX `mgs`) and MuPDF (`mutool`)
are optional, for figure conversion.

## Docker

A batteries-included image (`latexml_oxide` + a reproducible TeX Live + the graphics tools)
is published to the GitHub Container Registry for both amd64 and arm64.
No local TeX Live needed — bind-mount your document tree and convert:

```
$ docker run --rm -v "$PWD:/work" ghcr.io/dginev/latexml-oxide:$VERSION paper.tex
```

`:latest` tracks the most recent release. The container builds its own binary against the image's TeX Live,
so the embedded kernel dumps match the bundled texmf tree exactly.

## System dependencies

The binary is self-contained (libxml2/libxslt/kpathsea are linked in), but at runtime it **shells out**
to external tools for graphics conversion, and reads TeX assets from your TeX Live tree.
None are bundled — install the ones your documents need.
When a required tool is missing, the conversion log names it **and the package to install**.
The `.deb` declares all of these, so `apt install ./latexml-oxide_*.deb` pulls them automatically.

| Tool (`command`) | apt package | Homebrew | Used for |
|---|---|---|---|
| `convert` | `imagemagick` | `imagemagick` | raster image conversion |
| `mutool` | `mupdf-tools` | `mupdf-tools` | primary PDF graphics (fast) |
| `pdftocairo` | `poppler-utils` | `poppler` | vector-SVG from PDF |
| `gs`, `ps2pdf` | `ghostscript` | `ghostscript` | PDF/PostScript conversion |
| `dvipng` | `dvipng` | TeX Live | raster LaTeX-image output |
| `dvisvgm` | `dvisvgm` | TeX Live | vector-SVG LaTeX-image output |
| `kpsewhich`, `latex`, `pdflatex`, `tftopl` | `texlive-latex-base` (+`-extra`, `-science`) | `texlive` / MacTeX | TeX package/class/font resolution |

## From crates.io (`cargo install`)

```
$ cargo +nightly install latexml
```

This builds the `latexml_oxide` CLI from source, so it needs the prerequisites under
[Build from source](BUILDING.md#build-from-source), and a TeX distribution at runtime.
The crate ships no TeX format dumps: the first conversion builds them for your TeX Live
(about 10 s) into a per-user cache directory (`LATEXML_FORMAT_CACHE` to choose it), and
later runs reuse them until TeX Live changes.

**As a library:**

```toml
[dependencies]
latexml = "0.7"
```

```rust
let xml  = latexml::api::convert_to_xml(tex)?;   // TeX → LaTeXML XML
let html = latexml::api::convert_to_html(tex)?;  // TeX → HTML5 + Presentation MathML
```

For finer control (preloads, search paths, `--whatsin`, split, encoding, …), drive
`latexml::converter::Converter` and `latexml::post` directly.
