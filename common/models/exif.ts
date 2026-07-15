export interface ExifData {
  Make: string
  Model: string
  LensMake: string
  LensModel: string
  ExposureTime: string
  FNumber: string
  ISO: string
  FocalLength: string
  FocalLengthIn35mmFormat: string
  ExposureProgram: string
  DateTimeOriginal: string
  ExposureCompensation: string
  MeteringMode: string
  WhiteBalance: string
}

export function emptyExifData(): ExifData {
  return {
    Make: '',
    Model: '',
    LensMake: '',
    LensModel: '',
    ExposureTime: '',
    FNumber: '',
    ISO: '',
    FocalLength: '',
    FocalLengthIn35mmFormat: '',
    ExposureProgram: '',
    DateTimeOriginal: '',
    ExposureCompensation: '',
    MeteringMode: '',
    WhiteBalance: '',
  }
}
