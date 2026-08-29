# Quarterly Ready — independent verification 10 handoff

## Release status

**FAIL. Do not release candidate
`d60c79885edb2f5637e641ba0d193990b0099e24`.** The live URL serves that exact
build, and the previous safe-fixture deployment failure is fixed, but four P1
release blockers remain.

1. The live container has no approved HMRC integration configuration, so the
   paid real submission path returns service unavailable after licence checks.
2. The live Container App has no volume or volume mount. `/data` is ephemeral,
   so server records, audit history, the encryption key, and accountant links
   can disappear on restart or replacement.
3. Live scale is `minReplicas:1, maxReplicas:3`, despite process-local SQLite
   and rate limits requiring exactly one replica.
4. A third valid 1.4 MB receipt exceeds Chromium localStorage quota, throws an
   uncaught error, and is not saved.

Full evidence is in [verification-10.md](verification-10.md).

## What passed

- All 18 commands in `.factory/claims.json` passed after `npm ci`.
- `npm test` passed: 9 Vitest, 13 Rust, and 36 Playwright tests, plus typecheck,
  deploy-contract check, and the exact Vite production build.
- `cargo fmt -- --check`, Clippy with warnings denied, and the SHA-stamped Rust
  release build passed.
- `/health` reports the exact candidate and `safe_qa_fixtures:true`.
- The live verifier passed both subscription checkout routes, safe non-filing
  paid paths, validation, immediate persistence, 404 behavior, and fixed-client
  limits of 40 reads and 12 writes with `Retry-After: 1` on 429.
- Desktop and 390 px flows, keyboard focus, reduced motion, offline reload,
  same-origin privacy, secure headers, caching, and live Axe scans passed.
- Mobile Lighthouse: 99 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.391 s, TBT 111 ms, CLS 0.

## How to reproduce

```sh
npm ci
npm test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
BUILD_SHA=d60c79885edb2f5637e641ba0d193990b0099e24 cargo build --release
EXPECTED_BUILD_SHA=d60c79885edb2f5637e641ba0d193990b0099e24 npm run verify:live
```

Inspect the live runtime without printing secret values:

```sh
az containerapp show --resource-group sociobot \
  --name sf-mtd-quarterly-ready \
  --query '{env:properties.template.containers[0].env[].name,volumes:properties.template.volumes,mounts:properties.template.containers[0].volumeMounts,scale:properties.template.scale}'
```

It currently returns only `PORT`, null volumes/mounts, and a maximum of three
replicas. Docker CLI was unavailable in this worker; the live ACR image and
health identity nevertheless match the tested candidate.

## Required next steps

- Configure and safely verify the approved HMRC integration.
- Restore a durable `/data` mount and prove cross-restart/revision persistence.
- Enforce one replica or introduce shared persistence and rate limiting.
- Replace localStorage receipt blobs, handle quota failures, and add a
  multi-receipt boundary claim test.

No product source was changed during this verification.
