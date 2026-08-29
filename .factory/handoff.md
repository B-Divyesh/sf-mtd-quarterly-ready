# Quarterly Ready — verification 6 handoff

## Outcome: FAIL

Independent QA on 2026-08-29 tested commit
`a6e37268ebc04e26247e3c8499fa002a12cbf307` at
`https://mtd-quarterly-ready.sociobot.in`. `/health` reports that exact SHA.
The free demo, records review, export/handoff, privacy, offline, accessibility,
and live rate limiting passed. Release is blocked because the monthly and
annual Sociobot checkout URLs both return HTTP 404 rather than hosted checkout
redirects. The server also accepts malformed transaction objects through
`PUT /api/workspace`.

See [verification-6.md](verification-6.md) for exact commands, observations,
claim-by-claim results, and severity-ranked defects.

## How verified

```sh
npm ci
npm test
EXPECTED_BUILD_SHA=a6e37268ebc04e26247e3c8499fa002a12cbf307 npm run verify:live
```

All local claims and quality suites passed; the final command correctly fails
on the reproducible production checkout 404. Public rate allowance observed:
40 reads/s and 12 writes/s, then 429 with `Retry-After: 1`.

## Required next steps

1. Enable/register both production Sociobot checkout plans and prove their
   redirects without a live charge.
2. Add strict server-side validation for every transaction record.
3. Re-run the full verification and exercise paid accountant-link and approved
   integration submission paths with an authorised test licence.
