import { useCallback } from 'react'
import type { ReactNode } from 'react'
import {
  ReactFlow, Background, BackgroundVariant, Controls, MiniMap,
  useNodesState, useEdgesState, addEdge, Handle, Position,
} from '@xyflow/react'
import type { Node, Edge, Connection, NodeProps, NodeTypes } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import clsx from 'clsx'
import { useTheme } from '../../lib/theme'

// A themed default node — use `type: 'themed'` on your nodes to get this look.
function ThemedNode({ data }: NodeProps) {
  return (
    <div className="bg-app-card border border-app-border rounded-lg px-3.5 py-2 text-sm text-app-text shadow-md min-w-[7rem] text-center">
      <Handle type="target" position={Position.Left} className="!bg-app-accent !border-app-bg" />
      {data.label as ReactNode}
      <Handle type="source" position={Position.Right} className="!bg-app-accent !border-app-bg" />
    </div>
  )
}

const nodeTypes: NodeTypes = { themed: ThemedNode }

interface NodeGraphProps {
  defaultNodes?: Node[]
  defaultEdges?: Edge[]
  height?: number | string
  minimap?: boolean
  className?: string
}

// Node/flow editor wrapping @xyflow/react with theme-aware nodes, edges, controls,
// minimap and dotted background. Uncontrolled by default (pan/zoom/connect built in).
export function NodeGraph({ defaultNodes = [], defaultEdges = [], height = 400, minimap = true, className }: NodeGraphProps) {
  const { theme } = useTheme()
  const [nodes, , onNodesChange] = useNodesState(defaultNodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(defaultEdges)
  const onConnect = useCallback((c: Connection) => setEdges(eds => addEdge(c, eds)), [setEdges])

  return (
    <div className={clsx('rounded-xl border border-app-border overflow-hidden', className)} style={{ height }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        nodeTypes={nodeTypes}
        colorMode={theme === 'light' ? 'light' : 'dark'}
        fitView
        proOptions={{ hideAttribution: true }}
      >
        <Background variant={BackgroundVariant.Dots} gap={16} size={1} color="rgb(var(--app-border))" />
        <Controls />
        {minimap && <MiniMap pannable zoomable maskColor="rgb(var(--app-bg) / 0.6)" nodeColor="rgb(var(--app-accent))" />}
      </ReactFlow>
    </div>
  )
}
