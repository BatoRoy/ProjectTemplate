import { useState, useRef } from 'react'
import {
  Copy, Scissors, Clipboard, Trash2, Pencil, Search, Mail, Plus, Settings, Home, FileText,
} from 'lucide-react'
import type { Node, Edge } from '@xyflow/react'
import {
  Button, Card, Badge, Spinner, Skeleton, EmptyState, Tabs,
  ContextMenu, Dropdown, Select, Tooltip, ConfirmDialog, Drawer, Popover, CommandPalette,
  Switch, Checkbox, RadioGroup, TextField, NumberInput, TextArea, TagsInput, OtpInput,
  MaskedInput, CurrencyInput, Combobox, MultiSelect, Slider, RangeSlider, ColorPicker, FileDropzone, TimeInput,
  Accordion, Stepper, Breadcrumbs, Avatar, AvatarGroup, Progress, CircularProgress, SegmentedControl, Alert,
  Stack, HStack, Grid as GridBox, Container, Center, AspectRatio, Divider, AutoGrid, Masonry, AppShell,
  PanelGroup, Panel, EditorTabs, Dashboard,
  Calendar, TimePicker, DatePicker, DateRangePicker, DateTimePicker, CalendarView,
  DataTable, Pagination, Timeline, LineChart, AreaChart, BarChart, Sparkline, SortableList, KanbanBoard, NodeGraph,
} from '.'
import { FileCode, BarChart3, ListChecks, Activity } from 'lucide-react'
import clsx from 'clsx'
import { useContextMenu } from '../hooks/useContextMenu'
import { useTheme } from '../lib/theme'
import type { ToastType, SelectOption, DateRange, ColumnDef, CalendarEvent, TimelineItem, KanbanColumn, TimeValue, EditorTab, DashboardWidgetType, DashboardItem } from '../types'

interface ShowcasePageProps {
  toast: (message: string, type?: ToastType) => void
}

const SECTIONS = [
  { id: 'inputs', label: 'Inputs' },
  { id: 'date', label: 'Date' },
  { id: 'feedback', label: 'Feedback' },
  { id: 'layout', label: 'Layout' },
  { id: 'dashboard', label: 'Dashboard' },
  { id: 'overlays', label: 'Overlays' },
  { id: 'data', label: 'Data & Viz' },
]

// Living gallery of the component library, grouped into category tabs. Use it as a
// reference, then delete it (and its Sidebar NAV entry) once your real pages exist.
export function ShowcasePage({ toast }: ShowcasePageProps) {
  const [section, setSection] = useState('inputs')
  const { wide } = useTheme()
  return (
    <div className={clsx('mx-auto p-6 space-y-6', wide ? 'max-w-none' : 'max-w-3xl')}>
      <div>
        <h1 className="text-xl font-semibold text-app-text">Component kit</h1>
        <p className="text-sm text-app-subtext mt-1">A library for building almost any app — themed by the active accent.</p>
      </div>
      <Tabs tabs={SECTIONS} value={section} onChange={setSection} />
      {section === 'inputs' && <InputsSection toast={toast} />}
      {section === 'date' && <DateSection />}
      {section === 'feedback' && <FeedbackSection toast={toast} />}
      {section === 'layout' && <LayoutSection toast={toast} />}
      {section === 'dashboard' && <DashboardSection />}
      {section === 'overlays' && <OverlaysSection toast={toast} />}
      {section === 'data' && <DataSection toast={toast} />}
    </div>
  )
}

const fruitOptions: SelectOption[] = [
  { value: 'apple', label: 'Apple' }, { value: 'banana', label: 'Banana' },
  { value: 'cherry', label: 'Cherry' }, { value: 'date', label: 'Date' }, { value: 'elderberry', label: 'Elderberry' },
]

