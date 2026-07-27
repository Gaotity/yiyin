import { Drawer as BaseDrawer } from '@base-ui/react/drawer'
import type { ReactNode } from 'react'

interface DrawerProps {
  open: boolean
  onOpenChange(open: boolean): void
  title: string
  children: ReactNode
}

export function Drawer({ open, onOpenChange, title, children }: DrawerProps) {
  return (
    <BaseDrawer.Root
      open={open}
      onOpenChange={onOpenChange}
      swipeDirection="right"
    >
      <BaseDrawer.Portal>
        <BaseDrawer.Backdrop className="dialog-backdrop" />
        <BaseDrawer.Viewport className="drawer-viewport">
          <BaseDrawer.Popup className="drawer-surface">
            <BaseDrawer.Title>{title}</BaseDrawer.Title>
            <BaseDrawer.Content>{children}</BaseDrawer.Content>
            <BaseDrawer.Close className="button">关闭</BaseDrawer.Close>
          </BaseDrawer.Popup>
        </BaseDrawer.Viewport>
      </BaseDrawer.Portal>
    </BaseDrawer.Root>
  )
}
