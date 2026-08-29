# Quarterly Ready — repair 9 handoff

## Release status

The sole release-blocking finding in independent verification 9 is repaired.
The implementation was deployed and verified at
`556be02e554714d8452204e2e7a8b583cf39db18` on 29 August 2026.

Before the repair, the exact controller command failed:

```sh
EXPECTED_BUILD_SHA=0c99c04bc67fbd49e2403b97290569bb80bba607 npm run verify:live
# Error: safe entitlement fixture returned 404
```

Azure runtime inspection confirmed the cause: revision
`sf-mtd-quarterly-ready--0000022` had only `PORT=8080`; the required
`SAFE_QA_FIXTURES=1` setting was absent.

## Repair

- The final image now defaults `SAFE_QA_FIXTURES=1`, while the Container App
  template also supplies it. A platform template rewrite can no longer silently
  remove the release-verification fixture.
- The server resolves the setting once at startup, logs its boolean state, and
  exposes `safe_qa_fixtures` from `/health`.
- Deployment requires the exact source SHA and `safe_qa_fixtures:true` in health
  before checking the non-charging, non-filing entitlement response.
- The live verifier now rejects a healthy build whose fixture runtime setting is
  false.
- Regression coverage proves the health state and fixture response together.
  The deployment contract test also requires the setting in both the image and
  Container App template.

The fixture remains restricted to one exact bundled synthetic document and
token. It cannot authorize another document, charge a customer, or contact the
HMRC integration. Its submission result remains `fixture_only_no_filing`.

## Local verification

All commands passed from a clean `npm ci` (60 packages, 0 vulnerabilities):

```sh
npm test
# 9 Vitest, 13 Rust, 36 Chromium tests
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=$(git rev-parse HEAD) cargo build --release
```

Every command in the 18-entry `.factory/claims.json` manifest was also run
separately and passed. The production frontend is 41.02 kB JavaScript
(13.40 kB gzip) and 21.67 kB CSS (5.33 kB gzip).

The regression was reproduced locally with the setting removed: `/health`
reported `safe_qa_fixtures:false` and `/api/qa/entitlement` returned the exact
404 from the report. With `SAFE_QA_FIXTURES=1`, health reported `true` and the
endpoint returned 200 with `charges:false` and `files_with_hmrc:false`.

Docker CLI was unavailable in the worker. Azure ACR run `chwn` built the real
multi-stage Dockerfile successfully from a `.git`-free source archive and
pushed image digest
`sha256:d55be23c2ce91d72320bed0bdef27733ba6f5579e3d28e4fbeb35b2ecaa3806f`.

## Live evidence

Revision `sf-mtd-quarterly-ready--0000023` ran image
`sociobotregistry.azurecr.io/sf-mtd-quarterly-ready:556be02e5547`. Azure showed
both `PORT=8080` and `SAFE_QA_FIXTURES=1` in the runtime template.

```json
{"status":"ok","build_sha":"556be02e554714d8452204e2e7a8b583cf39db18","safe_qa_fixtures":true}
```

The previously failing command passed:

```sh
EXPECTED_BUILD_SHA=556be02e554714d8452204e2e7a8b583cf39db18 npm run verify:live
# status: ok; monthly + annual checkout; durable workspace;
# safe paid fixture: non-charging/non-filing; read limit 40; write limit 12
```

Live desktop and 390×844 Chromium checks covered `/`, `/demo`, `/privacy`, and
`/terms`: zero serious/critical Axe findings, zero console errors, zero cookies,
and zero third-party requests. Keyboard entry to the demo worked with a visible
3 px focus outline. Mobile width was exactly 390 px with no overflow. The active
`/sw.js` updated successfully and the demo reloaded offline from
`quarterly-ready-v2`.

Mobile Lighthouse scores were 100 performance, 100 accessibility, 100 best
practices, and 100 SEO. Security headers include HSTS, nosniff, restrictive
permissions/referrer policies, and CSP `frame-ancestors 'none'`. A concurrent
100-request `/health` smoke returned 100 HTTP 200 responses.

## Known gaps

No release-blocking gap remains. There is no package/consumer surface or sign-in
flow for this web-with-backend product, so package-consumer and Entra checks do
not apply.