function InputsSection({ toast }: ShowcasePageProps) {
  const [pwd, setPwd] = useState('')
  const [search, setSearch] = useState('')
  const [num, setNum] = useState(3)
  const [note, setNote] = useState('')
  const [tags, setTags] = useState(['react', 'electron'])
  const [otp, setOtp] = useState('')
  const [phone, setPhone] = useState('')
  const [price, setPrice] = useState<number | null>(1999)
  const [fruit, setFruit] = useState<string | null>('apple')
  const [multi, setMulti] = useState<string[]>(['apple', 'cherry'])
  const [vol, setVol] = useState(60)
  const [range, setRange] = useState<[number, number]>([20, 80])
  const [color, setColor] = useState('#7c3aed')
  const [sw, setSw] = useState(true)
  const [cb, setCb] = useState(false)
  const [radio, setRadio] = useState('a')
  const [time, setTime] = useState<TimeValue>({ hours: 9, minutes: 41, seconds: 0 })
  const [tSeconds, setTSeconds] = useState(true)
  const [tCap, setTCap] = useState(true)
  const [tWrap, setTWrap] = useState(true)

  return (
    <div className="space-y-6">
      <Card title="Text fields">
        <div className="grid grid-cols-2 gap-4">
          <TextField label="Search" leftIcon={Search} placeholder="Search…" value={search} onChange={e => setSearch(e.target.value)} clearable onClear={() => setSearch('')} />
          <TextField label="Email" leftIcon={Mail} type="email" placeholder="you@example.com" />
          <TextField label="Password" type="password" placeholder="••••••••" value={pwd} onChange={e => setPwd(e.target.value)} />
          <TextField label="With error" value="bad value" error="That doesn't look right" onChange={() => {}} />
          <TextField label="Char count" showCount maxLength={20} placeholder="Max 20" />
          <TextField label="Prefix / suffix" prefix="$" suffix="USD" placeholder="0.00" />
        </div>
      </Card>

      <Card title="Specialized inputs">
        <div className="grid grid-cols-2 gap-4">
          <NumberInput label="Quantity" value={num} onChange={setNum} min={0} max={10} />
          <MaskedInput label="Phone" mask="(###) ###-####" value={phone} onChange={setPhone} placeholder="(555) 123-4567" />
          <CurrencyInput label="Price" value={price} onChange={setPrice} />
          <div>
            <label className="text-xs font-medium text-app-subtext">Verification code</label>
            <div className="mt-1"><OtpInput value={otp} onChange={setOtp} length={6} /></div>
          </div>
          <TagsInput label="Tags" value={tags} onChange={setTags} className="col-span-2" />
          <TextArea label="Notes (autosize)" autosize value={note} onChange={e => setNote(e.target.value)} placeholder="Type a lot…" className="col-span-2" />
        </div>
      </Card>

      <Card title="Selects">
        <div className="grid grid-cols-2 gap-4">
          <Combobox label="Combobox" value={fruit} onChange={setFruit} options={fruitOptions} />
          <MultiSelect label="Multi-select" value={multi} onChange={setMulti} options={fruitOptions} />
          <div className="col-span-2 flex items-center gap-3">
            <span className="text-sm text-app-subtext w-24">Quick select</span>
            <Select value={fruit ?? 'apple'} onChange={setFruit} options={fruitOptions} />
          </div>
        </div>
      </Card>

      <Card title="Sliders & color">
        <div className="space-y-5">
          <div><div className="text-xs text-app-subtext mb-2">Volume: {vol}</div><Slider value={vol} onChange={setVol} /></div>
          <div><div className="text-xs text-app-subtext mb-2">Range: {range[0]}–{range[1]}</div><RangeSlider value={range} onChange={setRange} /></div>
          <ColorPicker label="Color" value={color} onChange={setColor} />
        </div>
      </Card>

      <Card title="Time input (typeable)">
        <div className="flex flex-wrap items-end gap-6">
          <TimeInput
            label="Duration"
            value={time}
            onChange={setTime}
            showSeconds={tSeconds}
            capHours={tCap}
            wrap={tWrap}
          />
          <div className="text-sm text-app-subtext">
            Value: <span className="mono-text text-app-text">{String(time.hours).padStart(2, '0')}:{String(time.minutes).padStart(2, '0')}:{String(time.seconds).padStart(2, '0')}</span>
          </div>
        </div>
        <div className="flex flex-wrap gap-4 mt-4">
          <Switch checked={tSeconds} onChange={setTSeconds} label="Show seconds" />
          <Switch checked={!tCap} onChange={v => setTCap(!v)} label="Allow 99 hours" />
          <Switch checked={tWrap} onChange={setTWrap} label="Wrap at ends" />
        </div>
        <p className="text-xs text-app-muted mt-3">
          ←/→ switch boxes · ↑/↓ step (hold to repeat) · 59m + 1 carries to the hour.
        </p>
      </Card>

      <Card title="Toggles & file upload">
        <div className="space-y-4">
          <Switch checked={sw} onChange={setSw} label="Enable notifications" />
          <Checkbox checked={cb} onChange={setCb} label="I agree to the terms" />
          <RadioGroup value={radio} onChange={setRadio} options={[{ value: 'a', label: 'Option A' }, { value: 'b', label: 'Option B' }, { value: 'c', label: 'Option C' }]} />
          <FileDropzone hint="PNG, JPG up to 10MB" onFiles={f => toast(`${f.length} file(s) selected`, 'success')} onPaths={p => toast(`${p.length} path(s) selected`, 'success')} />
        </div>
      </Card>
    </div>
  )
}

