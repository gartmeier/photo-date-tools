# photo-mtime

Set photo file modification times from the best available date source.

Date source priority:

1. EXIF `DateTimeOriginal`, `DateTimeDigitized`, or `DateTime`
2. Date in the file name, like `2024-03-12` or `20240312`
3. Parent folders in `YEAR/MONTH` form, like `2024/03`

## Usage

```sh
cargo run -- /path/to/photos
```

Multiple directories can be passed:

```sh
cargo run -- /path/to/photos /path/to/more-photos
```

## Behavior

`photo-mtime` walks each directory recursively. For each file, it finds a date and sets the file's `mtime` to that date.

EXIF dates and file/folder dates usually do not include a timezone, so dates are interpreted as local time.

Files without a parseable date are skipped.

## Build

```sh
cargo build --release
```

The binary will be at:

```sh
target/release/photo-mtime
```

Assisted by OpenAI GPT-5.5.
