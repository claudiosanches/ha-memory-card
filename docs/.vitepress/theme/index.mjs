import DefaultTheme from 'vitepress/theme';
import './style.css';

/** Opens a screenshot full size; a click anywhere or Escape closes it. */
function zoom(event) {
  const image = event.target;
  if (!(image instanceof HTMLImageElement) || !image.closest('.vp-doc') || image.closest('a')) return;
  let dialog = document.querySelector('dialog.zoom');
  if (!dialog) {
    dialog = document.createElement('dialog');
    dialog.className = 'zoom';
    dialog.append(document.createElement('img'));
    dialog.addEventListener('click', () => dialog.close());
    dialog.addEventListener('close', () => document.documentElement.classList.remove('zoomed'));
    document.body.append(dialog);
  }
  const full = dialog.querySelector('img');
  full.src = image.currentSrc || image.src;
  full.alt = image.alt;
  document.documentElement.classList.add('zoomed');
  dialog.showModal();
}

export default {
  extends: DefaultTheme,
  enhanceApp() {
    if (typeof document !== 'undefined') document.addEventListener('click', zoom);
  },
};
