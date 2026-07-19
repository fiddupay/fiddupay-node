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
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          accent: '#10b981', // Neon green success
          gold: '#f59e0b', // Accent warning
        },
        background: '#0b0f19', // Premium deep dark slate
        surface: '#151c2c', // Card surface
        border: 'rgba(255, 255, 255, 0.08)',
      },
      fontFamily: {
        sans: ['Bricolage Grotesque', 'Inter', 'sans-serif'],
      },
      boxShadow: {
        glow: '0 0 15px rgba(59, 130, 246, 0.15)',
        glowGreen: '0 0 15px rgba(16, 185, 129, 0.15)',
      }
    },
  },
  plugins: [],
}
