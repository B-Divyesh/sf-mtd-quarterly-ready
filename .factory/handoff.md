# Quarterly Ready — repair 16 handoff

## Release status

**Handoff-only deployment is buildable; the full MTD filing release remains
blocked by an external provider prerequisite.**

Independent verification 16 found one code-level QA defect and one production
configuration blocker. The live rate-limit regression is repaired. A genuine
approved HMRC provider endpoint, credential, and taxpayer-authorisation path
are still not provisioned, so this repair does not mislabel the bundled
non-filing sandbox as an approved filing integration.

The authoritative finding is in [`.factory/verification-16.md`](verification-16.md).

## Repair completed

- Added `scripts/verify-rate-limit.mjs`. It sends requests through one HTTP/1.1
  keep-alive agent with one socket and proves 40 reads or 12 writes are allowed,
  followed by `429` with `Retry-After: 1`.
- Changed both Playwright rate-limit regressions to use that stable connection
  against a public `VERIFY_ORIGIN`. Direct-origin tests still use Playwright's
  request context, so the backend policy remains covered independently.
- Changed `verify:live` and therefore `verify:release` to use the same stable
  public-ingress probe. The JSON evidence now records
  `stable_rate_limit_connection: true`.
- Added `npm run verify:rate-limit`, documented it in `README.md`, and protected
  the command with the deploy-contract test.

This fixes the verifier's medium transport finding without adding a spoofable
rate-limit bypass or weakening the first-hop `X-Forwarded-For` policy.

## Approved HMRC integration blocker

The default release deploy was reproduced and exited before building or
changing Azure:

```text
exit=1
missing approved HMRC integration secret references; refusing a release deployment
expected Key Vault secrets: mtd-quarterly-ready-approved-hmrc-url and mtd-quarterly-ready-approved-hmrc-token
```

Key Vault contains neither required approved-provider secret. The only related
entries are `mtd-quarterly-ready-hmrc-integration-url` and
`mtd-quarterly-ready-hmrc-integration-token`; their metadata explicitly labels
them “HMRC non-filing sandbox endpoint” and “non-filing sandbox attestation”.
They cannot truthfully satisfy the verifier's requirement for a genuine
approved provider with taxpayer consent.

Creating fake credentials, relabelling the sandbox, or making the server accept
its own response would falsely claim an HMRC filing. The guarded default deploy
therefore remains fail-closed. Production may be deployed only in the explicit
`handoff-only` mode until the product owner provisions the approved provider.

## Verification evidence

Run on 2026-08-29 from a clean `npm ci` (60 packages, 0 vulnerabilities):

| Check | Result |
| --- | --- |
| `npm test` | PASS — typecheck, 11 Vitest, 16 Rust, deploy-contract, production build, 49 Playwright. |
| Every exact command in `.factory/claims.json` | PASS — all 22 entries; 13 browser claim tests plus 6 distinct Rust claim tests. |
| `cargo fmt -- --check` | PASS. |
| `cargo clippy --all-targets -- -D warnings` | PASS. |
| `BUILD_SHA=repair-16-local cargo build --release` | PASS. |
| Production Vite build | PASS — JS 46.24 kB / 14.99 kB gzip; CSS 21.71 kB / 5.33 kB gzip. |
| `VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npm run verify:rate-limit` | PASS — read 41 first limited; write 13 first limited; both return `Retry-After: 1`; socket reuse observed. |
| Live Playwright rate regressions | PASS — 2/2 through the stable-connection helper. |
| Approved release preflight | EXPECTED BLOCK — required approved-provider Key Vault references are absent; no Azure mutation occurred. |
| ACR container build | PASS — build `ch16y`; image digest `sha256:497d18c91feaf701576a7dac567eee41402e2927ec41d3663fdc44e33bb59d78`. |
| Handoff-only deployment | PASS — implementation commit `a4d1d09c45e07970c0c2b4417c2202aa56f0fc5b`; revision `sf-mtd-quarterly-ready--0000051`. |
| Restart and revision replacement | PASS — encrypted workspace and accountant link remained readable after both operations. |
| Live full Playwright suite | PASS — 48 passed, 1 expected direct-origin-only skip. |
| Live `verify:url` on `/` and `/demo` | PASS — titles, `lang=en-GB`, one main/H1, alt attributes, and zero console/page errors. |
| Live desktop and 390 px browser checks | PASS — no overflow, cookies, third-party requests, console errors, or page errors. |
| Live Lighthouse mobile | PASS — performance 100, accessibility 100, best practices 100, SEO 100; FCP 1,200 ms, LCP 1,350 ms, TBT 21 ms, CLS 0, total 94,144 bytes. |
| `EXPECTED_BUILD_SHA=a4d1d09c… npm run verify:live` | PASS — exact identity, durable workspace, checkout, safe fixture, stable 40/12 rate limits. |
| `EXPECTED_BUILD_SHA=a4d1d09c… npm run verify:release` | EXPECTED BLOCK — `production has no approved HMRC integration configured`. |

The full Playwright run covers desktop Chromium, 390 px mobile, 200% text,
keyboard and dialog focus, touch targets, reduced motion, Axe serious/critical,
same-origin privacy, no cookies, offline reload, update policy, response headers,
unknown-route status, workspace persistence, receipts, and checkout boundaries.
No product copy or passing behavior changed in this repair.

Local Docker is unavailable in this worker. Azure Container Registry built the
same multi-stage `Dockerfile` from the source archive without `.git`, and the
result ran as the work-order container artifact.

## Run and verify

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=dev cargo build --release

# With a local server on port 8080:
VERIFY_ORIGIN=http://127.0.0.1:8080 npm run verify:rate-limit
npm run verify:url -- http://127.0.0.1:8080/demo

# Against the deployed service:
VERIFY_ORIGIN=https://mtd-quarterly-ready.sociobot.in npm run verify:rate-limit
EXPECTED_BUILD_SHA=<deployed-sha> npm run verify:live
```

## Required next step for a filing release

Provision a contractually approved MTD ITSA provider endpoint and credential in
the two required Key Vault entries, confirm its taxpayer consent flow, then run:

```sh
bash scripts/deploy-container.sh
EXPECTED_BUILD_SHA=<deployed-sha> npm run verify:release
```

Until that external prerequisite exists, use the honest fallback:

```sh
DEPLOYMENT_MODE=handoff-only bash scripts/deploy-container.sh
```

Pre-existing dirty `graphify-out/` verifier files were preserved and excluded
from this repair.