function DateSection() {
  const [date, setDate] = useState<Date | null>(new Date())
  const [range, setRange] = useState<DateRange>({ start: null, end: null })
  const [dt, setDt] = useState<Date | null>(new Date())
  const [time, setTime] = useState(new Date())
  const [cal, setCal] = useState<Date | null>(new Date())
  const [hour12, setHour12] = useState(false)
  const [seconds, setSeconds] = useState(false)

  const today = new Date()
  const ev = (d: number, h: number, dur: number, title: string): CalendarEvent => {
    const start = new Date(today); start.setDate(today.getDate() + d); start.setHours(h, 0, 0, 0)
    const end = new Date(start); end.setHours(h + dur)
    return { id: `${d}-${h}-${title}`, title, start, end }
  }
  const events = [ev(0, 10, 1, 'Standup'), ev(0, 14, 2, 'Design review'), ev(2, 9, 3, 'Workshop'), ev(-1, 16, 1, 'Retro'), ev(4, 11, 2, 'Demo')]

  return (
    <div className="space-y-6">
      <Card title="Pickers">
        <div className="grid grid-cols-2 gap-4">
          <DatePicker label="Date" value={date} onChange={setDate} clearable />
          <DateRangePicker label="Date range" value={range} onChange={setRange} />
          <DateTimePicker label="Date & time" value={dt} onChange={setDt} hour12={hour12} showSeconds={seconds} className="col-span-2" />
        </div>
        <div className="flex items-center gap-4 mt-4">
          <Switch checked={hour12} onChange={setHour12} label="12-hour clock" />
          <Switch checked={seconds} onChange={setSeconds} label="Show seconds" />
        </div>
      </Card>

      <div className="grid grid-cols-2 gap-6">
        <Card title="Calendar"><Calendar mode="single" selected={cal} onSelect={setCal} /></Card>
        <Card title="Time picker"><TimePicker value={time} onChange={setTime} hour12={hour12} showSeconds={seconds} /></Card>
      </div>

      <Card title="Calendar view">
        <div className="h-[28rem]"><CalendarView events={events} /></div>
      </Card>
    </div>
  )
}

