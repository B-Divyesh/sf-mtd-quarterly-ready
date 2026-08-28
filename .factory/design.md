# Quarterly Ready visual thesis

## Direction

**Mid-century instrument panel.** Quarterly tax work is a periodic check, not a daily accounting hobby. The interface borrows from a well-kept 1960s laboratory console: one calm readout, labelled controls, visible status lamps, and paper slips tucked beside the panel. This makes completion and unresolved work legible without pretending tax is playful.

The product is single-mode. Its warm, explicitly painted light treatment matches a physical instrument face and keeps the financial tables readable in daylight.

## Palette

| Token | Value | Use |
| --- | --- | --- |
| `--paper` | `#F2E9D5` | page and paper slips |
| `--panel` | `#DDD0B4` | instrument body |
| `--panel-deep` | `#C6B897` | recessed controls |
| `--ink` | `#17201E` | primary text |
| `--muted` | `#525A54` | secondary text (7:1 on paper) |
| `--orange` | `#B64222` | primary controls (5.1:1 with cream) |
| `--orange-dark` | `#7E2E19` | hover and focus |
| `--teal` | `#0B625C` | complete and positive status |
| `--amber` | `#8A5600` | needs-review status |
| `--danger` | `#982D2D` | errors |
| `--brass` | `#99743A` | hairlines and hardware |

Status never depends on colour: every lamp has a word and icon. All text and UI pairings meet WCAG AA.

## Type and spacing

- Display and readouts: Georgia, a self-host-free system serif with sturdy engraved forms.
- Controls and body: Arial/Helvetica system sans, with uppercase labels and slight tracking only for short instrumentation labels.
- Numeric tables use tabular figures.
- Type scale: 14, 16, 20, 28, and clamp(40–68) px.
- Spacing follows an 8 px base: 4, 8, 16, 24, 32, 48, 64, 96.
- Text measure is capped at 68 characters.

## Shape and interaction grammar

- The main app reads as one continuous control desk, not a grid of floating cards.
- Recessed wells contain totals and filters. Paper forms overlap the panel by 4 px.
- Buttons are rectangular Bakelite controls with 4 px corners, a dark lower edge, and a short pressed state.
- Status is shown with circular lamps, plain labels, and a checklist rail.
- Tables become labelled paper slips at 390 px; secondary columns collapse deliberately.
- Focus uses a 3 px dark teal ring with a 3 px cream offset.

## Motion

The signature motion is a **quarter-dial sweep**: when review progress changes, the dial hand rotates to the new position over 240 ms with a physical ease-out. New paper rows settle upward by 8 px over 180 ms. Nothing loops. Under `prefers-reduced-motion: reduce`, both changes are instant and route transitions use no movement.

## Asset plan and provenance

The hero illustration is an original generated still-life of a mid-century tax-record instrument, cropped as a wide editorial panel. It explains the product world while the live interface proves the function. The product also uses hand-authored SVG marks: the four-segment quarter dial, status lamps, receipt notch, and wordmark monogram.

### Prompt sheet

- Subject: compact 1960s British bookkeeping instrument, four-quarter dial, paper receipt slot, tidy ledger slips, no people.
- World: small tutor or landlord desk, practical government-office precision, not luxury technology.
- Materials: warm enamel, cream paper, dark Bakelite, brushed brass, subtle ink registration.
- Light/lens: soft north-window light, straight-on 50 mm editorial product photograph, gentle real shadows.
- Palette words: parchment cream, tobacco orange, deep bottle teal, charcoal ink, aged brass.
- Negative list: text, numbers, logos, brands, currency symbols, screens, gradients, neon, blue SaaS lighting, people, hands, watermarks.

Generated with the factory Azure image model (`factory-image`) on 2026-08-28. The exact prompt is stored beside each source image in `assets/src/`. Generated assets are original to Quarterly Ready. Hand-authored SVG assets are MIT-licensed with the application.

## Social image

The 1200×630 Open Graph image is composed from the generated instrument artwork with product-colour framing. It contains no essential UI instructions; all required meaning remains in HTML.
