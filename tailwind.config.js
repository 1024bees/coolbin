/** @type {import('tailwindcss').Config} */


const colors = require('tailwindcss/colors')

module.exports = {
  mode: 'jit',
    content: [
    './templates/**/*.{html,js}',
  ],
  theme: {
    extend: {
      colors: {
        sky: colors.sky,
        cyan: colors.cyan,
      },
    },
  },
  variants: {},
  plugins: [],
}

