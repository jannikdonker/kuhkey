import './style.css';
import init, { greet } from '../../wasm/kuhkey_ext.js';
import cowurl from '../../assets/cow.jpeg'

init().then(() => {
  document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
    <img src="${cowurl}" width="120" alt="cow">
    <p>${greet('Yoshi')}</p>
  `;
});