function FeedbackSection({ toast }: ShowcasePageProps) {
  const [seg, setSeg] = useState('list')
  const [step, setStep] = useState(1)
  const [progress] = useState(64)

  return (
    <div className="space-y-6">
      <Card title="Feedback atoms">
        <div className="space-y-4">
          <div className="flex flex-wrap gap-2">
            <Badge tone="success">Success</Badge><Badge tone="error">Error</Badge>
            <Badge tone="warning">Warning</Badge><Badge tone="info">Info</Badge><Badge>Neutral</Badge>
          </div>
          <div className="flex items-center gap-4">
            <Spinner /><Progress value={progress} className="flex-1" /><CircularProgress value={progress} showLabel />
          </div>
          <div className="space-y-2"><Skeleton className="h-4 w-3/4" /><Skeleton className="h-4 w-1/2" /></div>
        </div>
      </Card>

      <Card title="Alerts">
        <div className="space-y-2">
          <Alert tone="info" title="Heads up">This is an informational message.</Alert>
          <Alert tone="success" title="Saved" />
          <Alert tone="warning" title="Careful" onClose={() => toast('Dismissed', 'info')}>This action may have consequences.</Alert>
          <Alert tone="error" title="Failed to save" />
        </div>
      </Card>

      <Card title="Navigation & controls">
        <div className="space-y-4">
          <Breadcrumbs items={[{ label: 'Home', onClick: () => {} }, { label: 'Library', onClick: () => {} }, { label: 'Item' }]} />
          <SegmentedControl value={seg} onChange={setSeg} options={[{ value: 'list', label: 'List' }, { value: 'grid', label: 'Grid' }, { value: 'board', label: 'Board' }]} />
          <div className="flex items-center gap-3">
            <Avatar name="Ada Lovelace" />
            <AvatarGroup avatars={[{ name: 'Ada L' }, { name: 'Bob K' }, { name: 'Cy R' }, { name: 'Dan P' }, { name: 'Eve M' }]} />
          </div>
        </div>
      </Card>

      <Card title="Stepper">
        <Stepper steps={[{ id: '1', label: 'Account' }, { id: '2', label: 'Profile' }, { id: '3', label: 'Done' }]} current={step} onStepClick={setStep} />
        <div className="flex gap-2 mt-4">
          <Button variant="ghost" onClick={() => setStep(s => Math.max(0, s - 1))}>Back</Button>
          <Button onClick={() => setStep(s => Math.min(2, s + 1))}>Next</Button>
        </div>
      </Card>

      <Card title="Accordion">
        <Accordion items={[
          { id: '1', title: 'What is this?', content: 'A collapsible section of content.' },
          { id: '2', title: 'Can multiple open?', content: 'Set the `multiple` prop to allow it.' },
          { id: '3', title: 'Is it themed?', content: 'Yes — everything uses the accent + theme tokens.' },
        ]} />
      </Card>
    </div>
  )
}

function LayoutSection({ toast }: ShowcasePageProps) {
  const [tabs, setTabs] = useState<EditorTab[]>([
    { id: 'a', label: 'index.tsx', icon: FileCode },
    { id: 'b', label: 'styles.css', icon: FileCode, dirty: true },
    { id: 'c', label: 'README.md', icon: FileText },
  ])
  const [activeTab, setActiveTab] = useState('a')
  let added = 0
  const box = (label: string) => (
    <div className="flex items-center justify-center rounded-md bg-app-accent/15 text-app-accentBright text-sm py-3 px-4">{label}</div>
  )

  return (
    <div className="space-y-6">
      <Card title="Primitives">
        <div className="space-y-4">
          <HStack gap={2}>{box('H')}{box('Stack')}{box('row')}</HStack>
          <GridBox cols={4} gap={2}>{box('1')}{box('2')}{box('3')}{box('4')}</GridBox>
          <Stack direction="row" gap={3} align="center">
            <div className="text-sm text-app-subtext">Divider</div>
            <Divider orientation="vertical" />
            <Center className="h-10 w-20 rounded-md bg-app-card text-xs text-app-muted">Center</Center>
            <AspectRatio ratio={16 / 9} className="w-28 rounded-md bg-app-card"><Center className="h-full text-xs text-app-muted">16:9</Center></AspectRatio>
          </Stack>
          <Divider label="section" />
          <Container size="sm" padded={false}><div className="text-xs text-app-muted text-center">Container (max-width, centered)</div></Container>
        </div>
      </Card>

      <Card title="Responsive (resize the window)">
        <div className="space-y-4">
          <div className="text-xs text-app-muted">AutoGrid</div>
          <AutoGrid min={140} gap={3}>{Array.from({ length: 6 }, (_, i) => box(`#${i + 1}`))}</AutoGrid>
          <div className="text-xs text-app-muted">Masonry</div>
          <Masonry columns={3} gap={3}>
            {[6, 12, 8, 16, 10, 7].map((h, i) => (
              <div key={i} className="rounded-md bg-app-card border border-app-border p-2 text-xs text-app-subtext" style={{ height: `${h * 8}px` }}>card {i + 1}</div>
            ))}
          </Masonry>
        </div>
      </Card>

      <Card title="Editor tabs (drag to reorder, close, add)">
        <div className="border border-app-border rounded-lg overflow-hidden">
          <EditorTabs
            tabs={tabs}
            activeId={activeTab}
            onActivate={setActiveTab}
            onClose={id => setTabs(t => t.filter(x => x.id !== id))}
            onReorder={setTabs}
            onAdd={() => { const id = `new${++added}-${Date.now()}`; setTabs(t => [...t, { id, label: `untitled-${added}`, icon: FileCode }]); setActiveTab(id) }}
          />
          <div className="p-4 text-sm text-app-subtext bg-app-bg">Active tab: <span className="mono-text text-app-text">{tabs.find(t => t.id === activeTab)?.label ?? '—'}</span></div>
        </div>
      </Card>

      <Card title="Resizable panels (nested)">
        <div className="h-64 border border-app-border rounded-lg overflow-hidden">
          <PanelGroup direction="horizontal" storageKey="demo-panels-h">
            <Panel defaultSize={28} min={15}><div className="h-full bg-app-surface p-3 text-sm text-app-subtext">Sidebar</div></Panel>
            <Panel>
              <PanelGroup direction="vertical" storageKey="demo-panels-v">
                <Panel><div className="h-full p-3 text-sm text-app-subtext">Editor</div></Panel>
                <Panel defaultSize={32} min={15}><div className="h-full bg-app-surface p-3 text-sm text-app-subtext">Terminal</div></Panel>
              </PanelGroup>
            </Panel>
          </PanelGroup>
        </div>
        <p className="text-xs text-app-muted mt-2">Drag the dividers — sizes persist across reloads.</p>
      </Card>

      <Card title="AppShell scaffold">
        <div className="h-56 border border-app-border rounded-lg overflow-hidden">
          <AppShell
            header={<div className="px-4 py-2.5 text-sm font-medium text-app-text">Header</div>}
            sidebar={<div className="w-40 bg-app-bg border-r border-app-border p-3 text-sm text-app-muted space-y-2"><div>Nav item</div><div>Nav item</div></div>}
            footer={<div className="px-4 py-2 text-xs text-app-muted">Footer</div>}
          >
            <div className="p-4 text-sm text-app-subtext">Scrollable content region. <Button variant="ghost" className="ml-1 !py-1 !px-2" onClick={() => toast('AppShell action', 'info')}>Action</Button></div>
          </AppShell>
        </div>
      </Card>
    </div>
  )
}

