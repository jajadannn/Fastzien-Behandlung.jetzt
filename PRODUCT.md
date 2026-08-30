# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary users are German-speaking adults (informal "du" address) dealing with chronic or recurring pain — back pain, migraine/headaches, neck & shoulder tension, joint/hip pain, stress-related tension, or post-injury/post-surgery rehabilitation — located in and around Sulgen, Schramberg, Dunningen, and Villingendorf (Baden-Württemberg, Germany). They discover the practice online, then book and manage appointments themselves through a customer portal (in-practice or home visit).

A second user role is the practitioner himself (Thilo Seifried) acting as admin: managing customers, appointments, payments, waitlist, site content (FAQ/reviews/settings), and calendar sync from an admin dashboard.

## Product Purpose

A public website plus booking/customer-portal system for a solo, certified Gantke®-method fascia-treatment practice. It lets prospective clients understand the method and book a first session with minimal friction, lets existing clients manage appointments/credits/waitlist themselves, and lets the practitioner run the whole operation (bookings, payments, calendar, communications) without office staff.

## Positioning

Gantke® is a specific, named, certified fascia-treatment method (not generic massage) delivered personally by one certified practitioner — not an anonymous clinic or franchise. The claimed mechanism: it addresses the underlying cause of chronic pain rather than only the symptoms, with effects described as noticeable from the first session.

## Operating Context

- Sessions are ~90 minutes, either in-practice (Sulgauer Straße 24, 78713 Sulgen) or as a home visit (+15 € travel surcharge) within the region.
- Opening hours: Mo–Fr 16:00–22:00, Sa 9:00–19:00 (configurable in admin settings).
- Pricing: single session (~195 €) or a 10-session credit pack (~169,90 €/session); both configurable in admin settings, not hardcoded truths.
- Cancellation is allowed up to 24 hours before the appointment via the customer portal.
- Calendar sync via Nextcloud/CalDAV (OAuth2 or app-password), plus a per-customer ICS subscription link.
- New accounts must verify their e-mail before full portal access (booking, waitlist, credits) unlocks; profile completion (name/address) is required before booking.
- When the practitioner cancels, he can propose alternative time slots directly in the cancellation e-mail/portal flow.
- Transactional e-mail is a first-class, technically constrained surface: must render correctly in Outlook desktop and other clients with poor CSS support (inline styles, table-based layout only, no external stylesheets/fonts/SVG, gradients need a solid-color fallback).

## Capabilities and Constraints

- Stack: Rust (Actix-Web) backend, Tera server-rendered templates, SQLite (rusqlite), SMTP e-mail via `lettre`.
- Undecided/not product truth: exact prices and hours are admin-configurable, not fixed facts to hardcode into new copy.

## Brand Commitments

- Legal/trading name: "Faszienbehandlung Thilo Seifried", method name "Gantke® Faszienbehandlung" — the ® must always be preserved when the method name is written.
- Compliance-critical: the practice is explicitly **not** Heilkunde under the German Heilpraktikergesetz and does not replace medical diagnosis or treatment (stated in the Impressum). Copy must never imply medical diagnosis, cure, or treatment claims.
- Voice: informal "du", warm, calm, professional — never salesy or hyped.
- No emoji anywhere, on any surface (web, portal, admin, e-mail) — established and enforced this project across a full redesign pass. Decorative glyphs use a hand-built monochrome stroke-icon system (inline SVG sprite) on the web; e-mail cannot use that sprite (see Operating Context) and instead stays icon-free with clean typographic hierarchy.
- Existing visual identity (web): sky blue (#70AECD → #4d93b8) and plum (#964279 → #7a3661) as the two brand colors, off-white background (#f7fbfd), Cormorant Garamond for display/serif headings, DM Sans for body text, fully rounded ("pill") buttons, generous whitespace, calm/subtle motion (no flashy animation).
- Logo asset at `/Logo.png`.

## Evidence on Hand

- Customer reviews stored in the database are real client quotes, confirmed earlier this session — never fabricate additional testimonials, review counts, or star ratings.
- Real contact details exist in admin settings (address, phone, e-mail) and must be sourced from there / kept consistent with them, not invented.
- Two verified domain-typo bugs already found and partly fixed this session: some legacy links/canonical tags use the wrong hyphenated domain `faszien-behandlung.jetzt` instead of the real `faszienbehandlung.jetzt` — treat any hyphenated instance encountered as a bug, not a fact to preserve.

## Product Principles

1. One coherent, calm, professional identity across every surface — web, customer portal, admin, and e-mail — with zero colorful emoji anywhere.
2. Personal and small-scale by design: a single named, certified practitioner, real reviews, no invented scale or fabricated social proof.
3. Compliance-aware copy: never cross from wellness/bodywork language into medical diagnosis or cure claims.
4. Low-friction booking is the core value; every surface should make discovering, booking, and managing an appointment as frictionless as possible.
5. E-mail is a constrained, high-stakes surface (transactional, must render correctly everywhere) — polish comes from typography, hierarchy, and restraint, not from techniques that break in Outlook.

## Accessibility & Inclusion

No specific accessibility standard has been mandated by the user (confirmed earlier this session); general web best practice applies (semantic HTML, sufficient color contrast, keyboard-operable controls).
