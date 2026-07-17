import type { ExternalDestinationDto } from '../../platform/types'

interface FooterProps {
  onOpen(destination: ExternalDestinationDto): void
}

export function Footer({ onOpen }: FooterProps) {
  return (
    <footer className="app-footer">
      <button type="button" onClick={() => onOpen('currentRelease')}>
        v1.6.0
      </button>
      <button type="button" onClick={() => onOpen('bilibiliProfile')}>
        B站 - 不长肉的小伙吒
      </button>
      <button type="button" onClick={() => onOpen('repository')}>
        © 2023 Github - GGChivalrous.
      </button>
    </footer>
  )
}
