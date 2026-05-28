// Entry point that registers the <bloop-audio-vm> custom element.
// Import this module once to make the element available in HTML.
import { BloopAudioElement } from './components/BloopAudioElement.js';

if (!customElements.get('bloop-audio-vm')) {
  customElements.define('bloop-audio-vm', BloopAudioElement);
}

export { BloopAudioElement };
