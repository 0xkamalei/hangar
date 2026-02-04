/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: '#0A0A0A',
        card: '#121212',
        accent: {
          primary: '#FF6B35', // Orange-Red
          secondary: '#FFA500', // Gold-Orange
          tech: '#00FF88', // Tech Green
        },
        neutral: {
          main: '#E0E0E0',
          muted: '#888888',
        }
      },
      fontFamily: {
        sans: ['PingFang SC', 'Microsoft YaHei', 'Helvetica Neue', 'sans-serif'],
      },
      backgroundImage: {
        'hero-gradient': 'linear-gradient(135deg, #0A0A0A 0%, #1A0F0F 50%, #0A0A0A 100%)',
      },
      animation: {
        'ken-burns': 'kenburns 20s ease-in-out infinite alternate',
        'fade-in-up': 'fadeInUp 1.2s ease-out forwards',
        'glow': 'glow 2s ease-in-out infinite alternate',
      },
      keyframes: {
        kenburns: {
          '0%': { transform: 'scale(1)' },
          '100%': { transform: 'scale(1.05)' },
        },
        fadeInUp: {
          '0%': { opacity: '0', transform: 'translateY(20px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        glow: {
          '0%': { boxShadow: '0 0 5px rgba(255, 107, 53, 0.2)' },
          '100%': { boxShadow: '0 0 20px rgba(255, 107, 53, 0.6)' },
        }
      }
    },
  },
  plugins: [],
}
