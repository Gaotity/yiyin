export interface ResourceDescriptor {
  resourceId: string
  displayName: string
  resourceUrl: string
}

export interface FontDescriptor extends ResourceDescriptor {
  name: string
}

export interface TaskDescriptor extends ResourceDescriptor {
  taskId: string
}
