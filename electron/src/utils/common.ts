import fs from 'node:fs'

import path from 'node:path'

export * from '@/common/utils'

export function getFileName(dir: string, fileName: string) {
  const fileNameList = fs.readdirSync(dir)
  const fileNameParse = path.parse(fileName)
  fileName = `${fileNameParse.name}.jpg`
  const isExist = fileNameList.find(i => i === fileName)

  if (!isExist) {
    return `${fileNameParse.name}.jpg`
  }

  const fileNameSplitArr = fileNameParse.name.split('-')
  const parseList = fileNameList
    .filter(i => i.startsWith(fileNameParse.name))
    .sort()
    .map((i) => {
      const parse = path.parse(i)
      const arr = parse.name.split('-')
      const info = {
        ...parse,
        baseName: '',
        index: 0,
      }

      if (arr.length === fileNameSplitArr.length + 1) {
        const index = +(arr.pop() ?? '')
        if (!Number.isNaN(index)) {
          info.index = index
        }
      }

      info.baseName = arr.join('-')
      return info
    })
    .sort((a, b) => b.index - a.index)

  return `${fileNameParse.name}-${(parseList[0]?.index ?? 0) + 1}.jpg`
}
