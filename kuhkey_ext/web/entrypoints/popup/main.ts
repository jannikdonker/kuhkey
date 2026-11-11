import './style.css';
import init, { greet } from '../../wasm/kuhkey_ext.js';

init().then(() => {
  document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
    <h1>KuhKey</h1>
    <p>${greet('Yoshi')}</p>
  `;
});
