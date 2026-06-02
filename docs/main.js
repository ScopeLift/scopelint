document.addEventListener("DOMContentLoaded", () => {
  const root = document.documentElement;
  const toggle = document.getElementById("theme-toggle");
  const label = toggle?.querySelector(".theme-toggle-label");

  const getPreferredTheme = () => {
    const stored = window.localStorage.getItem("scopelint-theme");
    if (stored === "light" || stored === "dark") return stored;
    return window.matchMedia &&
      window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  };

  const applyTheme = (theme) => {
    root.setAttribute("data-theme", theme);
    if (label) {
      label.textContent = theme === "dark" ? "Dark" : "Light";
    }
  };

  const initialTheme = getPreferredTheme();
  applyTheme(initialTheme);

  if (toggle) {
    toggle.addEventListener("click", () => {
      const current = root.getAttribute("data-theme") === "light" ? "light" : "dark";
      const next = current === "light" ? "dark" : "light";
      window.localStorage.setItem("scopelint-theme", next);
      applyTheme(next);
    });
  }

  const links = document.querySelectorAll('a[href^="#"]:not([href="#"])');

  for (const link of links) {
    link.addEventListener("click", (event) => {
      const href = link.getAttribute("href");
      if (!href) return;

      const target = document.querySelector(href);
      if (!target) return;

      event.preventDefault();
      const headerOffset = 64;
      const rect = target.getBoundingClientRect();
      const offsetTop = rect.top + window.scrollY - headerOffset;

      window.scrollTo({
        top: offsetTop,
        behavior: "smooth",
      });
    });
  }
});

