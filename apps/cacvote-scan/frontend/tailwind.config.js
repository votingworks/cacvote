/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: 'all',
  content: [
    './src/**/*.{rs,html,css}',
    './dist/**/*.html',
    '../../../libs/ui-rs/src/**/*.{rs,html,css}',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
