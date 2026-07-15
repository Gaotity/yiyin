<script lang='ts'>
  import type { ShadowRenderRequest, TextRenderRequest } from '@common/platform/bridge'
  import type { IFontInfo } from '@web/util/util'
  import type { IFileInfo, TInputEvent } from './interface'
  import { Message } from '@ggchivalrous/db-ui'
  import { ImageTool } from '@web/modules/image-tool'
  import { TextTool } from '@web/modules/text-tool'
  import { config } from '@web/store/config'
  import { importFont } from '@web/util/util'
  import { Actions, Footer, Header, ParamSetting, TempSetting } from './components'
  import './index.scss'

  let fileInfoList: IFileInfo[] = []
  let fileSelectDom: HTMLInputElement | null = null
  let showParamSetting = false
  let showTempSetting = false
  let fontList: IFontInfo[] = []

  $: importFont(fontList)
  $: fontList = $config.fonts.map(font => ({ name: font.name, path: font.resourceUrl }))

  const unsubscribeText = window.platform.events.onTextRender(async (data: TextRenderRequest) => {
    const textTool = new TextTool(data.exif, data)
    const images = await textTool.genTextImg().catch((error) => {
      console.error('Text render failed', error)
      return []
    })
    await window.platform.tasks.completeTextRender(data.taskId, images)
  })

  const unsubscribeShadow = window.platform.events.onShadowRender(async (data: ShadowRenderRequest) => {
    const tool = new ImageTool(data)
    const image = await tool.genMainImgShadow()
    await window.platform.tasks.completeShadowRender(data.taskId, image)
  })

  window.addEventListener('beforeunload', () => {
    unsubscribeText()
    unsubscribeShadow()
  }, { once: true })

  async function registerFiles(files: File[]) {
    const imageFiles = files.filter((file) => {
      if (file.type.startsWith('image/')) return true
      Message.error(`${file.name} 文件非图片文件`)
      return false
    })
    if (!imageFiles.length) return

    const result = await window.platform.files.registerImages(imageFiles)
    if (!result.ok) {
      Message.error(`图片添加失败：${result.error.message}`)
      return
    }

    fileInfoList = [...result.data.reverse(), ...fileInfoList]
    if ($config.options.iot) await startTask()
  }

  async function onFileChange(event: TInputEvent) {
    await registerFiles(Array.from(event.currentTarget.files ?? []))
    if (fileSelectDom) fileSelectDom.value = ''
  }

  async function startTask() {
    const result = await window.platform.tasks.start()
    if (!result.ok) Message.error(result.error.message || '水印生成开启失败')
  }

  window.addEventListener('drop', (event) => {
    event.preventDefault()
    event.stopPropagation()
    void registerFiles(Array.from(event.dataTransfer?.files ?? []))
  })
  window.addEventListener('dragover', (event) => {
    event.preventDefault()
    event.stopPropagation()
  })
</script>

<Header />

<div id='root'>
  <div class='guide'>壹印 · 本地照片水印工具</div>
  <div class='desc'>图片处理仅在本机完成</div>

  <input type='file' id='path' accept='image/jpeg,image/png,image/webp' bind:this={fileSelectDom} on:change={onFileChange} multiple class='hide' />

  <div class='body'>
    <div class='content'>
      <Actions bind:fileInfoList={fileInfoList} />
    </div>

    <div class='button-wrap'>
      <label for='path' class='button grass'>添加图片</label>
      <div class='button grass' on:click={startTask} on:keypress role='button' tabindex='-1'>生成印框</div>
      <div class='button grass' on:click={() => { showParamSetting = true }} on:keypress role='button' tabindex='-1'>参数设置</div>
      <div class='button grass' on:click={() => { showTempSetting = true }} on:keypress role='button' tabindex='-1'>模板设置</div>
    </div>
  </div>

  <Footer />
  <ParamSetting bind:visible={showParamSetting} />
  <TempSetting bind:visible={showTempSetting} />
</div>
