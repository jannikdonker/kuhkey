import { defineConfig } from 'wxt';

// See https://wxt.dev/api/reference/wxt/interfaces/inlineconfig.html
export default defineConfig({
  manifest: {
    // so + add can fill in the website of the current tab (no extra install warning)
    permissions: ['activeTab'],
    // so the popup can load the rust/wasm code
    content_security_policy: {
      extension_pages: "script-src 'self' 'wasm-unsafe-eval'; object-src 'self';",
    },
  },
});
