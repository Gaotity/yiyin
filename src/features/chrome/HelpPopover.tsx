import { Popover } from '../../components/Popover'

const HELP = [
  {
    title: '为什么输出的水印没有相机参数？',
    body: [
      '在界面的图片列表中，如果相机参数后面跟着一个错误图案，说明你的图片没有携带相机信息',
      '软件读取不到相机信息就无法显示出相机信息',
      '如果图片原图就没有相机信息，那么您可以进行参数自定义',
    ],
  },
  {
    title: '图片输出目录在什么地方？',
    body: ['鼠标点击图片列表下方的“打开输出目录”按钮即可打开输出目录'],
  },
  {
    title: 'Lr修图为什么导出没有相机信息？',
    body: [
      'Lr中，右键图片导出，导出界面最下面有一个元数据，选择所有元数据即可',
    ],
  },
  {
    title: 'Ps修图为什么导出没有相机信息？',
    body: [
      '在Adobe Photoshop中保留EXIF信息导出照片，您可以按照以下步骤操作：',
      '1. 打开Photoshop，并打开您想要导出的图片。',
      '2. 点击“文件”(File)菜单，然后选择“导出”(Export)。',
      '3. 在“导出”菜单中选择“导出为”(Export As...)。这将打开“导出为”对话框。',
      '4. 在“导出为”对话框中，选择您想要的图片格式。通常，如果您想保留EXIF信息，应该选择JPEG格式，因为它是唯一一种广泛支持EXIF元数据的格式。',
      '5. 在右侧的“导出选项”(Export Options)中，您可以看到一个“元数据”(Metadata)选项。点击它，您将看到几个选项：',
      '- 无元数据 (No Metadata)：这将导出不包含任何元数据的图片。',
      '- 文本和图层信息 (Text and Layer Information)：这将导出包含文本和图层信息的图片，但不包括EXIF元数据。',
      '- 所有元数据 (All Metadata)：这将导出包含所有原始元数据的图片，包括EXIF信息。',
      '6. 确保您选择了“所有元数据”(All Metadata)选项。',
      '7. 在“位置”(Location)部分，选择您想要保存图片的文件夹。',
      '8. 点击“导出”(Export)按钮，Photoshop将按照您选择的设置导出图片，并保留EXIF信息。',
      '请注意，如果您在Photoshop中对图片进行了大量编辑，可能会导致某些EXIF信息（如镜头信息、曝光时间等）不再准确反映编辑后的图片状态。此外，如果您使用的是Photoshop的“保存为Web所用格式”(Save for Web)功能或较早版本的Photoshop，保留EXIF信息的步骤可能会有所不同。',
    ],
  },
  {
    title: 'Ps修图为什么导出没有相机信息(V2)？',
    body: ['导出-web旧版'],
  },
  {
    title: '能不能多选图片？',
    body: [
      'window: 按住Ctrl去点图片',
      'Mac: 鼠标按住空白处框中 或者 按住 shift/commond 键然后点击图片',
    ],
  },
  {
    title: '为什么输出的图片小小个的？',
    body: ['Raw格式图片输出是有问题的，建议使用常用的jpg、png'],
  },
]

export function HelpPopover() {
  return (
    <Popover label="帮助" trigger="?" className="help-popover">
      <p className="popover-heading">一些常见问题解答</p>
      <div className="help-list">
        {HELP.map((item) => (
          <details key={item.title}>
            <summary>{item.title}</summary>
            <div className="help-body">
              {item.body.map((line) => (
                <p key={line}>{line}</p>
              ))}
            </div>
          </details>
        ))}
      </div>
    </Popover>
  )
}
