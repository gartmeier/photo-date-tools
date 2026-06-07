use std::{env, fs, io, path::Path, str};

use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone};
use exif::{In, Tag, Value};
use filetime::{FileTime, set_file_mtime};
use regex::Regex;
use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let file_name_re = Regex::new(
        r".*(?P<year>(?:19|20)\d{2})-?(?P<month>0[1-9]|1[0-2])-?(?P<day>0[1-9]|[12]\d|3[01]).*",
    )
    .unwrap();

    for dir_path in &args[1..] {
        for entry in WalkDir::new(dir_path) {
            let entry = entry?;
            let path = entry.path();

            if entry.file_type().is_file() {
                let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };

                let datetime = parse_datetime_from_exif(path)
                    .or_else(|| parse_datetime_from_file_name(&file_name_re, file_name))
                    .or_else(|| parse_datetime_from_parent_dirs(path));

                if let Some(datetime) = datetime {
                    set_mtime(path, datetime)?;
                }
            }
        }
    }

    Ok(())
}

fn parse_datetime_from_exif(path: &Path) -> Option<NaiveDateTime> {
    let file = fs::File::open(path).ok()?;
    let mut bufreader = io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;

    let field = exif
        .get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .or_else(|| exif.get_field(Tag::DateTimeDigitized, In::PRIMARY))
        .or_else(|| exif.get_field(Tag::DateTime, In::PRIMARY))?;

    parse_exif_datetime(&field.value)
}

fn parse_exif_datetime(value: &Value) -> Option<NaiveDateTime> {
    let Value::Ascii(values) = value else {
        return None;
    };

    let value = values.first()?;
    let value = str::from_utf8(value).ok()?.trim();
    NaiveDateTime::parse_from_str(value, "%Y:%m:%d %H:%M:%S").ok()
}

fn parse_datetime_from_file_name<'a>(re: &Regex, name: &'a str) -> Option<NaiveDateTime> {
    let caps = re.captures(name)?;

    let year = caps.name("year")?.as_str().parse::<i32>().ok()?;
    let month = caps.name("month")?.as_str().parse::<u32>().ok()?;
    let day = caps.name("day")?.as_str().parse::<u32>().ok()?;

    NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(0, 0, 0)
}

fn parse_datetime_from_parent_dirs(path: &Path) -> Option<NaiveDateTime> {
    let month = path.parent()?.file_name()?.to_str()?.parse::<u32>().ok()?;

    let year = path
        .parent()?
        .parent()?
        .file_name()?
        .to_str()?
        .parse::<i32>()
        .ok()?;

    NaiveDate::from_ymd_opt(year, month, 1)?.and_hms_opt(0, 0, 0)
}

fn set_mtime(path: &Path, datetime: NaiveDateTime) -> anyhow::Result<()> {
    let datetime = Local
        .from_local_datetime(&datetime)
        .single()
        .ok_or_else(|| anyhow::anyhow!("ambiguous local datetime: {datetime}"))?;

    let mtime = FileTime::from_unix_time(datetime.timestamp(), datetime.timestamp_subsec_nanos());
    set_file_mtime(path, mtime)?;

    Ok(())
}
