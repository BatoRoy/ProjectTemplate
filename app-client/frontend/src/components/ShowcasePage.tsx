import { useState } from 'react'
import {
  Copy, Scissors, Clipboard, Trash2, Pencil, Download, MousePointerClick, Inbox, Bug,
} from 'lucide-react'
import { Button } from './Modal'
import { Card, Badge, Spinner, Skeleton, EmptyState } from './Feedback'
import { Switch, Checkbox, RadioGroup, Textarea } from './Form'
import { Tabs } from './Tabs'
import { Dropdown, Select } from './Dropdown'
import { Tooltip } from './Tooltip'
import { ConfirmDialog } from './ConfirmDialog'
import { ContextMenu } from './ContextMenu'
import { useContextMenu } from '../hooks/useContextMenu'
import type { ToastType } from '../types'

interface ShowcasePageProps {
  toast: (message: string, type?: ToastType) => void
}

// A live gallery of the shared component kit. Use it as a reference, then delete it
// (and its NAV entry in Sidebar.tsx) once your real pages exist.
export function ShowcasePage({ toast }: ShowcasePageProps) {
  const menu = useContextMenu()
  const [confirmOpen, setConfirmOpen] = useState(false)
  const [tab, setTab] = useState('one')
  const [sw, setSw] = useState(true)
  const [cb, setCb] = useState(false)
  const [radio, setRadio] = useState('a')
  const [fruit, setFruit] = useState('apple')

  // Trigger the ErrorBoundary on demand.
  const [boom, setBoom] = useState(false)
  if (boom) throw new Error('Demo error thrown from ShowcasePage')

  return (
    <div className="max-w-2xl mx-auto px-8 py-10 space-y-6">
      <div>
        <h2 className="text-2xl font-semibold text-app-text">Component kit</h2>
        <p className="text-app-subtext mt-1">Right-click menu, overlays, form controls, and feedback atoms.</p>
      </div>

      {/* Context menu */}
      <Card title="Context menu">
        <div
          onContextMenu={menu.onContextMenu}
          className="flex items-center justify-center gap-2 h-28 rounded-lg border-2 border-dashed
                     border-app-border text-app-muted text-sm select-none"
        >
          <MousePointerClick size={16} />
          Right-click anywhere in this box
        </div>
        {menu.isOpen && (
          <ContextMenu
            position={menu.position}
            onClose={menu.close}
            items={[
              { label: 'Copy', icon: Copy, shortcut: '⌘C', onClick: () => toast('Copied', 'success') },
              { label: 'Cut', icon: Scissors, shortcut: '⌘X', onClick: () => toast('Cut', 'info') },
              { label: 'Paste', icon: Clipboard, shortcut: '⌘V', disabled: true, onClick: () => {} },
              { type: 'separator' },
              { label: 'Rename', icon: Pencil, onClick: () => toast('Rename', 'info') },
              { label: 'Delete', icon: Trash2, danger: true, onClick: () => toast('Deleted', 'error') },
            ]}
          />
        )}
      </Card>

      {/* Overlays & menus */}
      <Card title="Overlays & menus">
        <div className="flex flex-wrap items-center gap-3">
          <Dropdown
            trigger="Actions"
            items={[
              { label: 'Edit', icon: Pencil, onClick: () => toast('Edit', 'info') },
              { label: 'Download', icon: Download, onClick: () => toast('Download', 'info') },
              { type: 'separator' },
              { label: 'Delete', icon: Trash2, danger: true, onClick: () => toast('Deleted', 'error') },
            ]}
          />
          <Select
            value={fruit}
            onChange={setFruit}
            options={[
              { value: 'apple', label: 'Apple' },
              { value: 'banana', label: 'Banana' },
              { value: 'cherry', label: 'Cherry' },
            ]}
          />
          <Tooltip content="I'm a tooltip">
            <Button variant="ghost">Hover me</Button>
          </Tooltip>
          <Button variant="danger" onClick={() => setConfirmOpen(true)}>Confirm dialog</Button>
        </div>
      </Card>

      {/* Form controls */}
      <Card title="Form controls">
        <div className="space-y-4">
          <Switch checked={sw} onChange={setSw} label="Enable notifications" />
          <Checkbox checked={cb} onChange={setCb} label="I agree to the terms" />
          <RadioGroup
            value={radio}
            onChange={setRadio}
            options={[
              { value: 'a', label: 'Option A' },
              { value: 'b', label: 'Option B' },
              { value: 'c', label: 'Option C' },
            ]}
          />
          <Textarea label="Notes" placeholder="Type something…" />
        </div>
      </Card>

      {/* Tabs */}
      <Card title="Tabs">
        <Tabs
          value={tab}
          onChange={setTab}
          tabs={[{ id: 'one', label: 'First' }, { id: 'two', label: 'Second' }, { id: 'three', label: 'Third' }]}
        />
        <p className="text-sm text-app-subtext mt-3">Active tab: <span className="mono-text">{tab}</span></p>
      </Card>

      {/* Feedback */}
      <Card title="Feedback">
        <div className="space-y-4">
          <div className="flex flex-wrap gap-2">
            <Badge tone="success">Success</Badge>
            <Badge tone="error">Error</Badge>
            <Badge tone="warning">Warning</Badge>
            <Badge tone="info">Info</Badge>
            <Badge>Neutral</Badge>
          </div>
          <div className="flex items-center gap-3 text-app-subtext">
            <Spinner /> <span className="text-sm">Loading…</span>
          </div>
          <div className="space-y-2">
            <Skeleton className="h-4 w-3/4" />
            <Skeleton className="h-4 w-1/2" />
          </div>
          <div className="border border-app-border rounded-lg">
            <EmptyState
              icon={Inbox}
              title="No items yet"
              subtitle="Create your first item to see it here."
              action={<Button onClick={() => toast('Created', 'success')}>New item</Button>}
            />
          </div>
        </div>
      </Card>

      {/* Error boundary */}
      <Card title="Error boundary">
        <div className="flex items-center justify-between">
          <p className="text-sm text-app-subtext">Throw a render error to see the fallback.</p>
          <Button variant="ghost" onClick={() => setBoom(true)}>
            <Bug size={14} /> Throw error
          </Button>
        </div>
      </Card>

      {confirmOpen && (
        <ConfirmDialog
          title="Delete item?"
          message="This action cannot be undone. The item will be permanently removed."
          confirmLabel="Delete"
          danger
          onConfirm={() => { setConfirmOpen(false); toast('Item deleted', 'error') }}
          onClose={() => setConfirmOpen(false)}
        />
      )}
    </div>
  )
}
