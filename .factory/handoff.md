# Quarterly Ready — M2 account foundation handoff

## Current status: verification 26 PASS; M2 foundation is implemented but not customer-live or accepted

Independent verification 26 reviewed implementation `08b2470eaff37e998e30323e98c2f87d8750baf6`, documentation commit `ef9442547aae1c0caff1a732b78cf4b89667a980`, and the later live build label `0dcc20f5fec98a476540ea9f684cb92879de6903`. It passed with **0 findings and 0 untested claims**. All 27 exact claim commands passed from a clean checkout; `npm test` passed 11 frontend unit, 22 Rust, and 59 browser tests. Live checks passed for fresh desktop/phone first-screen copy, one-click sample/reset, Axe, URL structure, account-unavailable truthfulness, one-replica `/data` persistence, 20 concurrent saves, and the 40/12 limit followed by 429 with `Retry-After`.

The live build differs from the implementation only in documentation/Graphify output; no runtime source, asset, dependency, Docker, or build configuration differs. The complete independent evidence and earlier-finding dispositions are in [verification-26.md](verification-26.md). No product code or pre-existing Graphify worktree changes were modified by verification.

Implementation `08b2470eaff37e998e30323e98c2f87d8750baf6` is live as `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:c365807e7e365a3f8f0ee016a5a4b686aa5333bff778a554ed0cdafc62085e98`. It adds the prepared account/session/tenant boundary while keeping M1 browser workflow and demo accepted and unchanged. Live identity configuration is absent, so `/account` says sign-in is unavailable; M2 is not an account, cross-device, or paid-lifecycle release. See [handoff-m2.md](handoff-m2.md) for current evidence and dependencies. The remainder preserves historical M1 evidence.

# Historical M1 repair 21 handoff

## Status: PASS

This repair closes Verification 24's only finding: public decorative labels
were not plain words. The current M1 product is still a non-filing
records-to-reviewed-handoff workflow.

- **Implementation commit:** `575aa8659469ab6f45bce623c3da560a82248895`
- **Earlier verification documentation commit:** `3a64f54` (the historical
  Verification 24 FAIL report)
- **Live build:** `575aa8659469ab6f45bce623c3da560a82248895`
- **Live immutable image:**
  `sociobotregistry.azurecr.io/sf-mtd-quarterly-ready@sha256:2ee8384c21c77c05dd2334c108bb34c65a1bc6093040b04fdc5c9734776afb3f`

The historical Verification 24 report remains a FAIL record. Its one copy
finding is now fixed and this handoff records the repair evidence.

## What changed

The first screen now begins directly with the job: **Turn records into a
checked quarterly update**. It names the audience (UK sole traders, tutors,
and landlords) and shows **Try it with sample data** as the first action.

All public mood/decorative labels were replaced at their source, on landing,
app, legal, share, and static 404 surfaces. Direct labels now include:

- **Quarter status**
- **How it works**
- **What this tool does not do**
- **Your data**
- **Accountant links from £12 a month**
- **Quarter checks**
- **Review your quarter**
- **Downloads and sharing**

The copy audit now includes the public-label inventory and terminology table.
The regression test uses fresh desktop and phone browser contexts, verifies the
job/audience/first action before scrolling, then enters the sample and checks
the named quarter status and usable download/sharing controls. It tests the
visitor outcome, not source strings.

## Verification

From the documented clean setup, `npm ci` completed with 60 packages and zero
audit vulnerabilities. All **24/24** exact commands declared in
`.factory/claims.json` passed independently. The managed `npm test` suite
passed **56/56** tests: type checking, 11 unit tests, 18 Rust tests, deploy
contract, production build, and browser tests. Formatting and Clippy also
passed. The production bundle is 15.32 kB gzip JavaScript and 5.32 kB gzip CSS.

The outcome regression was rerun after its fresh-context refinement. The live
browser run passed the plain-words desktop/phone test and the demo-isolation
claim. `verify:url` passed against `/demo` with the route title, `lang=en-GB`,
one main landmark, one H1, no missing image alternatives, and zero console
errors. Playwright Axe coverage remains free of serious and critical findings.

The live fresh phone and desktop checks confirmed, before scrolling:

- job: **Turn records into a checked quarterly update**;
- audience: the named UK sole-trader, tutor, and landlord audience;
- first action: **Try it with sample data**.

The phone sample then showed the persistent **Demo — sample data, nothing is
saved** label, the realistic Maya Patel Tutoring quarter, one uncategorised
transaction, and enabled CSV/accountant-link actions. The live demo-isolation
claim confirmed that its changes stay in `demo:` storage, use no cookies, and
make only same-origin requests. Reset remains available in the persistent
banner; real storage is not used by the sample.

## Deployment and backend checks

The durable deployment script built and released the immutable image above.
Live `/health` reports the implementation commit, safe QA fixtures, and
`hmrc_integration_mode: not_configured`.

