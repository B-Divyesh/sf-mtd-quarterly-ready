# Sociobot subscription registration contract

Quarterly Ready never stores Dodo or other payment-provider identifiers. The
Sociobot billing controller must own those identifiers and expose these two
enabled recurring plans under one product slug:

| Product slug | Plan query value | Customer price | Billing interval |
| --- | --- | --- | --- |
| `mtd-quarterly-ready` | `monthly` | GBP 1,200 pence | monthly |
| `mtd-quarterly-ready` | `annual` | GBP 9,900 pence | yearly |

Both registrations require:

- enabled production checkout;
- entitlement product `mtd-quarterly-ready` so one licence verifies at
  `/api/v1/products/mtd-quarterly-ready/verify`;
- return URL `https://mtd-quarterly-ready.sociobot.in/records` with the
  controller-provided `license` query value preserved;
- recurring subscription mode, with Sociobot as merchant of record;
- the same entitlement for live accountant links and approved-integration
  submissions.

The browser URLs are deliberately stable and contain no provider IDs:

- `https://api.sociobot.in/api/v1/products/mtd-quarterly-ready/checkout?plan=monthly`
- `https://api.sociobot.in/api/v1/products/mtd-quarterly-ready/checkout?plan=annual`

After registration, `npm run verify:live` requires each URL to return a hosted
checkout redirect. Registration happens in the controller, not in this
repository, because repository code must not contain payment-provider IDs or
credentials.
