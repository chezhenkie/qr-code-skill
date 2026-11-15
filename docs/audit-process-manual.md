# Audit process manual

Applies to any existing codebase or subsystem that needs an audit before further work: rewrite, hardening, publication, or a migration. The process is code-first, then documents, then triage, then execution. It runs in one session per scope.

## Scope and passes

Define the scope as a component boundary, meaning the files that ship and run together: a plugin and its patch script, a service and its container, a module and its tests. Record the boundary and what is explicitly out of scope before reading code.

When the scope is too large for one sitting, split it into passes along component boundaries, not along dimensions:

- Pass A: component one, all four lenses.
- Pass B: component two, all four lenses.

Run four lenses inside every pass:

- Security: input trust, filesystem and network reach, credentials, injection, resource exhaustion.
- Compliance: rules the product promised, licenses, privacy, retention, platform constraints.
- Performance: cost per event, allocations, buffers, timeouts, concurrency limits.
- Quality: correctness of the happy path, error paths, dead code, naming, test strength, documented vs actual behavior.

Reason for component splitting: a dimension split reads every file twice and separates findings that interact. Attachment caps are performance and security at once. Link filtering is compliance and security at once. A buffer is performance and correctness at once. Dimension splits push those to the seam between two documents, where they get lost. Compliance is doc-vs-code by nature, so it is a lens in step 2, not a pass of its own.

## Steps

1. Code-first pass. Read the code and scripts only. No handoff, no design docs, no spec, no changelog. This is deliberate: the point is to find what the code actually does before being told what it is supposed to do. Write the audit document with findings per component. Every finding names file and line, severity, and the reason it matters.

2. Document cross-check. Read the handoff, the docs, the specs, and the changelog. Compare every claim against the code. Append contradicting findings to the same audit document. A documented behavior the code does not implement is a finding, not a docs edit.

3. Critical triage. Re-read the audit document against itself. Drop duplicates, merge related items, re-rank by severity and cost, and reject findings that do not survive the code. Write a second document containing only accepted items: the change, the files touched, the test that proves it, and the risk of doing it. The audit document stays as the record; the plan document is what gets executed.

4. Execute the plan. One commit per item or per coherent group. After any change that touches a running system, rerun the checks that would fail if the change regressed: tests, verifier, link check, health check.

## Artifacts

- `audit-<scope>-code.md` - steps 1 and 2, findings only, no fixes
- `audit-<scope>-plan.md` - step 3, triaged and accepted items only
- commits - step 4, each naming the item it closes

Keep both documents in the repository's audit folder, not on a desktop or in chat.

## Evidence rules

- Every finding names file and line. No finding without a reproduction or a quoted line.
- No fix is designed in step 1. Step 1 observes.
- A finding rejected in triage gets one line saying why, so it is not re-raised next session.
- No change lands in step 4 without a test or a live check that fails when the change is reverted.
- Numbers come from a command, not from memory: test counts, sizes, timings, versions.

## Session boundary

If the audit cannot finish in one session, stop at the end of a step, not in the middle of one. The handoff then carries: the scope, which steps are closed, the path of both documents, and the next step. A half-written audit document is worse than none.
