# Sociobot subscription registration contract

Quarterly Ready never stores Dodo or other payment-provider identifiers. The
Sociobot billing controller must own those identifiers and expose these two
enabled recurring plans under one product slug:

| Product slug | Plan query value | Customer price | Billing interval |
| --- | --- | --- | --- |
| `mtd-quarterly-ready` | `monthly` | GBP 1,200 pence | monthly |
| `mtd-quarterly-ready-annual` | `annual` | GBP 9,900 pence | yearly |

Both registrations require:

- enabled production checkout;
- entitlement products `mtd-quarterly-ready` and `mtd-quarterly-ready-annual`;
  the application accepts a valid subscription token from either verification endpoint;
- return URL `https://mtd-quarterly-ready.sociobot.in/records` with the
  controller-provided `license` query value preserved;
- recurring subscription mode, with Sociobot as merchant of record;
- the same entitlement for live accountant links and approved-integration
  submissions.

The browser makes a deliberate `POST` to these stable controller URLs and then
redirects only to the returned Dodo URL. They contain no provider IDs:

- `https://api.sociobot.in/api/v1/products/mtd-quarterly-ready/checkout`
- `https://api.sociobot.in/api/v1/products/mtd-quarterly-ready-annual/checkout`

After registration, `npm run verify:live` requires each POST response to return
a hosted Dodo checkout URL. Registration happens in the controller, not in this
repository, because repository code must not contain payment-provider IDs or
credentials.

## Non-charging release fixture

The release deployment sets `SAFE_QA_FIXTURES=1` and checks
`GET /api/qa/entitlement` before it reports success. The endpoint returns a
public QA token and one exact synthetic document. The token authorises only that
unchanged document. It cannot authorise user records. Its submission result is
labelled `fixture_only_no_filing` and does not call Sociobot, Dodo, HMRC, or the
configured integration. `npm run verify:live` uses this path to prove the paid
share and submission policy without a purchase or tax filing.
