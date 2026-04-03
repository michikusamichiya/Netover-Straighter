/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    screens: {
      sm: "640px",
      md: "768px",
      lg: "1024px",
      xl: "1280px",
      "2xl": "1536px",
    },
    extend: {
      colors: {
        netover_bg: "#111111",
        netover_text: "#D5D5D5",
        netover_blue: "#1A95FF",
        netover_green: "#8ACC00",
      }
    },
  },
  plugins: [],
}