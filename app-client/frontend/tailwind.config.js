/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        app: {
          bg:          'rgb(var(--app-bg)           / <alpha-value>)',
          surface:     'rgb(var(--app-surface)      / <alpha-value>)',
          card:        'rgb(var(--app-card)         / <alpha-value>)',
          border:      'rgb(var(--app-border)       / <alpha-value>)',
          accent:      'rgb(var(--app-accent)       / <alpha-value>)',
          accentHover: 'rgb(var(--app-accent-hover) / <alpha-value>)',
          accentBright:'rgb(var(--app-accent-bright)/ <alpha-value>)',
          green:       'rgb(var(--app-green)        / <alpha-value>)',
          red:         'rgb(var(--app-red)          / <alpha-value>)',
          yellow:      'rgb(var(--app-yellow)       / <alpha-value>)',
          muted:       'rgb(var(--app-muted)        / <alpha-value>)',
          text:        'rgb(var(--app-text)         / <alpha-value>)',
          subtext:     'rgb(var(--app-subtext)      / <alpha-value>)',
        }
      },
      fontFamily: {
        mono: ['"JetBrains Mono"', '"Fira Code"', 'monospace'],
        sans: ['"Inter Variable"', 'Inter', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'sans-serif'],
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'slide-in': 'slideIn 0.2s ease-out',
        'fade-in': 'fadeIn 0.15s ease-out',
        'scale-in': 'scaleIn 0.12s ease-out',
      },
      keyframes: {
        slideIn: {
          '0%': { transform: 'translateX(-8px)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        scaleIn: {
          '0%': { transform: 'scale(0.96)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
      }
    },
  },
  plugins: [],
}
