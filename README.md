# Quarterly Ready

Turn UK business records into a checked quarterly update.

Quarterly Ready is for sole traders, tutors, and landlords preparing for Making Tax Digital. Choose a UK quarter, capture records, review categories, check totals, and prepare an accountant handoff.

It does not give tax advice. When an approved HMRC integration is unavailable, the product provides a reviewed handoff for recognised software and does not claim a submission was made.

Account sign-in is prepared for Sociobot Entra CIAM but is not enabled on the live service until the product’s identity registration is supplied. Browser records and free downloads continue to work without an account.

## Try the demo

Open `http://localhost:8080/demo` after starting the app. The hosted demo is at <https://mtd-quarterly-ready.sociobot.in/demo>.

The sample is a tutoring business with ten realistic transactions. One category and one receipt need attention.

Demo changes use the `demo:quarterly-ready:document` browser key. They never read or write real workspace data.

## What v1 includes

- Manual transaction and receipt capture.
- Bank CSV import.
- Income, cost, and net quarter totals.
- Category and receipt checks with explicit human review.
- A complete accountant CSV download.
- A reviewed JSON handoff for HMRC-recognised software.
- Read-only accountant links with a 30-day expiry.
- Encrypted SQLite documents and a hash-chained audit log.
- An offline browser copy after the first visit.
- When Sociobot identity sign-in is configured: businesses with member-scoped quarters, explicit browser-quarter migration, account export, and self-service account deletion.

The free version keeps your working quarter and all downloads. Each quarter uses separate browser storage; signed-in account quarters also use encrypted server storage. Receipt files use IndexedDB instead of localStorage and remain in the browser. A £12 monthly or £99 annual Sociobot subscription adds live accountant links. Checkout and server-side subscription checks use the Sociobot billing API. HMRC submission is a later M3 dependency and is not available.

## Accounts and account data

The `/account` page is the account entry point. It never moves demo data. Once sign-in is configured, a user creates a business, then explicitly chooses **Move this browser quarter**. A retried move has a migration ID, so it cannot overwrite the first moved copy. Later browser changes save to that selected account business.

Each account route derives the user from an HTTP-only server session and checks the business membership. It never accepts a browser workspace ID as authority. The account page can download the user’s account data as JSON or delete that user’s owned businesses, account quarters, account links, sessions, and membership. Browser records and browser-only receipt files are left unchanged by account deletion.

The two required controller-side subscription registrations are specified in [`.factory/billing.md`](.factory/billing.md). The application POSTs to the correct monthly or annual Sociobot product endpoint, then follows only its returned Dodo checkout URL. Provider product and price IDs stay in the Sociobot controller.

## Run locally

Requirements: Node 22+, npm, and current stable Rust.

```sh
npm ci
npm run build
PORT=8080 cargo run
```

Then open <http://localhost:8080>. No environment variables are required.

For frontend work, run the API and Vite in separate terminals:

```sh
PORT=8080 cargo run
npm run dev
```

## Test

```sh
npm test
npm run typecheck
npm run verify:live
# with the server running locally or VERIFY_ORIGIN set to a deployed URL
npm run verify:concurrency
# with the server running locally or VERIFY_ORIGIN set to a deployed URL
npm run verify:rate-limit
# with the server running locally or a deployed URL
npm run verify:url -- http://127.0.0.1:8080/demo
```

This runs TypeScript unit tests, Rust tests, a clean frontend build, Playwright claim tests, and Axe checks. Claim definitions live in [`.factory/claims.json`](.factory/claims.json).

`verify:url` is the repeatable browser smoke check used by release QA. It checks the title, language, one main landmark, one H1, image alt attributes, and browser console/page errors for the URL supplied.

`verify:rate-limit` sends paced requests through one keep-alive connection. This keeps the ingress client identity stable while it proves the 40-read and 12-write burst allowances, the OAuth callback's shared write quota, and positive `Retry-After` responses.

`verify:concurrency` runs two independent batches of ten simultaneous saves, waits 1.5 seconds, and confirms every acknowledged document remains readable.

The M2 account-boundary checks are also runnable without an external identity registration because they use signed server-session fixtures:

```sh
cargo test claim_tenant_isolation
cargo test claim_browser_quarter_migration_is_explicit_and_idempotent
cargo test claim_account_export_and_delete
```

Set `EXPECTED_BUILD_SHA` before `npm run verify:live` to check the deployed identity, HMRC capability disclosure, checkout, 404, empty-workspace, and rate-limit policies. Run `npm run verify:topology` with Azure access to assert one running replica and an Azure Files mount at `/data`.

For a release decision, run `EXPECTED_BUILD_SHA=<commit> npm run verify:release`. It requires an immutable image identity, one-replica Azure Files topology, and confirms that this product deployment remains handoff-only with no direct HMRC submission capability.

## Container

```sh
docker build --build-arg BUILD_SHA=$(git rev-parse --short HEAD) -t quarterly-ready .
docker run --rm -p 8080:8080 -v quarterly-ready-data:/data quarterly-ready
```

The container runs as a non-root user and listens on `PORT`. `/health` returns the build SHA and the startup-resolved safe-fixture state used by release verification.

Production uses one active container replica. SQLite stays on the container's local filesystem because Azure Files cannot reliably provide SQLite byte-range locks. The service serialises each mutation, commits it, streams a synced database snapshot to the mounted Azure Files share at `/data`, and only then acknowledges success. The AES-256-GCM key and restart snapshot live on `/data`. Keep one replica unless records and rate limiting move to shared infrastructure.

## Data and configuration

The server defaults to `/data` in the container and `./data` locally. Its live SQLite path defaults to `/tmp/quarterly-ready` and can be overridden with `DATABASE_DIR`. It creates its AES-256-GCM key on first boot and stores it with restricted permissions when the filesystem supports POSIX modes.

Optional local runtime variables are `DATA_DIR`, `DATABASE_DIR`, `FRONTEND_DIR`, and `SOCIOBOT_BILLING_URL`. `PORT` defaults to `8080`.

To enable account sign-in after the authorised Sociobot Entra CIAM registration exists, configure the public OIDC issuer and client ID through `OIDC_ISSUER` and `OIDC_CLIENT_ID`. `OIDC_REDIRECT_URI` is optional and defaults to the product’s HTTPS callback. The server uses OIDC discovery, PKCE, ID-token signature validation, a nonce, and an HTTP-only session; it does not take an OAuth client secret. With those values absent, `/health` reports `accounts_configured:false` and `/account` clearly says that account sign-in is not available yet.

The product-owned deployment contract accepts only `DEPLOYMENT_MODE=handoff-only` (its default). It sets `HMRC_INTEGRATION_MODE=not_configured`, supplies no direct-submission environment variables, and declares no container-app secret references. Any other deployment mode exits before a build or runtime change. This is intentional: Quarterly Ready provides records, CSV, and reviewed accountant handoffs, but it does not claim to file an HMRC return.

Release deployment sets `SAFE_QA_FIXTURES=1` in both the image and Container App template. It refuses to report success unless `/health` reports the fixture, `not_configured` HMRC capability, the immutable image identity, and the durable one-replica topology. The fixture token authorises one exact synthetic document and never files a return.

See [privacy](https://mtd-quarterly-ready.sociobot.in/privacy), [terms](https://mtd-quarterly-ready.sociobot.in/terms), and [the visual thesis](.factory/design.md).

## Licence

MIT. See [LICENSE](LICENSE).
