# Mini Plan: Round 10

## Status: 226 pass, 0 fail, 93 ignored

## Three most connected work packets

### Packet 1: Implement \lxDeclare fast-path (simple token patterns)
- **Impact**: Unblocks ~10 parse tests that use `\lxDeclare[role=X]{$token$}`
- **Approach**: Parse the pattern, if it's a single math token, create a simple
  document rewrite rule that sets attributes on matching XMTok elements
- **Skip**: Complex wildcards, scope, domToXPath
- **Files**: latexml_sty.rs (constructor), rewrite.rs (attributes operator)

### Packet 2: Apply rewrite rules during finalization
- **Impact**: Makes Packet 1 actually work
- **Files**: core_interface.rs (call rewrite rules after finalize)

### Packet 3: Sync test .tex files from Perl for unblocked tests
- **Impact**: Several tests have modified .tex files; need to sync from Perl
- **Files**: Various test .tex files

### Expected outcome
- +5-10 passing tests from parse suite
