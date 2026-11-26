import DefaultTheme from 'vitepress/theme';
import Layout from './Layout.vue';
import HomeThemes from './HomeThemes.vue';
import HomeInstall from './HomeInstall.vue';
import './style.css';

/** Visible page screenshots: images in the docs or zoomable home frames, not in links. */
const screenshots = () =>
  [...document.querySelectorAll('.vp-doc img, .mc-zoomable img')].filter(
    (image) => !image.closest('a') && image.offsetParent !== null,
  );

let dialog;
let images = [];
let index = 0;

/** Shows the screenshot at `at` in the dialog, with its alt text and place. */
function show(at) {
  index = (at + images.length) % images.length;
  const image = images[index];
  const full = dialog.querySelector('img');
  full.src = image.currentSrc || image.src;
  full.alt = image.alt;
  dialog.querySelector('figcaption').textContent =
    images.length > 1 ? `${image.alt} (${index + 1} of ${images.length})` : image.alt;
}

/**
 * A dialog for one screenshot at a time. A click on the image or around it,
 * or Escape, closes it. The arrows, arrow keys or a swipe go to the next or
 * previous one on the page.
 */
function build() {
  dialog = document.createElement('dialog');
  dialog.className = 'zoom';
  dialog.innerHTML = `
    <figure tabindex="-1" autofocus><img alt="" draggable="false"><figcaption></figcaption></figure>
    <button type="button" class="zoom-step" data-step="-1" aria-label="Previous image"><svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 6l-6 6l6 6"/></svg></button>
    <button type="button" class="zoom-step" data-step="1" aria-label="Next image"><svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M9 6l6 6l-6 6"/></svg></button>`;
  let swipeFrom = null;
  let swiped = false;
  dialog.addEventListener('pointerdown', (event) => {
    swipeFrom = event.clientX;
  });
  dialog.addEventListener('pointerup', (event) => {
    if (swipeFrom !== null && images.length > 1 && Math.abs(event.clientX - swipeFrom) > 50) {
      swiped = true;
      show(index + (event.clientX < swipeFrom ? 1 : -1));
    }
    swipeFrom = null;
  });
  dialog.addEventListener('click', (event) => {
    const step = event.target.closest('[data-step]');
    if (step) show(index + Number(step.dataset.step));
    else if (!swiped) dialog.close();
    swiped = false;
  });
  dialog.addEventListener('keydown', (event) => {
    if (images.length < 2) return;
    if (event.key === 'ArrowRight') show(index + 1);
    else if (event.key === 'ArrowLeft') show(index - 1);
  });
  dialog.addEventListener('close', () => document.documentElement.classList.remove('zoomed'));
  document.body.append(dialog);
}

function zoom(event) {
  const image = event.target;
  if (
    !(image instanceof HTMLImageElement) ||
    (!image.closest('.vp-doc') && !image.closest('.mc-zoomable')) ||
    image.closest('a')
  ) {
    return;
  }
  if (!dialog) build();
  images = screenshots();
  dialog.classList.toggle('single', images.length < 2);
  show(images.indexOf(image));
  document.documentElement.classList.add('zoomed');
  dialog.showModal();
}

export default {
  extends: DefaultTheme,
  Layout,
  enhanceApp({ app }) {
    app.component('HomeThemes', HomeThemes);
    app.component('HomeInstall', HomeInstall);
    if (typeof document !== 'undefined') document.addEventListener('click', zoom);
  },
};
