import type { PlatformClient } from './client'
import type {
  BootstrapDto,
  CommandErrorDto,
  ExternalDestinationDto,
  MetadataDto,
  ResourceDescriptorDto,
  TaskDescriptorDto,
  TaskStatusEventDto,
  UpdateConfigRequestDto,
} from './types'

interface FakeOptions {
  snapshot?: BootstrapDto
  deferredBootstrap?: boolean
}

export interface FakePlatformCalls {
  bootstrap: number
  unlisten: number
  resetConfig: number
  minimizeWindow: number
  closeWindow: number
  updateConfig: UpdateConfigRequestDto[]
  startTasks: string[][]
  openExternalUrl: ExternalDestinationDto[]
}

export interface FakePlatformHarness {
  client: PlatformClient
  calls: FakePlatformCalls
  emitTaskStatus(event: TaskStatusEventDto): void
  emitDropError(error: CommandErrorDto): void
  resolveBootstrap(): void
  rejectNextBootstrap(error: CommandErrorDto): void
  setSnapshot(snapshot: BootstrapDto): void
}

export function defaultBootstrap(
  overrides: Partial<BootstrapDto> = {},
): BootstrapDto {
  return {
    config: {
      version: '2.0.0',
      options: {
        quickOutput: false,
        landscape: false,
        solidBackground: false,
        solidColor: '#fff',
        radius: 2.1,
        radiusVisible: true,
        shadow: 6,
        shadowVisible: true,
        backgroundRatioVisible: false,
        backgroundRatio: { width: 0, height: 0 },
        font: 'PingFang SC',
        mainImageWidth: 90,
        textMargin: 0.4,
        quality: 100,
        miniTopBottomMargin: 0,
        backgroundBlur: 100,
        previewVisible: false,
      },
      templateFields: [],
      customTemplateFields: [],
      templates: [],
    },
    // Fixture copy of the domain-generated values — nothing mechanically
    // couples these; keep in sync with
    // numeric_constraints_match_the_validation_bounds in yiyin-domain.
    constraints: {
      mainImageWidth: { minimum: 1, maximum: 100, decimals: 0 },
      textMargin: { minimum: 0, maximum: 10_000, decimals: 2 },
      miniTopBottomMargin: { minimum: 0, maximum: 100, decimals: 2 },
      radius: { minimum: 0, maximum: 50, decimals: 1 },
      shadow: { minimum: 0, maximum: 50, decimals: 1 },
      quality: { minimum: 1, maximum: 100, decimals: 0 },
      backgroundBlur: { minimum: 0, maximum: 100, decimals: 0 },
    },
    resources: [],
    tasks: [],
    warnings: [],
    ...overrides,
  }
}

