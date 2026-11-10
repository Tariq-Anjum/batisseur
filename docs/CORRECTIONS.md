# Verification history

Test-first logs record expected failures before each implementation slice.
They are not repair cycles. Compiler paths are replaced with [checkout].

## Correction 1 — LOW

Observation: slice-three.log shows the named linux-kiosk-touch benchmark
expecting a unique native route while its input is identical to the deliberate
native/browser collision case.
Root cause: fixture omitted its distinguishing native device-I/O constraint.
Research: compared both local fixture records and pack eligibility/conflict data;
no external behavior or dependency uncertainty is involved.
Change: declare native_io=true for the named native-kiosk case; retain the
separate ambiguous touch-panel case and its abstention oracle unchanged.
Expected: native case selects the native pack; collision still abstains.
Bounded check: full cargo test plus cargo run -- bench, below.
Rollback: restore this one fixture field to its previous absent state.
Actual result: recorded in correction-one.log.
Next decision: proceed only if all frozen oracles and CLI tests pass.

## Correction 2 — MEDIUM

OBSERVATION: edge-cases-red.log proves a direct-touch/pointer contradiction
and an unmodeled one-megabyte memory limit both route successfully.
ROOT_CAUSE: independent pack membership checks permit a cross-product of
profiles/inputs; resource limits have no evidence-backed pack comparisons.
RESEARCH: fresh targeted inspection of rejection(), the intent schema, and
web-pwa's two allowed-input arrays confirmed both gaps. These are local
logic errors; no uncertain external/runtime behavior. Network is prohibited
by this task, so investigation uses the exact local source and regressions.
RISK_CLASS: MEDIUM, reversible input validation in an advisory local CLI.
PROPOSED_CHANGE: data-driven profile/input coherence and conservative
abstention for unmodeled hard constraints.
WHY_SEMANTICS_UNCHANGED: enforces explicit constraints and typed interactions
already required by the architecture; type selection remains unchanged.
WHY_SECURITY_BOUNDARY_UNCHANGED: no execution, permissions, or access added.
WHY_ARCHITECTURE_UNCHANGED: checks stay in the deterministic typed cascade.
ROLLBACK_PATH: remove the new profile data and constraint preflight block;
retain failing regressions as evidence if rollback is needed.
EXPECTED_RESULT: both edge cases abstain; all 16 frozen cases stay unchanged.
ACTUAL_RESULT: correction-two.log records the full suite and benchmark.
NEXT_DECISION: final proof only after these checks pass.

## Correction 3 — LOW (evidence harness only)

OBSERVATION: final product checks pass, but recursive grep reports
`input file is also the output` and exits 2 when its stdout is redirected to
privacy-scan.log inside the scanned tree. No content match was reported.
ROOT_CAUSE: the evidence command opened its output file before recursively
scanning that same file; this is an evidence collection error.
RESEARCH: inspected the exact shell redirection and GNU grep 3.12-modified
help/version. Recursive traversal includes the log, and help distinguishes
exit 1 (no selection) from exit 2 (error). Cross-checked the successful
product log and the zero-byte output file. No external behavior is uncertain;
network research is prohibited by the brief.
CHANGE: capture the unchanged required grep command in subprocess pipes;
write the zero-match evidence file only after grep has finished.
EXPECTED_RESULT: exit 1, empty stdout and stderr, then an empty evidence log.
ACTUAL_RESULT: privacy-scan-status.json records the result.
NEXT_DECISION: commit only on a clean scan; no further corrections authorized.
