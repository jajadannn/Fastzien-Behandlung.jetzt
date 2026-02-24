const titelname = ["Faszienbehandlung - Thilo Seifried", "Faszienbehandlung Buchen"];
let i = 0;

setInterval(() => {
    document.getElementById("titel").innerHTML = titelname[i];
    i = (i + 1) % titelname.length;
}, 10000);

const appointmentModal = document.getElementById("appointment-modal");
const appointmentButtons = document.querySelectorAll(".open-appointment");
const closeModalButton = document.querySelector(".modal-close");

const openAppointmentModal = () => {
    appointmentModal.classList.add("is-visible");
    appointmentModal.setAttribute("aria-hidden", "false");
    document.body.classList.add("modal-open");
};

const closeAppointmentModal = () => {
    appointmentModal.classList.remove("is-visible");
    appointmentModal.setAttribute("aria-hidden", "true");
    document.body.classList.remove("modal-open");
};

appointmentButtons.forEach((button) => {
    button.addEventListener("click", openAppointmentModal);
});

closeModalButton.addEventListener("click", closeAppointmentModal);

appointmentModal.addEventListener("click", (event) => {
    if (event.target === appointmentModal) {
        closeAppointmentModal();
    }
});

document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && appointmentModal.classList.contains("is-visible")) {
        closeAppointmentModal();
    }
});