function DashboardSection() {
  const stat = (label: string, value: string, data: number[]) => (
    <div className="h-full flex flex-col justify-between">
      <div><div className="text-2xl font-semibold text-app-text">{value}</div><div className="text-xs text-app-muted">{label}</div></div>
      <Sparkline data={data} area width={160} height={32} />
    </div>
  )
  const series = [{ name: 'Visits', data: Array.from({ length: 14 }, (_, i) => ({ x: i, y: 20 + Math.round(30 * Math.sin(i / 2) + i * 2) })) }]

  const widgetTypes: DashboardWidgetType[] = [
    { type: 'stat', label: 'Stat', icon: Activity, defaultSize: { w: 3, h: 2 }, render: () => stat('Active users', '1,284', series[0].data.map(p => p.y)) },
    { type: 'chart', label: 'Chart', icon: BarChart3, defaultSize: { w: 6, h: 4 }, render: () => <LineChart series={series} height={150} /> },
    { type: 'list', label: 'List', icon: ListChecks, defaultSize: { w: 3, h: 4 }, render: () => (
      <ul className="space-y-1.5 text-sm text-app-subtext">{['Review PR', 'Ship release', 'Update docs', 'Reply to issues'].map(t => <li key={t} className="flex items-center gap-2"><span className="w-1.5 h-1.5 rounded-full bg-app-accent" />{t}</li>)}</ul>
    ) },
  ]
  const defaultItems: DashboardItem[] = [
    { id: 'd1', type: 'stat', x: 0, y: 0, w: 3, h: 2 },
    { id: 'd2', type: 'chart', x: 3, y: 0, w: 6, h: 4 },
    { id: 'd3', type: 'list', x: 9, y: 0, w: 3, h: 4 },
    { id: 'd4', type: 'stat', x: 0, y: 2, w: 3, h: 2 },
  ]

  return (
    <div className="space-y-3">
      <p className="text-sm text-app-subtext">Toggle <span className="text-app-text font-medium">Edit</span> to drag, resize, add and remove widgets. Switch <span className="text-app-text font-medium">Snap</span>/<span className="text-app-text font-medium">Free</span> placement. Layout persists.</p>
      <Dashboard widgetTypes={widgetTypes} defaultItems={defaultItems} storageKey="demo-dashboard" rowHeight={56} />
    </div>
  )
}

