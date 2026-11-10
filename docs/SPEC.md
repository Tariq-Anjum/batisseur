# batisseur v0.1.0 specification

Status: experimental. This version is a deterministic, advisory router, not
an executor. It consumes typed JSON, selects one sufficient platform capability
inside the supplied task-type cell, explains the decision, and emits a plan.
No server, database, background process, network request, or installation runs.

## Accepted shared selection contract

The accepted tt-1.1.0 handshake is binding. `contracts/tt-1.toml` is the single
shared task taxonomy, vendored unchanged from tisserand under MIT. Its source,
revision, version and SHA-256 are recorded in `contracts/provenance.json`.
The content hash identifies the snapshot even if the upstream URL moves.
An explicit intent version differing from tt-1.1.0 is an error. The compiled
contract and every compiled pack must also declare tt-1.1.0.

`task_type` is required. Accepted IDs and artifact values come from that pinned
contract. The JSON Schema enums are projections of the same snapshot; they are
not an independently extended taxonomy. No build-specific type list exists.
Unknown required values abstain; `unclear` always maps to `needs_information`.
Neither work_kind nor a platform, interaction, app name, framework, or family
can choose or override the supplied type. The CLI performs no language-model
inference. A separate caller may propose typed values, subject to validation.

Families derive only from type (`contracts/families.json`):

| Type | Family |
| --- | --- |
| clarify | Product |
| research, explore | Discovery |
| design | Architecture; Experience if artifact=interface |
| plan | Planning |
| implement, setup | Build |
| debug | Debug |
| verify | Verification, Quality |
| review | Security, Audit; finished work required |
| deliver | Delivery |
| document | Maintenance |
| handoff, answer, unclear | none |

The family-map keys must exactly match the canonical taxonomy IDs. Family
labels are descriptive, never selection inputs. `handoff` and `answer` yield
no platform capability after validating the required axes. An abstention has
no active family, capability, selected pack, verifier set, or plan.

## BuildIntent

`schemas/build-intent.schema.json` is JSON Schema 2020-12. Objects reject
unknown properties, enums reject unrecognized strings, arrays reject duplicate
items, and numeric limits must be nonnegative integers. Known categorical
fields may be null or `unknown`, meaning unresolved. Missing required fields
and required fields with null/unknown values abstain during routing.

Required top-level fields:

- `taxonomy_version`: exactly tt-1.1.0.
- `task_type`: one canonical taxonomy ID.
- `artifact`: canonical artifact category.
- `work_kind`: new_product, feature, bug, audit, reconstruction, migration,
  modernization, or prototype. This is a build axis, not a task type.
- `target`: platform, os, form_factor, deployment_env required;
  display_env optional.
- `interaction`: profile and primary_input required; secondary_inputs,
  focus_model, viewing_distance, orientation and accessibility optional.
- `application`: kind (gui/cli/tui/service/library/plugin/firmware),
  runtime (local/networked/hybrid), mode (foreground/kiosk/daemon).
- `authority`: explicit permitted_ops (read/write/publish); optional
  external_access, destructive_ops, reverse_engineering booleans.

Optional `constraints`: offline, hardware, latency_ms, memory_mb, storage_mb,
power, security, privacy, distribution, existing_stack, native_io, privileged.
Optional `evidence`: source, screenshots, design, binaries, reference_app,
api_docs, finished_work, verifiers. Evidence strings are metadata; the CLI
neither loads those locations nor treats them as executable instructions.

The router supports offline, deployment/distribution, stack, native-I/O and
privilege constraints. Packs currently lack comparable measurements for
hardware, latency, memory, storage, power, security, and privacy limits. A
supplied such constraint abstains and requests `constraints.<field>_evidence`;
it is never silently treated as satisfied. This needs a vetted pack extension,
not an invented value pasted into the intent.

`validate` checks complete structural input and unresolved required fields.
It does not certify platform support, permissions, feasibility, or safety.
`route` permits incomplete objects to return actionable abstention results;
invalid JSON, wrong types/enums, unknown properties and version mismatch are
errors. A valid abstention exits successfully; the status is in the result.

## Interaction contract

Profiles are separate from platform. `contracts/interaction-profiles.json`
checks coherent profile/primary-input pairs: precision_pointer/pointer,
keyboard_first/keyboard, direct_touch/touch, indirect_focus/dpad,
gamepad/gamepad, rotary/rotary, pen_stylus/pen, voice/voice, spatial/spatial,
mixed/mixed, none/none. Some valid profiles have no platform pack in v0.1.
Indirect focus additionally requires a known focus_model. Secondary input,
orientation, viewing distance and accessibility remain explicit context for
the selected verifier contract; no physical-device usability claim is made.
A direct-touch profile combined with a pointer primary input abstains.
Android alone and a generic touchscreen do not identify an app profile.

## Deterministic router cascade

1. **Explicit constraints and typed extraction:** validate the schema and
   taxonomy version; collect unresolved required fields; reject unsupported
   hard-limit claims and inconsistent interaction pairs. Require finished_work
   for review. Never infer a task type from work_kind.
2. **Eligibility:** compare exact typed platform, OS, form factor, deployment,
   application kind/mode/runtime, interaction profile and primary input with
   each pack's allowed values. Offline requirements must be supported.
3. **Weighted scoring:** score eligible packs only; emit every signed term,
   amount and explanation. Values are heuristic units, not measurements.
4. **Conflict/composition check:** remove packs whose native-I/O or explicit
   distribution constraints conflict. Report per-pack rejection reasons.
