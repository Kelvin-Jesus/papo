// mdBook renders ```mermaid fences as plain code blocks. Swap them for Mermaid
// diagrams, loading the library from the CDN only on pages that have one, and
// follow the book's light/dark theme.
(() => {
  const blocks = document.querySelectorAll("code.language-mermaid");
  if (!blocks.length) return;
  const darkThemes = ["navy", "coal", "ayu"];
  const isDark = () => darkThemes.some((t) => document.documentElement.classList.contains(t));
  const sources = [];
  blocks.forEach((code) => {
    const pre = code.closest("pre") || code;
    const host = document.createElement("pre");
    host.className = "mermaid";
    host.textContent = code.textContent;
    pre.replaceWith(host);
    sources.push([host, host.textContent]);
  });
  import("https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.esm.min.mjs")
    .then(({ default: mermaid }) => {
      const render = () => {
        sources.forEach(([host, src]) => {
          host.removeAttribute("data-processed");
          host.textContent = src;
        });
        mermaid.initialize({ startOnLoad: false, theme: isDark() ? "dark" : "default", securityLevel: "strict" });
        return mermaid.run({ nodes: sources.map(([host]) => host) });
      };
      render();
      // Re-render when the reader switches the book theme.
      new MutationObserver(render).observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    })
    .catch((err) => console.warn("mermaid failed to load; diagrams stay as text", err));
})();
