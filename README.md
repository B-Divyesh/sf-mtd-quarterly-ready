# Quarterly Ready

Turn UK business records into a checked quarterly update.

Quarterly Ready is for sole traders, tutors, and landlords preparing for Making Tax Digital. Choose a UK quarter, capture records, review categories, check totals, and prepare an accountant handoff.

It does not give tax advice. Direct submission appears only when the server confirms that an approved integration is configured. Otherwise, the product provides an honest reviewed handoff for recognised software.

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
- An optional MTD-compatible submission path when a real approved integration is configured, after explicit human review.
- Read-only accountant links with a 30-day expiry.
- Encrypted SQLite documents and a hash-chained audit log.
- An offline browser copy after the first visit.

The free version keeps your working quarter and all downloads. Each quarter uses separate browser and encrypted server storage. Receipt files use IndexedDB instead of localStorage and remain in the browser. A £12 monthly or £99 annual Sociobot subscription adds live accountant links. Checkout and server-side subscription checks use the Sociobot billing API.

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
```

This runs TypeScript unit tests, Rust tests, a clean frontend build, Playwright claim tests, and Axe checks. Claim definitions live in [`.factory/claims.json`](.factory/claims.json).

Set `EXPECTED_BUILD_SHA` before `npm run verify:live` to check the deployed identity, HMRC capability disclosure, checkout, 404, empty-workspace, and rate-limit policies. Run `npm run verify:topology` with Azure access to assert one running replica and an Azure Files mount at `/data`.

For a release decision, run `EXPECTED_BUILD_SHA=<commit> npm run verify:release`. It requires the live approved HMRC integration as well as the one-replica Azure Files topology; it fails rather than approving an honest-handoff-only deployment as the researched submission product.

## Container

```sh
docker build --build-arg BUILD_SHA=$(git rev-parse --short HEAD) -t quarterly-ready .
docker run --rm -p 8080:8080 -v quarterly-ready-data:/data quarterly-ready
```

The container runs as a non-root user and listens on `PORT`. `/health` returns the build SHA and the startup-resolved safe-fixture state used by release verification.

Production currently uses one container replica because SQLite and the per-client limiter are process-local. Add a shared database and distributed limiter before increasing that replica count.

## Data and configuration

The server defaults to `/data` in the container and `./data` locally. It creates its AES-256-GCM key on first boot and stores it with restricted permissions.

Optional variables are `DATA_DIR`, `FRONTEND_DIR`, `SOCIOBOT_BILLING_URL`, `HMRC_INTEGRATION_URL`, and `HMRC_INTEGRATION_TOKEN`. `PORT` defaults to `8080`.

Release deployment sets `SAFE_QA_FIXTURES=1` in both the image and Container App template. It refuses to report success unless `/health` reports `safe_qa_fixtures: true` and `/api/qa/entitlement` responds. That endpoint returns a token for only one exact bundled synthetic document. Its submission response is marked `fixture_only_no_filing`; it never contacts billing, Dodo, HMRC, or an integration.

For live submission, the controller must provision both `mtd-quarterly-ready-hmrc-integration-url` and `mtd-quarterly-ready-hmrc-integration-token` in the approved Key Vault. The deployment binds them as secret references without reading or printing their values. The integration must return a JSON `submission_id` or `correlation_id`; otherwise Quarterly Ready reports that no submission was made. With no integration configured, the UI does not offer direct submission and continues to provide records, CSV, and handoff downloads.

See [privacy](https://mtd-quarterly-ready.sociobot.in/privacy), [terms](https://mtd-quarterly-ready.sociobot.in/terms), and [the visual thesis](.factory/design.md).

## Licence

MIT. See [LICENSE](LICENSE).
