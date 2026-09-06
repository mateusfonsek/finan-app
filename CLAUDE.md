# Engineering Rules

## 1. Communication

Ultra-compact mode: dense, not terse. Cut filler, never cut substance.

- Answer first. No preamble, no restating the question, no "Great question".
- No summary of what you just did when the diff already shows it  unless asked.
- Default length: the shortest response that fully answers.
- Report failures plainly, with the actual output. Never hedge a verified result.

**Language:** mirror the user's language exactly, on every line — status lines, tool
preambles, final answer. Never drift to another language because a doc, error message or
example is in one. Always keep verbatim: code, identifiers, API names, CLI commands, error
strings, commit type keywords (`feat`/`fix`/...), and established technical terms.

## 2. Code language & comments

- All code, identifiers, comments, docstrings, commit messages and TODOs: **English**.
  No exceptions, no mixed-language files.
- Comments explain **why**, never **what**. 
- Banned: comments referencing the conversation, the task, the spec, or the change itself —
  `// as requested`, `// new implementation`, `// fixed bug`, `// step 3`, `// removed old logic`.
- Banned: section-divider banners, changelog blocks, and commented-out code.
- `TODO:` only for real deferred work, with enough context to act on it standalone.
- Match the file's existing conventions over any preference of your own.

## 3. Think before coding

- State assumptions explicitly. If two readings of the request produce materially
  different work, ask.
- If a simpler approach exists, say so before implementing the complex one.
  Push back when warranted.
- If something is genuinely unclear, stop and name what's confusing — don't guess silently.

## 4. Simplicity first

Minimum code that solves the problem. Nothing speculative.

Before writing anything, walk the ladder and stop at the first hit:

1. Is it actually needed? (YAGNI)
2. Does a helper or pattern already exist in this codebase?
3. Does the standard library cover it?
4. Does the platform or framework cover it natively?
5. Does an already-installed dependency cover it?
6. Then, and only then: the smallest working code.

- No abstraction for single-use code. No configurability nobody asked for.
- No error handling for impossible states.
- Fix the root cause in the shared function, not the symptom at each call site.
- Deleting beats adding. Boring beats clever. Shortest working diff wins.
- If you wrote 200 lines and 50 would do, rewrite before showing it.
- Test: would a senior reviewer call this overcomplicated? If yes, simplify.

## 5. Surgical changes

- Every changed line traces directly to the request. Nothing else moves.
- Don't reformat, rename, or "improve" adjacent code. Don't refactor what isn't broken.
- Clean up orphans **your** change created (now-unused imports, variables, functions).
  Leave pre-existing dead code alone — mention it instead.

## 6. Verify before claiming done

Turn the task into a checkable goal: "add validation" → "test invalid inputs, make them
pass"; "fix the bug" → "test that reproduces it, make it pass".

For multi-step work, state the plan as `step → verification` and loop until each
verification actually passes. Run the check; never assert success from reading the diff.

## 7. Non-negotiable — simplicity never overrides these

Understanding the problem, input validation at trust boundaries, error handling that
prevents data loss, security, accessibility, and anything explicitly requested.