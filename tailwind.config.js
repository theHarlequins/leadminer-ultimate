/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f0f9ff',
          100: '#e0f2fe',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
        },
      },
      boxShadow: {
        'neumorphism': '8px 8px 16px #d1d9e6, -8px -8px 16px #ffffff',
        'neumorphism-inset': 'inset 4px 4px 8px #d1d9e6, inset -4px -4px 8px #ffffff',
      },
    },
  },
  plugins: [],
}