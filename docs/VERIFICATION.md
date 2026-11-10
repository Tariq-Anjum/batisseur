# Verification criteria and evidence

The final check record is `verification.log`; detailed historical failures
and two bounded corrections are preserved separately in CORRECTIONS.md.
Compiler checkout paths in recorded logs are sanitized to [checkout].

| Criterion | Check | Expected / fail condition | Evidence |
| --- | --- | --- | --- |
| Offline reproducible dependency resolution | locked offline build | success without fetching; any missing dependency fails | verification.log |
| Formatting | cargo fmt --check | no diff; formatting diff fails | verification.log |
| Lint | cargo clippy --all-targets -- -D warnings | no warnings/errors | verification.log |
| Routing unit and integration checks | cargo test | all tests pass; any failing test fails | verification.log |
| Frozen routing accuracy | cargo run -q -- bench | 16/16, 100%; any mismatch fails | verification.log |
| Abstention and interaction collisions | BuildBench and routing regressions | 4/4 frozen abstentions, invalid pairs abstain | verification.log |
| Minimal activation and selection | BuildBench frozen capability/verifier oracles | zero unexpected activations, 16 verifier matches | verification.log |
| Permission correctness | CLI/routing tests and BuildBench | permission gap stays separate, 16 gate matches | verification.log |
| Determinism | CLI integration subprocesses and three saved benchmark runs | byte-identical output, 16 deterministic cases | determinism.json |
| Shared taxonomy pin | source byte comparison, provenance SHA-256, schema projection inspection | exact copy, declared version and IDs match | contract-check.json |
| Public repository content | required private-content scan | zero matches | privacy-scan.log |
| Bounded documents | line counts | SPEC <=400, skill <=80 | contract-check.json |
| Final version control | clean main with required author | clean tree, 1-3 commits | final report outside repository |

The schemas are checked structurally by the crate's own limited validator;
no independent general-purpose JSON Schema package is installed or fetched.
Official platform links are supplied as conservative references, not live
compatibility research. No real device verifiers, executor, resumed run,
remote CI, or publication has been exercised. Context bytes are partial;
latency and token metrics are intentionally unmeasured rather than invented.
