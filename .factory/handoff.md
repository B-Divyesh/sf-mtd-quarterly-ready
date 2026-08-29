# Quarterly Ready — repair 6 handoff

## Outcome

Verification-7's release blockers were repaired on top of report commit
`4180abb9a5f2ff4a9fec4a1f0d29368dfb7ad2f1`. The focused implementation commit
is `79064221cbec6c20d9c8a4efb2cc3a52f9f6dca0`; the deployment script builds and
waits for the containing final commit SHA before it can return success.

## Repairs

- Reproduced the exact cold failure with empty `CARGO_HOME` and
  `CARGO_TARGET_DIR`, `CARGO_BUILD_JOBS=1`, and the first declared claim. The
  old 120-second Playwright limit expired while Rust compiled.
- Raised the declared web-server allowance to 600 seconds. The same claim then
  passed from new empty caches; Rust took 2m00s and the command took 2.1m.
- Added standard UK quarter generation. `/records` opens the current quarter,
  offers current and future periods, and can keep creating later quarters.
  Browser documents and server workspace IDs are separated by quarter.
- Validated real calendar dates, matching UK quarter boundaries, period
  membership, positive amounts up to £1,000,000, type, and the category allow
  list before browser or server mutation.
- CSV import now reports the failing row and remains atomic. Regression probes
  cover `2026-02-30`, `2026-07-06`, zero, and `Bananas`; all leave the original
  ten demo rows and £260 income unchanged.
- Registered `free-quarter-persistence` in `.factory/claims.json`. Its clean
  browser test saves, reloads, rolls forward, and returns without cross-quarter
  data loss.
- `/share/demo` now keeps the persistent demo banner, Reset demo, and Start for
  real controls.
- The mobile “I checked these figures” target now measures 44 CSS px.
- Canonical, Open Graph URL/title, and Twitter title now follow SPA routes. The
  unused, non-conforming install manifest was removed; offline browser support
  remains.
- Added an opt-in safe paid fixture. It authorises only one byte-for-byte
  bundled synthetic document, declares `charges: false` and
  `files_with_hmrc: false`, never contacts billing or an integration, and
  returns `fixture_only_no_filing`. Live verification exercises both paid
  accountant-link and submission policy with it.
- Added HSTS and explicit revalidation policy for HTML, API, and service-worker
  responses while retaining immutable hashed assets.

## Verification evidence

Commands run from `/work/repo`:

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=repair-verification cargo build --release
npm run test:deploy-contract
# every `test` command in .factory/claims.json, separately and in order
```

Results:

- Clean install: 60 packages, 0 vulnerabilities.
- Claims: 18/18 independent commands passed. The first claim also passed from
  empty Rust registry and target directories with one compiler job.
- Full suite: 9 Vitest, 13 Rust, and 35 Playwright tests passed.
- TypeScript, Rust formatting, Clippy with warnings denied, deploy contract,
  and optimized build passed.
- Production frontend: 41.02 kB JS / 13.40 kB gzip; 21.67 kB CSS / 5.33 kB
  gzip; mobile hero 23.00 kB.
- Chromium: 1440×900 desktop and 390×844 mobile passed with no overflow,
  console errors, or page errors. Current period was Q2 2026–27. Keyboard demo
  navigation and submission-dialog focus passed.
- Axe integration: 0 serious or critical findings on `/`, `/demo`, `/privacy`,
  and `/terms`.
- Mobile review control: 44 px. All tested header/footer controls: at least
  44×44 px.
- Offline/update: service-worker readiness, network-off reload, preserved demo
  records, and “Offline — browser copy active” passed.
- Privacy: demo flow had no cookies or cross-origin requests.
- HTTP: CSP with header-only `frame-ancestors`, HSTS, nosniff, referrer and
  permissions policies, `no-cache` shell/API responses, and immutable hashed
  assets were observed.
- Lighthouse mobile: performance 100, accessibility 100, best practices 100,
  SEO 100; FCP 1.2 s, LCP 1.5 s, TBT 20 ms, CLS 0.
- Default runtime: started with only `PORT`; logged generated key state and
  build identity without a secret. A 100-request concurrent health smoke
  returned 100 HTTP 200 responses.
- Docker was unavailable locally. The deploy-contract test passed; the Azure
  ACR deployment performs the real multi-stage container build from a
  `.git`-free source upload.

## Release and operations

Deploy with:

```sh
./scripts/deploy-container.sh
EXPECTED_BUILD_SHA="$(git rev-parse HEAD)" npm run verify:live
```

`verify:live` checks exact `/health` identity, both hosted checkout creation
paths, durable storage, invalid quarter/row rejection, the safe entitlement
share/submission path, real 404 behavior, and read/write rate limits with
`Retry-After`.

The container remains the original `web-with-backend` artifact: Vite and
TypeScript served by Rust/axum with encrypted SQLite. It starts with only
`PORT`; `SAFE_QA_FIXTURES=1` is optional and limited to the exact synthetic
fixture.

## Known gaps

No release-blocking product gaps remain. The safe fixture proves policy and
wiring without making a real purchase or tax filing; it is intentionally not
evidence of an HMRC filing.
