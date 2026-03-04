const NEXTCLOUD_APPOINTMENT_URL = "https://cloud.sdlv.de/apps/appointments/embed/7GAGc8Ctarv8Y7Gt/form";

document.addEventListener("DOMContentLoaded", function () {
    // ── Modal ─────────────────────────────────────────────────
    const modal       = document.getElementById("appointment-modal");
    const iframe      = document.getElementById("appointment-iframe");
    const closeButton = document.getElementById("appointment-close");
    const triggers    = document.querySelectorAll(".book-trigger");

    function openAppointmentModal() {
        iframe.src = NEXTCLOUD_APPOINTMENT_URL;
        modal.classList.add("is-open");
        modal.setAttribute("aria-hidden", "false");
        document.body.classList.add("modal-open");
    }

    function closeAppointmentModal() {
        modal.classList.remove("is-open");
        modal.setAttribute("aria-hidden", "true");
        iframe.removeAttribute("src");
        document.body.classList.remove("modal-open");
    }

    triggers.forEach(t => t.addEventListener("click", openAppointmentModal));
    closeButton.addEventListener("click", closeAppointmentModal);
    modal.addEventListener("click", e => { if (e.target === modal) closeAppointmentModal(); });
    document.addEventListener("keydown", e => {
        if (e.key === "Escape" && modal.classList.contains("is-open")) closeAppointmentModal();
    });

    // ── Scroll-reveal ─────────────────────────────────────────
    const sections = document.querySelectorAll(".content");
    if ("IntersectionObserver" in window) {
        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    entry.target.style.opacity = "1";
                    entry.target.style.transform = "translateY(0)";
                    observer.unobserve(entry.target);
                }
            });
        }, { threshold: 0.08 });

        sections.forEach(s => {
            s.style.opacity = "0";
            s.style.transform = "translateY(24px)";
            s.style.transition = "opacity 0.55s ease, transform 0.55s ease";
            observer.observe(s);
        });
    }

    // ── Sticky nav highlight ──────────────────────────────────
    const navLinks = document.querySelectorAll(".nav a[href^='#']");
    if (navLinks.length) {
        const sectionIds = Array.from(navLinks).map(a => a.getAttribute("href").slice(1));
        const sectionEls = sectionIds.map(id => document.getElementById(id)).filter(Boolean);

        function updateActiveLink() {
            let current = "";
            sectionEls.forEach(sec => {
                if (window.scrollY >= sec.offsetTop - 120) current = sec.id;
            });
            navLinks.forEach(a => {
                a.removeAttribute("aria-current");
                if (a.getAttribute("href") === "#" + current) a.setAttribute("aria-current", "page");
            });
        }

        window.addEventListener("scroll", updateActiveLink, { passive: true });
        updateActiveLink();
    }
});
