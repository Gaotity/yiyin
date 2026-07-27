import { expect, test } from '@playwright/test'

test('completes the browser-only image workflow through the fake adapter', async ({
  page,
}) => {
  await page.goto('/?platform=fake')

  await expect(page).toHaveTitle('壹印')
  await expect(page.getByRole('region', { name: '渲染设置' })).toBeVisible()
  await expect(page.getByRole('region', { name: '图片任务' })).toBeVisible()

  await page.getByRole('button', { name: '添加图片' }).click()
  await expect(page.getByText('browser-photo.jpg')).toBeVisible()

  await page.getByRole('switch', { name: '实时预览' }).click()
  await expect(page.getByRole('img', { name: '预览图' })).toHaveAttribute(
    'src',
    'yiyin://resource/browser-preview',
  )

  await page.getByRole('button', { name: '参数设置' }).click()
  await expect(page.getByRole('heading', { name: '相机参数' })).toBeVisible()
  await page.getByRole('button', { name: '关闭' }).click()

  await page.getByRole('button', { name: '模板设置' }).click()
  await expect(
    page.getByRole('heading', { name: '文本模板设置' }).first(),
  ).toBeVisible()
  await page.getByRole('button', { name: '关闭' }).click()

  await page.getByRole('button', { name: '生成印框' }).click()
  await expect(page.getByText('输出完成')).toBeVisible()
  await page.getByRole('button', { name: '选择输出目录' }).click()
  await page.getByRole('button', { name: '打开输出目录' }).click()

  await page.getByRole('button', { name: '添加图片' }).click()
  await expect(page.getByRole('alert')).toHaveText(
    'The selected file is invalid.',
  )

  await page.getByRole('button', { name: '清空' }).click()
  await expect(page.getByText('请添加图片')).toBeVisible()
})
