# Design preservation finish check

Scope: ordinary refinement and extension of the incumbent system, not a new visual world or an approved identity replacement. This is a source-evidence check; it does not substitute for the final reviewer's browser or contrast verdict on the four fixes.

## Authorities and evidence

Read the shipped `reference/document.md` and `agents/impeccable_documenter.toml`. The documenter role explicitly directs ordinary extensions to preserve the incumbent system and report drift without repairing it unasked. `PRODUCT.md` remains confirmed product truth. Implementation authority remains the existing templates, stylesheets, and identity assets.

Checked `templates/landing.html`, `static/css/main.css` and its Git diff, `templates/customer/book_appointment.html`, the relevant additions to `static/css/customer.css`, and `templates/marketing_topic.html`. Existing screenshot artifacts are recorded at `.impeccable/review/landing-desktop-hero.jpg`, `landing-mobile.jpg`, `booking-mobile.jpg`, and `topic-mobile.jpg`; no context, detector, or browser QA was rerun in this pass.

## Five-line system summary

1. Palette: incumbent sky (`#70AECD`, `#4d93b8`) and plum (`#964279`, `#7a3661`), with off-white (`#f7fbfd`) and blue-gray text, remain in the root tokens; the CSS diff does not replace them.
2. Type: self-hosted Cormorant Garamond display/headings and DM Sans body/actions remain; the topic heading scale uses that same pairing rather than introducing another family.
3. Form: rounded cards and pill actions remain; the booking and topic surfaces reuse the existing radii, brand variables, and buttons.
4. Icon and motion commitments: existing monochrome SVG stroke icons and reduced-motion handling remain; the hero shade/glow animations are removed while existing image/reveal behavior is retained.
5. Durable rules: preserve the confirmed identity, use the established type/shape/icon vocabulary, and keep refinements calm and legible. These are existing product commitments, not newly invented named doctrines or a Creative North Star.

## Narrow intentional refinements

- Landing copy clarifies wellness/bodywork, the named practitioner, and the existing session offer. Pricing and duration still come from settings. `PRODUCT.md` keeps new packages, payment methods, company offers, and follow-up services in draft status.
- Real portrait derivatives appear in the hero practitioner detail and about section. The existing `Start.avif` remains the hero photograph. Hero shading is steadier and darker for readability rather than establishing another palette.
- The added `--sky-ink` (`#2f6583`) supplies a deeper blue for legibility. The closing CTA uses this blue with existing plum-dark and white text. Trust subtitles end at `.875rem` (14px under the default 16px root), and footer links receive brighter text and underline offset. These are scoped readability changes; the final reviewer determines their rendered adequacy.
- Mobile menu sizing, two-column trust items, booking field stacking, form spacing, calendar controls, and visible focus outlines refine existing interactions. Topic pages share `main.css`, the logo/navigation, serif headings, plum links, pill booking action, and footer.

Final narrow source recheck: the closing CTA includes an explicit `.cta-section>.cta-inner>p` white-text rule alongside opaque contact/supporting text; the footer container link rule covers topic legal links; the final desktop and mobile trust subtitle declarations both resolve to `.875rem`. Landing geo metadata now names Schramberg-Sulgen, Baden-Württemberg and carries `48.2157;8.5033` (with the corresponding ICBM value). These four corrections preserve the identity and product location; no additional system change is introduced. Rendered verification and the final verdict remain the finish reviewer's responsibility.

## Asset provenance and documentation boundary

`Logo.avif.json` and `Start.avif.json` explicitly identify the pre-existing project logo and hero image used without visual alteration. The provided static-images provenance scan reported three rasters and zero missing records; the directory contains the PNG logo fallback and two WebP portrait derivatives with sidecars. The supplied asset audit also records COM origin metadata for `og-hero.jpg`. This pass did not regenerate imagery or rerun that scanner. The PNG email fallback preserves the existing transactional-email constraint.

Neither `DESIGN.md` nor `.impeccable/design.json` exists. This is an intentional documentation gap for this ordinary extension, not authorization to invent a world, qualitative design language, token sidecar, or approved identity change. Existing product and source authorities are preserved. This report is the only file written by this pass.

Not canonized or repaired: no additional defect or unrelated drift is promoted into design-system rules; the final reviewer's four-fix verdict remains pending, and this bounded evidence check cannot establish a site-wide craft or accessibility pass.
