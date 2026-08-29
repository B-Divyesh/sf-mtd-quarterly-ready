# Demo sandbox

## Entry point

- Hosted: `https://mtd-quarterly-ready.sociobot.in/demo`
- Local: `http://127.0.0.1:8080/demo`
- Direct query links are not needed; `/demo` is a real route.

## Sample data

The demo opens “Maya Patel Tutoring” for 6 April to 5 July 2026. It has ten records across lesson income, travel, office costs, and professional fees.

One bank transfer needs a category. One expense needs a receipt. These gaps make the review and completion states testable.

## Isolation and reset

Demo transaction metadata uses the localStorage key `demo:quarterly-ready:document`. Receipt files use the `quarterly-ready-receipts-v1` IndexedDB database with `demo:` keys. Real browser data uses `quarterly-ready:document:<quarter-start>`, `real:` receipt keys, and a separate workspace ID for each quarter.

Demo actions never call the workspace, share, billing, or submission APIs. “Make accountant link” uses the fixed read-only `/share/demo` fixture. A direct submission control appears only when the server confirms an approved integration is configured, and it remains disabled in demo mode.

Choose **Reset demo** in the persistent banner to restore the sample and clear its receipt files. Choose **Start for real** to discard all demo storage and open an empty real quarter.

## Verification paths

- Change the unresolved category and watch the readiness dial move.
- Attach a small image or PDF to “Whiteboard markers”.
- Import a CSV with `date`, `description`, `amount`, `type`, and `category` columns.
- Confirm the figures and download the accountant CSV or HMRC handoff JSON.
- Make the demo accountant link and open its read-only pack.
- Confirm that `/share/demo` keeps the demo banner, Reset demo, and Start for real controls.
- Reload after disabling the network to verify the offline browser copy.

The authoritative automated checks are listed in `.factory/claims.json`.
