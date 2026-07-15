export function tryCatch<T = any>(p: () => T, defV?: T, errCb?: (e: Error) => void) {
  try {
    return p()
  }
  catch (e: any) {
    if (typeof errCb === 'function') {
      errCb(e)
    }
    return defV
  }
}

export function tryAsyncCatch<T = any>(p: Promise<T>, defV?: T, errCb?: (e: Error) => void) {
  return p.then(e => e, (e) => {
    if (typeof errCb === 'function') {
      errCb(e)
    }
    return defV
  })
}

export function usePromise<T = any>(): [Promise<T>, (value: T | PromiseLike<T>) => void, (reason?: any) => void] {
  let res!: (value: T | PromiseLike<T>) => void
  let rej!: (reason?: unknown) => void
  const pro = new Promise<T>((resolve, reject) => {
    res = resolve
    rej = reject
  })

  return [pro, res, rej]
}

export function sleep(ms: number) {
  return new Promise((e) => {
    setTimeout(e, ms)
  })
}

export async function pollSleep(fn: () => boolean, ms = 500, timeout = 60e3) {
  let totalMs = 0
  while (!fn() && totalMs < timeout) {
    await sleep(ms)
    totalMs += ms
  }
}

interface IArrToObjCb<T, R = T> {
  (item: T, objV: R, index: number): R
}

export function arrToObj<T = any, R = T>(arr: T[], field?: string, cb?: IArrToObjCb<T, R>): Record<any, R> {
  const _obj: Record<any, any> = {}

  if (typeof cb !== 'function') {
    cb = item => item as any
  }

  for (let i = 0; i < arr.length; i++) {
    const item: any = arr[i]

    if (field === undefined || field === null) {
      _obj[i] = cb(item, _obj[i], i)
    }
    else {
      _obj[item[field]] = cb(item, _obj[item[field]], i)
    }
  }

  return _obj
}