function OverlaysSection({ toast }: ShowcasePageProps) {
  const menu = useContextMenu()
  const [confirm, setConfirm] = useState(false)
  const [drawer, setDrawer] = useState(false)
  const [palette, setPalette] = useState(false)
  const [pop, setPop] = useState(false)
  const popRef = useRef<HTMLButtonElement>(null)

  return (
    <div className="space-y-6">
      <Card title="Context menu">
        <div
          onContextMenu={menu.onContextMenu}
          className="flex items-center justify-center h-24 rounded-lg border-2 border-dashed border-app-border text-app-muted text-sm"
        >
          Right-click here
        </div>
        {menu.isOpen && (
          <ContextMenu position={menu.position} onClose={menu.close} items={[
            { label: 'Copy', icon: Copy, shortcut: '⌘C', onClick: () => toast('Copied', 'success') },
            { label: 'Cut', icon: Scissors, onClick: () => toast('Cut', 'info') },
            { label: 'Paste', icon: Clipboard, disabled: true, onClick: () => {} },
            { type: 'separator' },
            { label: 'Delete', icon: Trash2, danger: true, onClick: () => toast('Deleted', 'error') },
          ]} />
        )}
      </Card>

      <Card title="Triggers">
        <div className="flex flex-wrap items-center gap-3">
          <Dropdown trigger="Actions" items={[
            { label: 'Edit', icon: Pencil, onClick: () => toast('Edit', 'info') },
            { label: 'Delete', icon: Trash2, danger: true, onClick: () => toast('Deleted', 'error') },
          ]} />
          <Tooltip content="A helpful hint"><Button variant="ghost">Hover me</Button></Tooltip>
          <Button variant="danger" onClick={() => setConfirm(true)}>Confirm dialog</Button>
          <Button variant="ghost" onClick={() => setDrawer(true)}>Open drawer</Button>
          <Button variant="ghost" onClick={() => setPalette(true)}>Command palette</Button>
          <button
            ref={popRef}
            onClick={() => setPop(o => !o)}
            className="px-4 py-2.5 rounded-lg border border-app-border text-sm text-app-subtext hover:text-app-text"
          >
            Popover
          </button>
          <Popover anchorRef={popRef} open={pop} onClose={() => setPop(false)} className="p-3 w-48">
            <p className="text-sm text-app-subtext">Anchored popover content. Dismisses on outside-click or Escape.</p>
          </Popover>
        </div>
      </Card>

      {confirm && (
        <ConfirmDialog
          title="Delete item?" message="This cannot be undone." confirmLabel="Delete" danger
          onConfirm={() => { setConfirm(false); toast('Deleted', 'error') }} onClose={() => setConfirm(false)}
        />
      )}
      <Drawer open={drawer} onClose={() => setDrawer(false)} title="Drawer">
        <p className="text-sm text-app-subtext">A slide-in panel. Use it for filters, details, or forms.</p>
      </Drawer>
      <CommandPalette
        open={palette}
        onClose={() => setPalette(false)}
        commands={[
          { id: 'home', label: 'Go to Home', icon: Home, group: 'Navigation', onRun: () => toast('Home', 'info') },
          { id: 'settings', label: 'Open Settings', icon: Settings, group: 'Navigation', onRun: () => toast('Settings', 'info') },
          { id: 'new', label: 'New File', icon: FileText, group: 'Actions', hint: '⌘N', onRun: () => toast('New file', 'success') },
          { id: 'add', label: 'Add Item', icon: Plus, group: 'Actions', onRun: () => toast('Added', 'success') },
        ]}
      />
    </div>
  )
}

interface Person { id: string; name: string; role: string; age: number }
const people: Person[] = Array.from({ length: 23 }, (_, i) => ({
  id: String(i + 1),
  name: ['Ada Lovelace', 'Alan Turing', 'Grace Hopper', 'Linus T', 'Margaret H', 'Dennis R', 'Ken T'][i % 7] + ` ${i + 1}`,
  role: ['Engineer', 'Designer', 'PM', 'QA'][i % 4],
  age: 24 + (i % 30),
}))
const personCols: ColumnDef<Person>[] = [
  { key: 'name', header: 'Name', accessor: r => r.name, sortable: true },
  { key: 'role', header: 'Role', accessor: r => r.role, sortable: true, render: r => <Badge tone="info">{r.role}</Badge> },
  { key: 'age', header: 'Age', accessor: r => r.age, sortable: true, align: 'right' },
]

