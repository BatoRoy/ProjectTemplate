import { Component } from 'react'
import type { ReactNode, ErrorInfo } from 'react'
import { AlertTriangle } from 'lucide-react'

interface ErrorBoundaryProps {
  children: ReactNode
  /** Optional custom fallback; receives the error and a reset callback. */
  fallback?: (error: Error, reset: () => void) => ReactNode
}

interface ErrorBoundaryState {
  error: Error | null
}

// Catches render-time crashes in the subtree and shows a themed fallback instead of
// a blank screen. Wrap the app (or any risky subtree) in App.tsx.
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('ErrorBoundary caught:', error, info)
  }

  reset = () => this.setState({ error: null })

  render() {
    const { error } = this.state
    if (!error) return this.props.children
    if (this.props.fallback) return this.props.fallback(error, this.reset)

    return (
      <div className="flex h-screen items-center justify-center p-6 text-app-text">
        <div className="bg-app-card border border-app-border rounded-xl shadow-2xl p-6 max-w-md w-full text-center">
          <div className="mx-auto mb-4 w-12 h-12 rounded-full bg-app-red/15 flex items-center justify-center">
            <AlertTriangle size={22} className="text-app-red" />
          </div>
          <h2 className="text-sm font-semibold">Something went wrong</h2>
          <p className="text-xs text-app-muted mt-1.5 mono-text break-words">{error.message}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-5 px-4 py-2.5 rounded-lg bg-app-accent text-app-accentInk text-sm font-medium hover:bg-app-accentHover transition-colors"
          >
            Reload
          </button>
        </div>
      </div>
    )
  }
}
