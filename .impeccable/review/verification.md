# Verification – 8 October 2026

Scope: the supplied marketing guide plus the user's Impeccable refinement request. Existing offers remain; proposed packages, additional payment methods and follow-up services are drafts.

- `cargo build`: passed.
- `cargo test`: 5 passed, 0 failed.
- `node --check` for booking.js and main.js: passed.
- Local isolated-database smoke checks: public pages and valid structured data; no private file/settings exposure; explicit email-link confirmation; atomic appointment/payment/source creation; token replay/expiry and booking conflicts; existing credentials/profile/credits preserved; visible SMTP failure; unsafe guest text escaped in invoices.
- Browser checks at 390, 768, 1254 and 1440px: inspected applicable page families; no horizontal overflow in checked layouts; mobile menu opens/closes including Escape; house-visit address fields appear and become required; no app console errors observed.
- One Impeccable detector run is saved in detector.json. Its static contrast warnings were checked against final cascade/rendered evidence; it was not rerun.
- Independent full review requested four material fixes. Its final **ship** verdict scores all four resolved and covers that fix list, not a blanket accessibility certification. See finish-review.md and finish-verdict.md.
- Documentation check recorded incumbent identity preservation in design-preservation.md.

Performance evidence is asset size, not a Lighthouse score or field Core Web Vitals measurement:

| Asset | Original | Delivered variant |
|---|---:|---:|
| Portrait | 14,755,009 bytes | 30,452 / 60,518 bytes WebP |
| Web logo | 1,410,032 bytes PNG | 37,315 bytes existing AVIF |
| PNG logo fallback | 1,410,032 bytes | 15,155 bytes |

The small portrait is reused in the hero; responsive sources serve the about photo. Hero AVIF remains ~190KB. Social preview now exists as a 1200×630 JPEG. Newly shipped rasters have origin metadata/sidecars; static/images scan reported 3 rasters, 0 missing provenance records.

The preview uses only synthetic records in the explicitly named temporary database. SMTP is intentionally disabled in the preview; a genuine production email and calendar check is still required before deployment. No external campaign, profile change, outreach message or paid advertising was executed. Production database content migration takes effect when the updated server is deployed and started, not during this local preview.
