window.addEventListener("DOMContentLoaded", () => {
  const connectBtn = document.getElementById("connect-btn");
  const goBtn = document.getElementById("go-btn");
  const urlInput = document.getElementById("url-input");

  if (connectBtn) {
    connectBtn.addEventListener("click", () => {
      window.location.href = "https://antigravity.google.com";
    });
  }

  if (goBtn && urlInput) {
    goBtn.addEventListener("click", () => {
      let target = urlInput.value.trim();
      if (!target) return;
      if (!/^https?:\/\//i.test(target)) {
        target = "https://" + target;
      }
      window.location.href = target;
    });

    urlInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        goBtn.click();
      }
    });
  }
});
