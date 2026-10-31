/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        studio: {
          bg: '#0b0e13',
          panel: '#12161d',
          border: '#232b36',
          preview: '#16a34a',
          program: '#dc2626',
        },
      },
    },
  },
  plugins: [],
};
