import type { ResourceDescriptor } from '@common/platform/resources'
import { Buffer } from 'node:buffer'
import { randomBytes } from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import sharp from 'sharp'
import { PlatformBoundaryError } from '../ipc/errors'
import { assertContainedPath, assertImageDimensions, detectFileKind, validateFileSize } from './policy'

interface ResourceEntry extends ResourceDescriptor {
  path: string
  allowedRoot: string
  kind: Exclude<ReturnType<typeof detectFileKind>, 'unknown'>
}

export class ResourceRegistry {
  private readonly entries = new Map<string, ResourceEntry>()
  private readonly idsByPath = new Map<string, string>()

  async registerImage(candidatePath: string, displayName: string) {
    return this.register(candidatePath, displayName, 'image')
  }

  async registerFont(candidatePath: string, displayName: string) {
    return this.register(candidatePath, displayName, 'font')
  }

  async registerManaged(candidatePath: string, displayName: string, type: 'image' | 'font') {
    return this.register(candidatePath, displayName, type)
  }

  resolve(resourceId: string) {
    const entry = this.entries.get(resourceId)
    if (!entry) throw new PlatformBoundaryError('RESOURCE_NOT_FOUND', 'The selected resource is no longer available')

    try {
      const realPath = fs.realpathSync(entry.path)
      assertContainedPath(entry.allowedRoot, realPath)
      if (!fs.statSync(realPath).isFile()) throw new Error('not a file')
      return realPath
    }
    catch (error) {
      if (error instanceof PlatformBoundaryError) throw error
      throw new PlatformBoundaryError('RESOURCE_NOT_FOUND', 'The selected resource is no longer available')
    }
  }

  resolveUrl(resourceUrl: string) {
    let url: URL
    try {
      url = new URL(resourceUrl)
    }
    catch {
      throw new PlatformBoundaryError('FILE_INVALID', 'Configuration contains an invalid resource reference')
    }

    if (url.protocol !== 'yiyin:' || url.hostname !== 'resource') {
      throw new PlatformBoundaryError('FILE_INVALID', 'Configuration contains an invalid resource reference')
    }
    return this.resolve(url.pathname.slice(1))
  }

  descriptor(resourceId: string): ResourceDescriptor {
    const entry = this.entries.get(resourceId)
    if (!entry) throw new PlatformBoundaryError('RESOURCE_NOT_FOUND', 'The selected resource is no longer available')
    return { resourceId, displayName: entry.displayName, resourceUrl: entry.resourceUrl }
  }

  private async register(candidatePath: string, displayName: string, type: 'image' | 'font') {
    let realPath: string
    let stat: fs.Stats
    try {
      realPath = fs.realpathSync(candidatePath)
      stat = fs.statSync(realPath)
    }
    catch {
      throw new PlatformBoundaryError('FILE_NOT_FOUND', 'The selected file is no longer available')
    }

    if (!stat.isFile()) throw new PlatformBoundaryError('FILE_INVALID', 'The selected resource is not a file')
    validateFileSize(type, stat.size)
    const handle = fs.openSync(realPath, 'r')
    const header = Buffer.alloc(16)
    try {
      fs.readSync(handle, header, 0, header.length, 0)
    }
    finally {
      fs.closeSync(handle)
    }

    const kind = detectFileKind(header)
    const allowedKinds = type === 'image' ? new Set(['jpeg', 'png', 'webp']) : new Set(['otf', 'ttf'])
    if (!allowedKinds.has(kind)) throw new PlatformBoundaryError('FILE_INVALID', `The selected ${type} has an unsupported format`)

    if (type === 'image') {
      const metadata = await sharp(realPath, { limitInputPixels: false }).metadata()
      assertImageDimensions(metadata.width, metadata.height)
    }

    const existingId = this.idsByPath.get(realPath)
    if (existingId) return this.descriptor(existingId)

    const resourceId = randomBytes(16).toString('hex')
    const entry: ResourceEntry = {
      resourceId,
      displayName: path.basename(displayName),
      resourceUrl: `yiyin://resource/${resourceId}`,
      path: realPath,
      allowedRoot: path.dirname(realPath),
      kind: kind as ResourceEntry['kind'],
    }
    this.entries.set(resourceId, entry)
    this.idsByPath.set(realPath, resourceId)
    return this.descriptor(resourceId)
  }
}
