<script lang='ts'>
  import type { FontDescriptor } from '@common/platform/resources'
  import { FontDialog } from '@components'
  import { Message, Option, Select } from '@ggchivalrous/db-ui'
  import { createEventDispatcher } from 'svelte'
  import './index.scss'

  export let fonts: FontDescriptor[] = []
  export let value = 'system-ui'
  export let clearable = false

  const dispatch = createEventDispatcher()
  const systemFonts = ['system-ui', 'sans-serif', 'serif', 'monospace']
  let visible = false

  $: if (!value || (!systemFonts.includes(value) && !fonts.some(font => font.name === value))) value = 'system-ui'

  async function deleteFont(name: string) {
    const result = await window.platform.files.removeFont(name)
    if (!result.ok) {
      Message.error(result.error.message)
      return
    }
    Message.success(result.data ? '删除成功' : '记录不存在')
    dispatch('update')
  }
</script>

<Select size='mini' bind:value {clearable} class='no-drag grass font-select' style='font-family: {value}'>
  <div class='button add-font' on:click={() => { visible = true }} on:keypress role='button' tabindex='-1'>+</div>
  {#each systemFonts as name}
    <Option value={name}><span class='font-name' style:font-family={name}>{name}</span></Option>
  {/each}
  {#each fonts as font}
    <Option value={font.name}>
      <span class='font-name' style:font-family={font.name}>{font.name}</span>
      <span class='font-del' on:click|preventDefault|capture|stopPropagation={() => deleteFont(font.name)} on:keypress role='button' tabindex='-1'>x</span>
    </Option>
  {/each}
</Select>

<FontDialog bind:visible on:update />
