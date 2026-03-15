---
name: understanding-a-failure-and-debugging-conversion
description: Debug a conversion difference between latexml-oxide (Rust) and latexml (Perl) using a minimal TeX example. Use when investigating why a specific TeX construct produces different XML output in Rust vs Perl.
---

# Understanding a failure and debugging Conversion for Minimal TeX Example

### Step 1: Extract Minimal Example

Extract the problematic TeX snippet into a standalone minimal `.tex` file. The file should:
- Use `\documentclass{article}` (or the relevant class)
- Include only the packages needed for the construct under test
- Contain the smallest possible input that reproduces the difference
- Save it as e.g. `/tmp/debug_minimal.tex`

### Step 2: Run Both Converters

Run latexml-oxide (Rust) and latexml (Perl) on the same input:

```bash
# Rust
timeout 30 cargo run --release --bin latexml_oxide /tmp/debug_minimal.tex > /tmp/debug_rust.xml 2>/tmp/debug_rust.log

# Perl (--includestyles ensures raw .sty files are processed, matching Rust behavior)
latexml --timeout=30 --includestyles /tmp/debug_minimal.tex > /tmp/debug_perl.xml 2>/tmp/debug_perl.log
```

### Step 3: Compare the Two XMLs

Diff the XML outputs to identify the exact divergence:

```bash
diff /tmp/debug_perl.xml /tmp/debug_rust.xml
```

Analyze the differences:
- Missing/extra elements or attributes
- Wrong attribute values
- Structural nesting differences
- Missing text content

### Step 4: Deep Debugging (if needed)

If the XML diff alone doesn't reveal the root cause, modify the minimal example to add TeX tracing:

```tex
\tracingall        % trace everything (very verbose)
\tracingmacros=1   % trace macro expansion
\tracingcommands=1 % trace command execution
\message{DEBUG: about to expand problematic macro}
```

Re-run both converters and compare the trace output in the `.log` files to find where expansion/digestion diverges.

Iterate: narrow the minimal example further based on what the traces reveal, until the root cause is isolated to a single macro definition or primitive behavior difference.
