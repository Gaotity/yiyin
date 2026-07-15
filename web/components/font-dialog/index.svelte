<script lang='ts'>
  import { Dialog, Form, FormItem, Input, Message } from '@ggchivalrous/db-ui'
  import { createEventDispatcher } from 'svelte'
  import './index.scss'

  export let visible = false
  const dispatch = createEventDispatcher()
  let name = ''
  let file: File | null = null

  async function submit() {
    if (!name.trim() || !file) {
      Message.info('请填写完整')
      return
    }
    const result = await window.platform.files.registerFont(file, name.trim())
    if (!result.ok) {
      Message.error(result.error.message)
      return
    }
    Message.success('添加成功')
    dispatch('update')
    visible = false
    name = ''
    file = null
  }

  function chooseFile(event: Event & { currentTarget: HTMLInputElement }) {
    file = event.currentTarget.files?.[0] ?? null
    if (!name && file) name = file.name.replace(/\.[^.]+$/, '').replace(/[^\u4E00-\u9FA5\w-]/g, '')
  }
</script>

<Dialog bind:visible width='400px' class='font-dialog'>
  <Form>
    <FormItem label='字体名称'><Input type='text' bind:value={name} placeholder='Enter name...' /></FormItem>
    <FormItem label='字体文件'><input class='font-input-file grass' type='file' accept='.ttf,.otf' on:change={chooseFile} /></FormItem>
  </Form>
  <footer class='modal-footer'>
    <div class='grass button' on:click={() => { visible = false }} on:keypress role='button' tabindex='-1'>取消</div>
    <div class='grass button' on:click={submit} on:keypress role='button' tabindex='-1'>添加</div>
  </footer>
</Dialog>
