# Quarterly Ready

Turn UK business records into a checked quarterly update.

Quarterly Ready is for sole traders, tutors, and landlords preparing for Making Tax Digital. Choose a UK quarter, capture records, review categories, check totals, and prepare an accountant handoff.

It does not give tax advice. When an approved HMRC integration is unavailable, the product provides a reviewed handoff for recognised software and does not claim a submission was made.

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
- An MTD-compatible provider path after explicit human review.
- A reviewed submission path when an approved HMRC integration is configured.
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
# with the server running locally or a deployed URL
npm run verify:url -- http://127.0.0.1:8080/demo
```

This runs TypeScript unit tests, Rust tests, a clean frontend build, Playwright claim tests, and Axe checks. Claim definitions live in [`.factory/claims.json`](.factory/claims.json).

`verify:url` is the repeatable browser smoke check used by release QA. It checks the title, language, one main landmark, one H1, image alt attributes, and browser console/page errors for the URL supplied.

Set `EXPECTED_BUILD_SHA` before `npm run verify:live` to check the deployed identity, HMRC capability disclosure, checkout, 404, empty-workspace, and rate-limit policies. Run `npm run verify:topology` with Azure access to assert one running replica and an Azure Files mount at `/data`.

For a release decision, run `EXPECTED_BUILD_SHA=<commit> npm run verify:release`. It requires a Key Vault-backed approved HMRC integration and the one-replica Azure Files topology.

## Container

```sh
docker build --build-arg BUILD_SHA=$(git rev-parse --short HEAD) -t quarterly-ready .
docker run --rm -p 8080:8080 -v quarterly-ready-data:/data quarterly-ready
```

The container runs as a non-root user and listens on `PORT`. `/health` returns the build SHA and the startup-resolved safe-fixture state used by release verification.

Production uses one container replica. SQLite runs locally and writes its encrypted, fsynced snapshot to the mounted Azure Files share after each mutation. Keep one replica unless the service moves both records and rate limiting to shared infrastructure.

## Data and configuration

The server defaults to `/data` in the container and `./data` locally. It creates its AES-256-GCM key on first boot and stores it with restricted permissions.

Optional variables are `DATA_DIR`, `FRONTEND_DIR`, `SOCIOBOT_BILLING_URL`, `HMRC_INTEGRATION_URL`, `HMRC_INTEGRATION_TOKEN`, and `HMRC_INTEGRATION_MODE`. `PORT` defaults to `8080`.

Release deployment sets `SAFE_QA_FIXTURES=1` in both the image and Container App template. It refuses to report success unless `/health` reports the fixture, the approved integration capability, and the durable one-replica topology. The fixture token authorises one exact synthetic document and never files a return.

An approved HMRC provider URL and service token must be stored in the named Key Vault secrets before a release deployment. The deploy script binds them through managed-identity secret references and never reads or prints their values. Without that approval and credential configuration, the UI offers only records, CSV, and handoff downloads.

See [privacy](https://mtd-quarterly-ready.sociobot.in/privacy), [terms](https://mtd-quarterly-ready.sociobot.in/terms), and [the visual thesis](.factory/design.md).

## Licence

MIT. See [LICENSE](LICENSE).
