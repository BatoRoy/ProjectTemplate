// Barrel for the component library. Import from one place:
//   import { Button, DatePicker, KanbanBoard, useToast } from '../components'

// ── Core / overlays ─────────────────────────────────────────
export { Modal, Input, Button } from './Modal'
export { ToastContainer } from './Toast'
export { ContextMenu } from './ContextMenu'
export { Dropdown, Select } from './Dropdown'
export { Tooltip } from './Tooltip'
export { ConfirmDialog } from './ConfirmDialog'
export { Tabs } from './Tabs'
export { ErrorBoundary } from './ErrorBoundary'
export { Sidebar } from './Sidebar'
export { AppOptionsModal } from './AppOptionsModal'
export { MenuList } from './Menu'

export { Popover } from './overlay/Popover'
export { Drawer } from './overlay/Drawer'
export { CommandPalette } from './overlay/CommandPalette'
export type { Command } from './overlay/CommandPalette'

// ── Form controls ───────────────────────────────────────────
export { Switch, Checkbox, RadioGroup } from './Form'
export { Field } from './inputs/Field'
export { TextField } from './inputs/TextField'
export { NumberInput } from './inputs/NumberInput'
export { TextArea } from './inputs/TextArea'
export { TagsInput } from './inputs/TagsInput'
export { OtpInput } from './inputs/OtpInput'
export { MaskedInput } from './inputs/MaskedInput'
export { CurrencyInput } from './inputs/CurrencyInput'
export { Combobox } from './inputs/Combobox'
export { MultiSelect } from './inputs/MultiSelect'
export { Slider, RangeSlider } from './inputs/Slider'
export { ColorPicker } from './inputs/ColorPicker'
export { FileDropzone } from './inputs/FileDropzone'
export { TimeInput } from './inputs/TimeInput'
export { SearchInput } from './inputs/SearchInput'
export { CodeEditor } from './inputs/CodeEditor'
export type { CodeLanguage } from './inputs/CodeEditor'

// ── Feedback & layout ───────────────────────────────────────
export { Card, Badge, Spinner, Skeleton, EmptyState } from './Feedback'
export { Accordion, Collapsible } from './layout/Accordion'
export { Stepper } from './layout/Stepper'
export { ResizablePanels } from './layout/ResizablePanels'
export { Breadcrumbs } from './layout/Breadcrumbs'
export { Avatar, AvatarGroup } from './layout/Avatar'
export { Progress, CircularProgress } from './layout/Progress'
export { SegmentedControl } from './layout/SegmentedControl'
export { Alert } from './layout/Alert'
// Structural layout
export { Stack, HStack, VStack, Grid, Container, Center, AspectRatio, Divider, Spacer } from './layout/Primitives'
export { AutoGrid, Masonry } from './layout/Responsive'
export { Scrollable } from './layout/Scrollable'
export { AppShell } from './layout/AppShell'
export { PanelGroup, Panel } from './layout/PanelGroup'
export { EditorTabs } from './layout/EditorTabs'
export { Dashboard } from './layout/Dashboard'

// ── Date & calendar ─────────────────────────────────────────
export { Calendar } from './date/Calendar'
export { TimePicker } from './date/TimePicker'
export { DatePicker } from './date/DatePicker'
export { DateRangePicker } from './date/DateRangePicker'
export { DateTimePicker } from './date/DateTimePicker'
export { CalendarView } from './date/CalendarView'

// ── Data & viz ──────────────────────────────────────────────
export { Pagination } from './data/Pagination'
export { DataTable } from './data/DataTable'
export { Timeline } from './data/Timeline'
export { LineChart, AreaChart, BarChart, Sparkline } from './data/Charts'
export { SortableList } from './data/SortableList'
export { KanbanBoard } from './data/KanbanBoard'
export { NodeGraph } from './data/NodeGraph'

// ── Hooks ───────────────────────────────────────────────────
export { useToast } from '../hooks/useToast'
export { useContextMenu } from '../hooks/useContextMenu'
export { useHotkeys } from '../hooks/useHotkeys'
export { useDismiss } from '../hooks/useDismiss'
export { useControllableState } from '../hooks/useControllableState'
export { useHoldRepeat } from '../hooks/useHoldRepeat'
export { useElementSize } from '../hooks/useElementSize'

// ── Shared types ────────────────────────────────────────────
export type * from '../types'
