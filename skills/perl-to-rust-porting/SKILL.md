---
name: translate-perl-code-to-rust
description: Translate Perl code to Rust, choose best practice idioms and evaluate differences. Use when a task has an origin Perl file (such as .pm, .ltxml or .pl extension) and aims to rewrite it in Rust (.rs) or to sync an existing translation with an updated Perl.
---

# Translate Perl code to Rust

### Step 1:

Read the original Perl code and any existing Rust that from previous translation attempts. Write a short summary of what is covered.

### Step 2:

Identify one to three test files in the Rust space which will exercise this translation. If no test file exists, find a Perl test which exercises the code and port that test to a Rust test before translating any of the code.

### Step 3:

Create a plan for the main functions and structures that need translation. Enumerate details that need idiomatic Rust treatment,
  or have needed it in other places of the existing Rust code base.

### Step 4:

Write down the translation, one function at a time. After a function is written, check if it has obvious idiomatic improvements for more natural Rust.

### Step 5:

After the translation is written, run the identified tests. If they fail, gradually refine a function at a time until the tests pass.

### Step 6:

Once a translation is completed and tests pass, ask user for confirmation. Upon confirmation add the files to git and write a summary commit message. Commit the files.

### Step 7:

This translation is complete. Repeat from step 1 if asked to translate another target.