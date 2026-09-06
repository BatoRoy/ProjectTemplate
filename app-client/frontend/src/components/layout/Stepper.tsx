import type { ReactNode } from 'react'
import { Check } from 'lucide-react'
import clsx from 'clsx'

interface Step {
  id: string
  label: ReactNode
  description?: ReactNode
}

interface StepperProps {
  steps: Step[]
  /** Index of the current (in-progress) step. Earlier steps render as complete. */
  current: number
  orientation?: 'horizontal' | 'vertical'
  onStepClick?: (index: number) => void
  className?: string
}

// Step indicator for wizards/flows. Steps before `current` are complete (check),
// the current step is accented, later steps are muted.
export function Stepper({ steps, current, orientation = 'horizontal', onStepClick, className }: StepperProps) {
  const horizontal = orientation === 'horizontal'

  return (
    <div className={clsx('flex', horizontal ? 'items-start' : 'flex-col', className)}>
      {steps.map((step, i) => {
        const complete = i < current
        const active = i === current
        const last = i === steps.length - 1
        return (
          <div key={step.id} className={clsx('flex', horizontal ? 'flex-1 items-center' : 'gap-3')}>
            <div className={clsx('flex', horizontal ? 'flex-col items-center text-center' : 'flex-col items-center')}>
              <button
                disabled={!onStepClick}
                onClick={() => onStepClick?.(i)}
                className={clsx(
                  'w-8 h-8 rounded-full flex items-center justify-center text-xs font-semibold border-2 transition-colors flex-shrink-0',
                  complete && 'bg-app-accent border-app-accent text-app-accentInk',
                  active && 'border-app-accent text-app-accentBright',
                  !complete && !active && 'border-app-border text-app-muted',
                  onStepClick && 'cursor-pointer',
                )}
              >
                {complete ? <Check size={15} /> : i + 1}
              </button>
              {!horizontal && !last && <div className="w-0.5 flex-1 my-1 bg-app-border min-h-[1.5rem]" />}
            </div>

            <div className={clsx(horizontal ? 'mt-2' : 'pb-6 pt-1', horizontal && 'px-1')}>
              <div className={clsx('text-sm font-medium', active || complete ? 'text-app-text' : 'text-app-muted')}>{step.label}</div>
              {step.description && <div className="text-xs text-app-muted mt-0.5">{step.description}</div>}
            </div>

            {horizontal && !last && <div className={clsx('h-0.5 flex-1 mx-2 mt-4', complete ? 'bg-app-accent' : 'bg-app-border')} />}
          </div>
        )
      })}
    </div>
  )
}
