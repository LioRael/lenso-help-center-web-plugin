# Plugin card: `lenso.help-center.web`

## User job

A customer needs to solve a concrete problem. They search only published help content first; when that is insufficient, they authenticate, submit the same problem to support, keep a receipt, check public replies, and optionally attach a bounded text file.

## First observable behavior

`GET /help` returns a responsive self-service page. A search produces published article references; an empty result leads directly to the support form instead of an Overview, inbox, or operations dashboard.

## Ownership and deletion boundary

This Plugin owns HTTP presentation, strict public input boundaries, redacted request diagnostics, and orchestration of the configured organization. It owns no article, customer, case, message, attachment, credential, or authorization fact and has no database or lifecycle.

Removing its App Instance removes the Help Center routes and UI. Knowledge Base, Auth, Support Intake, Support Attachment, their records, and agent-facing surfaces remain unchanged. No Kernel branch or private table is required to delete it.

## Contract

- Plugin ID: `lenso.help-center.web`
- Release: `0.1.0`
- Implementation: linked native Rust only
- Root slot: `web`
- Provides: `lenso.http.endpoint@1` descriptor `1.1.0`
- Requires exactly one: `lenso.auth@1` `1.0.0`
- Requires exactly one: `lenso.knowledge-base@1` `1.0.0`
- Requires exactly one: `lenso.support-intake@1` `1.0.0`
- Requires exactly one: `lenso.support-attachment@1` `1.1.0`
- Configuration: required immutable `organization_id`, `auth_issuer`, and Auth assertion public key
- State/lifecycle: stateless; no prepare/deactivate work

## Final authorization

Published search/read is granted by the Knowledge Base owner through an exact caller + organization public-read grant. Support routes first authenticate the HTTP credential through `lenso.auth@1`, then verify the assertion proof, configured issuer, current validity, user actor kind, and exact HTTP route audience before attaching it to the same InvocationContext. Support Intake compares its explicit requester subject, while Support Attachment independently verifies its own `upload_and_attach` audience and asks Support Case Authorization for the final case/message decision. The Web Plugin never grants itself case access.

## Limits and failure semantics

- Search query: 1–240 UTF-8 bytes; result limit 1–20.
- Titles: 1–240 UTF-8 bytes; descriptions: 1–12,000 bytes.
- Opaque refs and idempotency keys use a bounded ASCII alphabet.
- Attachment: public visibility only, basename filename, canonical Base64, valid non-empty UTF-8 text, at most 8 MiB, `text/plain` only.
- Delivery of the 8 MiB decoded limit requires a bounded 12 MiB-or-greater Ingress/proxy body limit because JSON Base64 expands the payload.
- Unknown generated Domain Errors become Runtime protocol violations.
- Forbidden case lookups and attachment attempts return the same not-found response as absent cases.
- Dependency Runtime Failures remain Runtime Failures; the Web layer does not fabricate success or fall back to another provider.

## Non-goals in this release

- Agent inbox, triage, assignment, or internal notes.
- Draft authoring or publishing UI.
- Anonymous support impersonation by email address.
- PNG/JPEG or arbitrary binary upload.
- Socket ownership, Host activation, provider selection, or hidden default Instances.
