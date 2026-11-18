# Code signing policy

**Status:** the Windows binary is **not code-signed**. SignPath Foundation declined our application to its free code-signing program for open-source projects (2026-11-18, GitHub issue 775); they suggest applying again once the project is in heavy mainstream use. Paid signing services are out of scope. Until a signed channel exists, the [installation guide](https://github.com/dginev/latexml-oxide/blob/main/docs/INSTALL.md) says how to unblock the binary, and each release's `latexml_oxide.exe` is submitted to Microsoft's false-positive analysis.

**Plan:** a Microsoft Store package (MSIX) in a 0.8.x release. Microsoft signs Store packages when it certifies them, and the package will install a `latexml_oxide` command for any terminal. The unsigned zip stays on the [Releases page](https://github.com/dginev/latexml-oxide/releases). The first signed channel will be named here.

## What will be signed

- **Signed:** `latexml_oxide.exe` inside the Store package, by Microsoft at certification.
- **How it is built:** from this repository's source, by the [`release.yml`](https://github.com/dginev/latexml-oxide/blob/main/.github/workflows/release.yml) GitHub Actions workflow on GitHub-hosted runners — the same executable as in the Releases zip.
- **Only our own binary:** the third-party libraries statically linked into it (libxml2, libxslt, libkpathsea and libmarpa; see [`THIRD-PARTY-NOTICES`](https://github.com/dginev/latexml-oxide/blob/main/THIRD-PARTY-NOTICES)) are compiled from their published sources in that same workflow.

## Team roles

| Role | Who |
|---|---|
| Authors (committers) | [the repository's maintainers](https://github.com/dginev/latexml-oxide/graphs/contributors) with write access: Deyan Ginev |
| Reviewers | every change lands through a reviewed pull request: Deyan Ginev |
| Publisher (Store submissions) | to be decided with the Store account |

All team members use multi-factor authentication for GitHub.

## Privacy policy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.

latexml-oxide reads files through the user's own TeX distribution (TeX Live or MiKTeX). It may also run image tools the user has installed: ImageMagick, Ghostscript, MuPDF or Poppler. What those programs do follows their own settings; for example, MiKTeX can be configured to download missing packages.

## System changes

- **No installer.** latexml-oxide is a single executable. Unpacking the zip installs it, and deleting the folder uninstalls it. The planned Store package installs and uninstalls through Windows' own app management.
- **No system changes.** It does not change the registry, services, environment variables or `PATH`.
- **What it writes:**
  - the converted output, where the user directs it;
  - a cache of compiled TeX formats, under `%XDG_CACHE_HOME%` or `%HOME%\.cache` when one of them is set, and otherwise under `%TEMP%`;
  - when `HOME` or `XDG_CACHE_HOME` is set, a cache of converted images in `latexml-oxide\graphics` under that folder.

  These caches are safe to delete at any time.
