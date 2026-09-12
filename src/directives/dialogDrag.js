const INTERACTIVE = "button, input, textarea, select, a, .v-btn, [role='button'], label";

export default {
  mounted(el) {
    let content = null;
    let startX = 0;
    let startY = 0;
    let baseX = 0;
    let baseY = 0;
    let dragging = false;

    function currentOffset() {
      return {
        x: Number(content?.dataset.dragX || 0),
        y: Number(content?.dataset.dragY || 0),
      };
    }

    function setOffset(x, y) {
      if (!content) return;
      content.dataset.dragX = String(x);
      content.dataset.dragY = String(y);
      content.style.transform = `translate(${x}px, ${y}px)`;
      content.style.transition = "none";
    }

    function onDown(e) {
      if (e.button !== 0) return;
      if (e.target.closest(INTERACTIVE)) return;
      content = el.closest(".v-overlay__content") || el.closest(".v-card");
      if (!content) return;
      dragging = true;
      startX = e.clientX;
      startY = e.clientY;
      ({ x: baseX, y: baseY } = currentOffset());
      document.body.classList.add("modal-no-select");
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
      if (e.cancelable) e.preventDefault();
    }

    function onMove(e) {
      if (!dragging || !content) return;
      setOffset(baseX + (e.clientX - startX), baseY + (e.clientY - startY));
    }

    function onUp() {
      if (!dragging) return;
      dragging = false;
      document.body.classList.remove("modal-no-select");
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
    }

    el.addEventListener("mousedown", onDown);
    el._dialogDragCleanup = () => {
      el.removeEventListener("mousedown", onDown);
      onUp();
    };
  },
  unmounted(el) {
    if (el._dialogDragCleanup) el._dialogDragCleanup();
  },
};