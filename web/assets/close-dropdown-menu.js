document.addEventListener("click", (event) => {
    const item = event.target.closest("[data-dropdown-close]");

    if (item) {
        item.closest("details[data-dropdown-menu]")?.removeAttribute("open");
        return;
    }

    document.querySelectorAll("details[data-dropdown-menu][open]").forEach((menu) => {
        if (!menu.contains(event.target)) {
            menu.removeAttribute("open");
        }
    });
});
