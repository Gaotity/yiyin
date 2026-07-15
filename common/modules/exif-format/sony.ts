import { ExifBase } from './base'

export class SonyExif extends ExifBase {
  override Model(): string {
    return this.exif.Model.replace('ILCE-', 'α').toLowerCase()
  }
}
