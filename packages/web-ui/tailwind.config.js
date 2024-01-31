import plugin from "tailwindcss/plugin";
import tailwindScrollbar from "tailwind-scrollbar";

/** @type {import('tailwindcss').Config} */
export default {
  content: ["src/**/*.tsx", "index.html"],
  theme: {
    extend: {},
  },
  plugins: [
    tailwindScrollbar({ nocompatible: true }),
    plugin(({ addVariant }) => {
      addVariant("aria-current", `&[aria-current]`);
    }),
  ],
};