function DataSection({ toast }: ShowcasePageProps) {
  const [items, setItems] = useState(['First item', 'Second item', 'Third item', 'Fourth item'])
  const [board, setBoard] = useState<KanbanColumn[]>([
    { id: 'todo', title: 'To do', cards: [{ id: 'c1', title: 'Research' }, { id: 'c2', title: 'Wireframes' }] },
    { id: 'doing', title: 'In progress', cards: [{ id: 'c3', title: 'Build UI', description: 'Component library' }] },
    { id: 'done', title: 'Done', cards: [{ id: 'c4', title: 'Setup repo' }] },
  ])
  const [page, setPage] = useState(3)

  const series = [{ name: 'Visits', data: Array.from({ length: 14 }, (_, i) => ({ x: i, y: 20 + Math.round(30 * Math.sin(i / 2) + i * 2) })) }]
  const bars = [{ label: 'Mon', value: 12 }, { label: 'Tue', value: 19 }, { label: 'Wed', value: 7 }, { label: 'Thu', value: 22 }, { label: 'Fri', value: 15 }]

  const today = new Date()
  const tl = (offset: number, len: number, lane: string, label: string): TimelineItem => {
    const start = new Date(today); start.setDate(today.getDate() + offset)
    const end = new Date(start); end.setDate(start.getDate() + len)
    return { id: `${lane}-${label}`, label, start, end, lane }
  }
  const timelineItems = [tl(-2, 4, 'Design', 'Mockups'), tl(2, 5, 'Design', 'Polish'), tl(0, 6, 'Build', 'Components'), tl(6, 4, 'Build', 'Integration'), tl(1, 3, 'QA', 'Test plan')]

  const nodes: Node[] = [
    { id: '1', type: 'themed', position: { x: 0, y: 40 }, data: { label: 'Input' } },
    { id: '2', type: 'themed', position: { x: 180, y: 0 }, data: { label: 'Process' } },
    { id: '3', type: 'themed', position: { x: 180, y: 90 }, data: { label: 'Validate' } },
    { id: '4', type: 'themed', position: { x: 360, y: 45 }, data: { label: 'Output' } },
  ]
  const edges: Edge[] = [
    { id: 'e1-2', source: '1', target: '2' }, { id: 'e1-3', source: '1', target: '3' },
    { id: 'e2-4', source: '2', target: '4' }, { id: 'e3-4', source: '3', target: '4' },
  ]

  return (
    <div className="space-y-6">
      <Card title="Data table"><DataTable data={people} columns={personCols} rowKey={r => r.id} filterable selectable pageSize={6} /></Card>

      <Card title="Pagination (standalone)"><Pagination page={page} pageCount={12} onChange={setPage} siblings={1} /></Card>

      <div className="grid grid-cols-2 gap-6">
        <Card title="Line chart"><LineChart series={series} /></Card>
        <Card title="Area chart"><AreaChart series={series} /></Card>
        <Card title="Bar chart"><BarChart data={bars} /></Card>
        <Card title="Sparkline">
          <div className="flex items-center gap-3">
            <span className="text-2xl font-semibold text-app-text">1,284</span>
            <Sparkline data={series[0].data.map(p => p.y)} area width={120} height={36} />
          </div>
        </Card>
      </div>

      <Card title="Timeline"><Timeline items={timelineItems} /></Card>

      <Card title="Sortable list (drag to reorder)">
        <SortableList items={items} getId={x => x} onReorder={setItems} renderItem={x => <span className="text-sm text-app-text">{x}</span>} />
      </Card>

      <Card title="Kanban board (drag cards)"><KanbanBoard columns={board} onChange={setBoard} /></Card>

      <Card title="Node graph">
        <NodeGraph defaultNodes={nodes} defaultEdges={edges} height={320} />
        <p className="text-xs text-app-muted mt-2">Drag nodes, pan/zoom, and connect handles. <Button variant="ghost" className="ml-1 !py-1 !px-2" onClick={() => toast('Tip: drag from a node handle', 'info')}>Tip</Button></p>
      </Card>
    </div>
  )
}
