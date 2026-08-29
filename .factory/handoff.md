# Quarterly Ready — verification 7 handoff

## Outcome: FAIL

Independent QA on 2026-08-29 tested candidate
`b26820a560ce27db2b7271dac0e204931c4c6888` and
`https://mtd-quarterly-ready.sociobot.in` against the original work order and
researched brief. The candidate must not be released.

The first-read/demo gate passes, and the warm automated suite, normal demo
workflow, privacy checks, accessibility scan, offline reload, performance
budgets, checkout creation, persistence, and rate limits pass. Release remains
blocked by these findings:

1. `/health` reports `5d1f989b266e2f320c172266f4ef0056977b4eba`, not the
   candidate SHA.
2. The first declared claim command failed in the clean checkout when the
   120-second Playwright web-server timeout expired during the cold Rust build.
3. The real product is fixed to the already-ended 6 April–5 July 2026 quarter
   and offers no current/future quarter selection.
4. CSV import accepts impossible/out-of-quarter dates, zero values, and unknown
   categories; an unknown category is shown as Sales and exported into the
   HMRC handoff.
5. Landing/README copy claims the free product keeps a quarter, but that claim
   is absent from `.factory/claims.json`.

The live demo share route also loses the required persistent demo banner, the
paid accountant-link/HMRC submission could not be exercised without a safe
entitlement, and one mobile checkbox label is 40.8 px high.

Full commands, claim-by-claim results, measurements, and severity-ranked
defects are in [verification-7.md](verification-7.md).

## Verification summary

- Claims from clean candidate: **16/17 passed; 1 failed**. Warm rerun passed.
- `npm test`: PASS warm — 4 Vitest, 12 Rust, and 29 Playwright tests.
- TypeScript, deploy contract, exact Vite build, Rust release build, formatting,
  and Clippy: PASS.
- Bundle: 11.93 kB gzip JS, 5.27 kB gzip CSS, 23.00 kB mobile hero.
- Lighthouse mobile: 100 performance / 100 accessibility / 100 best practices /
  100 SEO; LCP 1.4 s, TBT 40 ms, CLS 0.
- Axe serious/critical: 0. Console/page errors: 0.
- Live limits: 40 reads/s and 12 writes/s, then 429 with `Retry-After: 1`.
- Sociobot licence verification: 30 accepted, then 429 with `Retry-After: 4`.
- Candidate backend: 8 concurrent writes and restart persistence passed.

## Required next steps

Repair the blockers above, deploy the exact repaired candidate, provision a
safe non-production paid entitlement/integration path, and repeat verification
from a genuinely cold cache.
