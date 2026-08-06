# Yiyin (壹印)

A desktop app that adds EXIF-based watermark frames to photos. Rust owns all product behavior (clean architecture: domain / application / infrastructure / Tauri adapter); React is display-only.

## Language

### Metadata

**Metadata**:
The display-ready EXIF information of a registered image — a fixed set of 15 built-in fields plus orientation and density. Numeric and enum fields cross the application boundary display-normalized at read time; make/model strings cross raw and are vendor-normalized by the renderer at render time.
_Avoid_: EXIF blob, tags, raw metadata

**Raw extracted value**:
A value exactly as a metadata source (EXIF tag, future XMP/ffprobe source) reports it — unformatted, vendor-unwashed. Only make/model strings leave the metadata adapter raw; every other value is display-normalized inside it at read time.
_Avoid_: raw EXIF, tag value

**Normalized display value**:
The canonical string form of a metadata field after domain-owned normalization rules (vendor name washing, Nikon `ℤ`/Roman numerals, Sony `α`, shutter `1/N` — sub-second exposures above 2/3s render as trimmed decimal seconds like `0.7` — etc.). The rules live in `yiyin-domain`: the metadata adapter applies the numeric and enum ones at read time, the renderer applies the vendor make/model ones at render time, and this is what templates render. Moving all normalization behind the application layer is the deferred ENG-105 follow-up.
_Avoid_: formatted EXIF, display string

### Configuration

**Output directory**:
The filesystem location exports are written to (default `Pictures/watermark`). A domain value object — never a bare string. The UI can choose it only through the native dialog; it never travels through the general config-update path or any DTO.
_Avoid_: output path, save folder, `output` string

**Numeric constraints**:
The minimum/maximum/decimals policy for the numeric render options, owned by the domain's bounded value objects and shipped to the frontend through the bootstrap payload. The frontend consumes them for clamping — a UX responsibility — and never maintains its own bounds table.
_Avoid_: hand-copied bounds, clamp constants, settings table bounds

**Background ratio guide coupling**:
An active background ratio guide forces landscape output off. The domain owns the rule (`Config::normalize`) and silently normalizes it on every config write path; the frontend mirrors it in the `setBackgroundRatioVisible` edit intent so the UI never displays the contradictory combination.
_Avoid_: ratio-guide checks in UI handlers, coupled toggle

### Templates

**Template**:
An ordered line of text with `{FieldKey}` placeholders, rendered at the bottom of the frame. System templates are built-in and cannot be deleted; custom templates can be added, edited, disabled, reordered, and deleted.
_Avoid_: text line, watermark text

**Template field**:
A single metadata-backed value usable in templates — with visibility, an optional user override of the extracted value, light/dark image variants, and font overrides. Hidden or empty fields collapse out of the render entirely (no dangling placeholders, no empty lines).
_Avoid_: parameter, EXIF field

### Tasks and rendering

**Task**:
One registered image moving through the export lifecycle: `registered → queued → running → completed | failed | cancelled`. Illegal transitions are unrepresentable.
_Avoid_: job, export item, queue entry

**Frozen render request**:
The immutable snapshot of configuration, templates, resources, and normalized metadata taken when a task starts or previews. Later configuration edits never affect an in-flight request.
_Avoid_: render config copy, job payload

**Preview**:
A throwaway quality-70 render of the selected task, shown in-app via a resource URL. Never written to the output directory.
_Avoid_: draft, thumbnail, temp render

**Preview supersession**:
Starting a new preview cancels the previous one. A superseded preview that finishes late can never overwrite the newer preview.
_Avoid_: preview race, stale preview

**Opaque resource id**:
A runtime capability handle (UUID) that the UI uses to reference an image, font, logo, or preview — resolved through the registry on every request. It is not a path, not guessable, and not valid across restarts.
_Avoid_: resource path, file URL, asset id