export function createFakePlatformClient(
  options: FakeOptions = {},
): FakePlatformHarness {
  let snapshot = clone(options.snapshot ?? defaultBootstrap())
  let deferred = options.deferredBootstrap ?? false
  let deferredResolve: ((value: BootstrapDto) => void) | undefined
  let nextBootstrapError: CommandErrorDto | undefined
  const listeners = new Set<(event: TaskStatusEventDto) => void>()
  const dropErrorListeners = new Set<(error: CommandErrorDto) => void>()
  const calls: FakePlatformCalls = {
    bootstrap: 0,
    unlisten: 0,
    resetConfig: 0,
    minimizeWindow: 0,
    closeWindow: 0,
    updateConfig: [],
    startTasks: [],
    openExternalUrl: [],
  }

  const client: PlatformClient = {
    bootstrap() {
      calls.bootstrap += 1
      if (nextBootstrapError) {
        const error = nextBootstrapError
        nextBootstrapError = undefined
        return Promise.reject(error)
      }
      if (deferred) {
        return new Promise<BootstrapDto>((resolve) => {
          deferredResolve = resolve
        })
      }
      return Promise.resolve(clone(snapshot))
    },
    updateConfig(request) {
      calls.updateConfig.push(clone(request))
      snapshot.config = clone(request.config)
      return Promise.resolve(clone(snapshot.config))
    },
    resetConfig() {
      calls.resetConfig += 1
      snapshot.config = defaultBootstrap().config
      return Promise.resolve(clone(snapshot.config))
    },
    chooseOutputDirectory: () => Promise.resolve(clone(snapshot.config)),
    openOutputDirectory: () => Promise.resolve(),
    chooseImages: () => Promise.resolve(clone(snapshot.tasks)),
    registerFont(name) {
      const resource = fakeResource(
        `font-${snapshot.resources.length + 1}`,
        'font',
        name,
      )
      snapshot.resources.push(resource)
      return Promise.resolve(clone(resource))
    },
    removeFont(id) {
      snapshot.resources = snapshot.resources.filter(
        (resource) => resource.id !== id,
      )
      return Promise.resolve(
        clone(
          snapshot.resources.filter((resource) => resource.kind === 'font'),
        ),
      )
    },
    registerOverlay() {
      const resource = fakeResource(
        `overlay-${snapshot.resources.length + 1}`,
        'overlay',
        'overlay.png',
      )
      snapshot.resources.push(resource)
      return Promise.resolve(clone(resource))
    },
    readTaskExif: () => Promise.resolve(null as MetadataDto | null),
    startTasks(ids) {
      calls.startTasks.push([...ids])
      snapshot.tasks = updateTasks(snapshot.tasks, ids, 'queued')
      return Promise.resolve(clone(snapshot.tasks))
    },
    previewTask(id) {
      snapshot.tasks = updateTasks(snapshot.tasks, [id], 'running')
      return Promise.resolve(clone(snapshot.tasks))
    },
    cancelTask(id) {
      snapshot.tasks = updateTasks(snapshot.tasks, [id], 'cancelled')
      return Promise.resolve(clone(snapshot.tasks))
    },
    clearTasks() {
      snapshot.tasks = []
      return Promise.resolve([])
    },
    minimizeWindow() {
      calls.minimizeWindow += 1
      return Promise.resolve()
    },
    closeWindow() {
      calls.closeWindow += 1
      return Promise.resolve()
    },
    openExternalUrl(destination) {
      calls.openExternalUrl.push(destination)
      return Promise.resolve()
    },
    onTaskStatus(listener) {
      listeners.add(listener)
      return Promise.resolve(() => {
        listeners.delete(listener)
        calls.unlisten += 1
      })
    },
    onDropError(listener) {
      dropErrorListeners.add(listener)
      return Promise.resolve(() => {
        dropErrorListeners.delete(listener)
        calls.unlisten += 1
      })
    },
  }

  return {
    client,
    calls,
    emitTaskStatus(event) {
      for (const listener of listeners) {
        listener(clone(event))
      }
    },
    emitDropError(error) {
      for (const listener of dropErrorListeners) {
        listener(clone(error))
      }
    },
    resolveBootstrap() {
      deferred = false
      deferredResolve?.(clone(snapshot))
      deferredResolve = undefined
    },
    rejectNextBootstrap(error) {
      nextBootstrapError = clone(error)
    },
    setSnapshot(next) {
      snapshot = clone(next)
    },
  }
}

export function createBrowserFakePlatformClient(): PlatformClient {
  const harness = createFakePlatformClient()
  const registered: TaskDescriptorDto = {
    id: 'browser-task',
    displayName: 'browser-photo.jpg',
    state: 'registered',
    progress: 0,
    preview: false,
    resource: null,
  }
  let chooseCount = 0

  harness.client.chooseImages = () => {
    chooseCount += 1
    if (chooseCount > 1) {
      return Promise.reject({
        code: 'FILE_INVALID',
        message: 'The selected file is invalid.',
      })
    }
    harness.setSnapshot(defaultBootstrap({ tasks: [registered] }))
    return Promise.resolve(clone([registered]))
  }
  harness.client.previewTask = () =>
    Promise.resolve([
      {
        ...registered,
        state: 'completed',
        progress: 100,
        preview: true,
        resource: fakeResource(
          'browser-preview',
          'preview',
          registered.displayName,
        ),
      },
    ])
  harness.client.startTasks = () =>
    Promise.resolve([
      {
        ...registered,
        state: 'completed',
        progress: 100,
        resource: fakeResource(
          'browser-output',
          'output',
          registered.displayName,
        ),
      },
    ])

  return harness.client
}

function fakeResource(
  id: string,
  kind: ResourceDescriptorDto['kind'],
  displayName: string,
): ResourceDescriptorDto {
  return { id, kind, displayName, url: `yiyin://resource/${id}` }
}

function updateTasks(
  tasks: TaskDescriptorDto[],
  ids: string[],
  state: TaskDescriptorDto['state'],
): TaskDescriptorDto[] {
  const selected = new Set(ids)
  return tasks.map((task) =>
    selected.has(task.id) ? { ...task, state } : task,
  )
}

function clone<T>(value: T): T {
  return structuredClone(value)
}
