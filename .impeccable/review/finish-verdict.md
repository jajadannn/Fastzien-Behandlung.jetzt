## verdict

1. Resolved — topic legal footer links are visibly light and underlined in fix-mobile-footer.jpg; .footer>.container a and supplied computed evidence confirm rgba(255,255,255,.92) against #12202a.
2. Resolved — closing CTA explanatory copy, phone, email and address are visibly readable in fix-mobile-cta.jpg and fix-desktop-cta.jpg. The deeper existing sky/plum gradient and opaque white matching selectors resolve the contrast defect, including the higher-specificity direct-child paragraphs. fix-dom-evidence.json predates the final paragraph override; current source confirms opaque white.
3. Resolved — trust descriptions are legible without clipping in fix-mobile-trust.jpg. Current default/mobile cascade sets .trust-sub to .875rem (14px), with 1.5 line-height and 15.2px labels; the supplied computed evidence agrees.
4. Resolved — templates/landing.html meta[name="geo.placename"] now contains Schramberg-Sulgen, Baden-Württemberg; supplied DOM evidence agrees.

## remaining

Clear. No fix-batch regressions observed in the supplied captures. This ship verdict covers the four scored fixes, not a new review of the whole surface.

disposition: ship
