import adapter from "@sveltejs/adapter-static";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    // See https://svelte.dev/docs/kit/adapters for more information about adapters.
    adapter: adapter(),
  },
};

export default config;
