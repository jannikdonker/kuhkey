import './style.css';
import initWasm from '../../wasm/kuhkey_ext.js';

// loads the rust/wasm module, its start fn then mounts the dioxus ui into #main
initWasm().catch((error) => console.error('Could not load the WASM module:', error));
