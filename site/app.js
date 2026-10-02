// Enables the copy buttons. The page is fully usable without JavaScript.
document.documentElement.classList.add("js");

document.addEventListener("click", async (event) => {
  const button = event.target.closest("button[data-copy]");
  if (!button) return;
  const code = button.parentElement.querySelector("code");
  if (!code) return;
  const original = button.textContent;
  try {
    await navigator.clipboard.writeText(code.textContent.trim());
    button.textContent = "Copiado";
  } catch {
    // Clipboard API can be blocked (insecure context, permissions): select instead.
    const range = document.createRange();
    range.selectNodeContents(code);
    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
    button.textContent = "Selecionado";
  }
  setTimeout(() => {
    button.textContent = original;
  }, 1600);
});
