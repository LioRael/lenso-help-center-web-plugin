# `lenso.help-center.web`

A linked native Web Plugin for the customer job “find a published answer, then contact support if it does not solve the problem.” It is intentionally not a support dashboard.

The public `/help` page and published Knowledge Base search/read routes do not require a credential. Creating or checking a support request and uploading an attachment require a Bearer credential. The Plugin resolves it through `lenso.auth@1`, then independently verifies its signature, issuer, validity interval, `user` actor kind, and exact route audience before attaching the ActorAssertion or using its subject.

## Routes

| Operation | HTTP path | Access | Delegates to |
| --- | --- | --- | --- |
| `help.center.web.page` | `GET /help` | Public | Static Plugin asset |
| `help.center.web.articles.search` | `POST /api/help/articles/search` | Public | `lenso.knowledge-base@1` |
| `help.center.web.articles.get` | `GET /api/help/articles/{article_ref}` | Public | `lenso.knowledge-base@1` |
| `help.center.web.support.create` | `POST /api/help/support` | Bearer | `lenso.auth@1` → `lenso.support-intake@1` |
| `help.center.web.support.status` | `POST /api/help/support/status` | Bearer | `lenso.auth@1` → `lenso.support-intake@1` |
| `help.center.web.support.attachment.upload` | `POST /api/help/support/attachments` | Bearer | `lenso.auth@1` → `lenso.support-attachment@1` |

Requester uploads are deliberately limited to one non-empty UTF-8 `text/plain` body of at most 8 MiB, public visibility, and a basename filename. The JSON `content` field is canonical padded Base64 through the generated portable `Bytes` type. PNG/JPEG and arbitrary binary upload are not claimed by this release.

## App wiring

The linked Host must make this factory available and activate one `web` root Instance with:

```toml
organization_id = "org_demo"
auth_issuer = "support-auth"
auth_assertion_public_key = "<URL-safe-base64 Ed25519 public key>"
```

Bind exactly one provider for each required Capability:

- `lenso.auth@1` descriptor `1.0.0`
- `lenso.knowledge-base@1` descriptor `1.1.0`
- `lenso.support-intake@1` descriptor `1.0.0`
- `lenso.support-attachment@1` descriptor `1.1.0`

Provider configuration must name the Help Center Instance explicitly: Knowledge Base public read grants must match its caller and `organization_id`; Support Intake and Support Attachment caller allowlists must match its exact Instance key. The Auth assertion issuer and public key must agree. Credentials need the exact `lenso.http.endpoint@1:<route-operation>` audience for the selected support route; attachment upload additionally needs `lenso.support-attachment@1:upload_and_attach` for the downstream owner check.

An 8 MiB decoded attachment expands to about 11.2 MiB as padded Base64 before the JSON envelope. Web Ingress and every upstream proxy must therefore use a bounded request-body limit of at least 12 MiB (`12582912` bytes); the default 1 MiB Web Ingress limit is not sufficient.

The in-progress Capability packages use immutable source revisions so a clean
checkout is reproducible. Crates.io publication remains deferred until those
owner packages publish compatible registry artifacts.

## Verification

Run from this repository:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

`EndpointTest` exercises the page/assets and public boundary without opening a socket. End-to-end authorization and cross-Plugin behavior belong to the Support App acceptance fixture because this Plugin does not own fake Knowledge Base, Auth, Support Intake, or Support Attachment state.
