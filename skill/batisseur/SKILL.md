---
name: batisseur
description: Route a typed software-build intent through the deterministic batisseur CLI.
---

# batisseur

Use an existing batisseur executable or this repository's Cargo command.
Do not install tools, change permissions, or start services through this skill.

1. Read `schemas/build-intent.schema.json` only when preparing an intent.
   `task_type` must be a tt-1.1.0 ID; inspect `contracts/tt-1.toml` if needed.
   Never derive task_type from a framework, platform, family, or work_kind.
2. Put candidate typed values in an intent JSON file. Leave unknown values
   null/unknown or absent; never fabricate missing information.
3. Run `batisseur route intent.json --json` (or
   `cargo run -q -- route intent.json --json` from the checkout).
4. If route_status is needs_information, ask for the listed missing_fields.
   Remediation keys such as supported_pack require a registry review;
   do not insert them as intent properties.
5. For a selected route, check risk_gate separately. A score or family label
   never grants authority. This CLI returns an advisory plan, not execution.
6. Load only the selected pack with `batisseur packs show <id>`.
   Consult `docs/SPEC.md` for the workflow and verifier contract as needed.

Use `batisseur validate intent.json` for complete structural input checks.
Use `batisseur bench --json` for frozen routing regression metrics.
Unknown task/contract versions are errors. Do not silently translate them.
Workflow sequences are data; do not append extra lifecycle steps to the route.