5. **Smallest sufficient capability set:** select one unique top pack and
   emit exactly `<task_type>:<pack>`. There are no dependencies to activate in
   v0.1; no fallback union of competing packs. A top-two difference <=3 is an
   abstention, even though pack-ID sorting makes the trace order stable.
6. **Risk gate:** compare workflow required_ops to declared permitted_ops.
   Reconstruction additionally needs reverse_engineering=true. Privileged work
   is outside executable MVP scope and always requires authorization review.
7. **Plan:** emit one primary DAG node referencing the supplied type's workflow
   and the selected pack. Dependencies are empty. Every node has executable=false.

Scoring: platform_fit=30, interaction_fit=25, runtime_fit=10 after eligibility;
distribution_fit=10 for an explicit supported distribution; existing_stack_fit=20
for an exact pack stack ID; verification_fit=5 if evidence.verifiers contains
all required verifier IDs; accessibility_fit=5 for declared accessibility needs.
The accessibility term denotes a requirement, not measured compliance.
Subtract dependency_cost, maintenance_cost, context_cost, runtime_cost,
licensing_penalty, security_penalty and unsupported_penalty from pack data.
All seven penalties are explicit even when zero. Every candidate has all 14
terms. Rejected candidates have eligibility/conflict reasons, not fabricated
fitness scores. Scores never grant authority.

The router adds no good-practice steps. Workflow stage sequences are data in
`contracts/workflows.json`: for example debug's reproduce/diagnose/fix/regress/
audit sequence is carried into one workflow node, never appended as separate
router-selected capabilities. All other type cells preserve their supplied
entry type. Verifiers are requirements of the selected platform contract,
not extra selected lifecycle stages. None are executed by this version.

## Abstention and output

`schemas/route-result.schema.json` specifies the stable output structure.
Results include task_type, derived families, selected_pack, capabilities,
required_verifiers, candidates with score terms, rejected pack reasons,
missing_fields, reasons, tie_margin, confidence, risk_gate and plan.

`needs_information` supplies sorted, unique missing_fields. Ordinary entries
are unresolved input paths. A tie requests constraints.existing_stack (or a
distinguishing constraint); `supported_pack` requests supported target values
or a vetted registry extension; `constraints.<field>_evidence` requests pack
measurement support. These last entries are remediation requests, not JSON
properties to insert. `unclear` requests task_type. Invalid explicit versions
are errors rather than an abstention. Insufficient permissions produce an
otherwise valid route with risk_gate.status=requires_authorization.

Confidence is categorical: insufficient, typed (no platform needed), or
typed_unique. It is neither a probability nor authority. All output ordering
is deterministic: no timestamps, random IDs, absolute paths or durations in
route results. JSON uses fixed pretty serialization and a final newline.
CLI errors go to stderr with exit code 2; benchmark assertion failure exits 1;
successful routes (including abstentions) exit 0.

## Pack contract

Eight versioned TOML packs are compiled into the executable, loaded as data
when routing or inspecting them. Adding/editing a pack requires rebuilding;
there is no runtime plugin loading or arbitrary code execution.

Each pack declares: id, version, taxonomy_version; typed eligibility arrays;
interaction_profiles/primary_inputs; offline/native-I/O support; stack ID;
framework_candidates (id/name/fit_notes); required_verifiers; conflicts;
official_references; and seven explicit nonnegative cost/penalty heuristics.
Frameworks are candidates, not mandates. Official links are generic starting
references and are not claimed to have been fetched during this offline build.
Licensing, support and deployment suitability need project-specific review.
No dynamic tool installation or heavyweight reference loading occurs.

## Run-state layout and resumption boundary

A future executor or caller may persist:

```text
.batisseur/runs/<id>/
  intent.json
  route.json
  plan.json
  state.json
  decisions.jsonl
  evidence.jsonl
  outputs/
```

The run ID is caller-owned; it is not generated by route. A future state
machine may use planned/blocked/running/verified/failed with completed DAG node
IDs and artifact hashes. Rechecking intent/pack/taxonomy versions is required
before resuming. v0.1 writes none of these files and makes no resume, execution,
audit-completion or release-provenance certification claim.

## BuildBench v0

`bench/buildbench-v0.jsonl` freezes 16 cases with independent expected status,
pack, families, capability IDs, verifier IDs and gate status. Cases cover the
eight packs, existing-app bug, finished security audit, unknown Electron
reconstruction, unsupported API library, Android-only collision, native/web
touch-panel collision, native HMI, and interface design for a browser panel.
Four cases must abstain. Native-HMI read-only permission tests risk escalation.

Metrics: correct routes/total and percentage; expected/correct abstentions;
unexpected capability activations; verifier-selection matches; risk/permission
matches; identical serialized results across three evaluations for every case.
Independent CLI subprocess tests also compare three benchmark outputs and
three route outputs byte-for-byte. `passed` requires every measured metric
to pass; the benchmark never updates its own frozen expected values.

context_bytes is a deterministic partial accounting of embedded benchmark,
taxonomy and input-schema bytes, explicitly labeled in measurement_notes.
It is not a token count, working-set measurement, or total loaded context.
Tokens and latency are null: no tokenizer or timing instrumentation exists.
Device testing, real pilot fidelity, CPU/RAM and cold/warm build benchmarks
remain future work. Frozen-set perfection is not general routing accuracy.

## Out of scope

Execution, installations, service activation, runtime adapters, model calls,
fuzzy intent extraction, artifact reverse engineering, live reference fetching,
UI building, external API access, release publication, dynamic pack discovery,
full JSON Schema validation for arbitrary third-party schemas, automatic
permission acquisition, and persistent run management are outside v0.1.
No adapter or release claim follows merely from an advisory route.