The release verifier passed all of the following:

- single active/running replica with min/max replicas 1/1;
- product Azure Files storage mounted at `/data`;
- persistence through a replica restart and revision replacement;
- 20 concurrent acknowledged saved workspaces;
- monthly and annual checkout routing and the non-charging/non-filing QA
  fixture;
- handoff-only HMRC status;
- 40 read and 12 write allowances, followed by 429 responses with
  `Retry-After` (including the OAuth callback path).

No provider, billing-registration, volume, or replica configuration changed in
this M1 copy repair. Existing Graphify changes remain unmodified and unstaged.

## Current milestone and remaining dependencies

**M1, shipped:** record/import transactions, keep receipt evidence locally,
review categories and quarter checks, download CSV/HMRC handoff files, and
share the isolated sample accountant pack. It does not make a tax filing.

**M2, not shipped:** accounts, authenticated tenant isolation, cross-device
recovery, and the paid-customer purchase/restore lifecycle.

**M3 external dependency:** an approved HMRC provider contract and runtime
configuration for explicit taxpayer consent plus a controlled submission
acknowledgement. Direct HMRC submission is hidden while that dependency is not
configured.

## Running it

Use `npm ci`, then `npm test` and `npm run build`. Run the local service with
`npm run serve`; the one-click isolated sample is available at `/demo`. The
full demo/storage details are in [demo.md](demo.md); public claims and their
exact commands are in [claims.json](claims.json).

## Verification 25 — PASS

Independent verification on 5 September 2026 passed with **0 findings and 0 untested claims**. The product implementation reviewed was `575aa8659469ab6f45bce623c3da560a82248895`; the repair documentation commit was `bc81f80dbc0d2a8f88741f4f777fbb7f36faf604`. Live `/health` carries the later report/Graphify-only descendant `73ddca8e56958abbdccff19a140d505f6bd8527a`, and the live immutable image is `sha256:06a13cf9ca86518490bcc5ac4257b63d97fe4a7fa05b744f22753442746a316e`.

Fresh desktop and 390 px phone checks showed the job, audience, and **Try it with sample data** action before scrolling. All 24 exact claim commands passed from a clean clone; `npm test` passed 56/56; live release verification proved the single replica, `/data` Azure Files mount, persistence, concurrent saves, checkout URLs, handoff-only HMRC state, and 40/12 rate limits with 429 and `Retry-After`. `verify:url` and live Axe integration passed for the public and legal routes. See [verification-25.md](verification-25.md) for complete evidence and earlier-finding dispositions.

Remaining work is intentionally outside accepted M1: M2 accounts/tenant isolation/cross-device paid lifecycle and M3 approved-provider submission.

## Strict review 1 — PASS

Strict review 1 on 5 September 2026 passed with **0 findings and 0 untested
claims**. It reviewed implementation
`575aa8659469ab6f45bce623c3da560a82248895`, with the separate prior
verification documentation commit
`8b7b017fad59364c0e1886378b7164d446e29a59` and live health build label
`73ddca8e56958abbdccff19a140d505f6bd8527a`. The label is a
documentation/Graphify/accessibility-test-only descendant; no runtime source,
asset, dependency, Docker, or build configuration changed beyond the
implementation candidate.

Fresh desktop and phone browser contexts showed the job, target audience, and
**Try it with sample data** before scrolling. The sample showed its persistent
isolation label, ten realistic Maya Patel Tutoring records, reset, and Start
for real; reset restored the sample with no errors. This independently closes
Verification 24's decorative-label finding: the public interface now uses
plain task/result names such as **Quarter status**, **How it works**, and
**Downloads and sharing**.

From `npm ci`, all 24 exact claim commands passed; `npm test` passed
56/56; production output is 15.32 kB gzip JS and 5.32 kB gzip CSS.
`verify:url` passed live for landing, demo, privacy, and terms. Fresh live
Axe scans reported zero serious or critical issues. The designed unknown route
returns the expected HTTP 404 and offers Return home.

`EXPECTED_BUILD_SHA=73ddca8e56958abbdccff19a140d505f6bd8527a npm run
verify:release` passed against the immutable
`sha256:06a13cf9ca86518490bcc5ac4257b63d97fe4a7fa05b744f22753442746a316e`
image: one replica, durable `/data` mount, persistence, 20 concurrent saves,
checkout fixtures, handoff-only HMRC status, and 40/12 rate limits followed
by 429 with `Retry-After`.

M1 remains the only shipped milestone. M2 accounts/tenant isolation and
paid-customer lifecycle, plus M3 approved-provider consent/submission, remain
unshipped external-dependency work. No product code, deployment configuration,
or pre-existing Graphify changes were modified by this review. See
[review-1.md](review-1.md) for the complete evidence.
