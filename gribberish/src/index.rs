use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};

use crate::error::GribberishError;

/// One line of a GRIB sidecar index file, locating a single message inside a
/// GRIB file along with whatever identity metadata the index format carries.
///
/// Two formats are supported:
///   * NOAA wgrib2 inventories (`.idx`): colon-delimited text lines of
///     `msg[.submsg]:offset:d=YYYYMMDDHH:VAR:level:forecast time[:extra...]`
///   * ECMWF open-data indexes (`.index`): one JSON object per line carrying
///     MARS keys plus explicit `_offset` and `_length`
///
/// An `IndexEntry` is intentionally partial compared to `MessageMetadata`: an
/// index is only authoritative about *where* messages live and roughly *what*
/// they are. Neither format carries grid shape, packing, or units — those
/// still require reading the message bytes. (cfgrib's pickled `*.idx` cache
/// files are an unrelated format and are not supported.)
#[derive(Clone, Debug, PartialEq)]
pub struct IndexEntry {
    /// 1-based message number: explicit in NOAA indexes, the line number in
    /// ECMWF indexes.
    pub message_number: usize,
    /// GRIB2 submessage number when a NOAA line reads e.g. `624.1:`.
    /// Submessages share their parent message's byte range.
    pub submessage: Option<usize>,
    /// Byte offset of the message start within the GRIB file.
    pub offset: u64,
    /// Message size in bytes. Explicit in ECMWF indexes; inferred from the
    /// next entry's offset in NOAA indexes, so it is `None` for the final
    /// entry unless the GRIB file size is supplied.
    pub length: Option<u64>,
    /// Model reference (initialization) time.
    pub reference_date: Option<DateTime<Utc>>,
    /// Variable identifier: NOAA abbreviation (`TMP`) or ECMWF MARS param (`2t`).
    pub var: Option<String>,
    /// Level, verbatim: NOAA level description (`2 m above ground`) or ECMWF
    /// `levelist` (`500`).
    pub level: Option<String>,
    /// Forecast time, verbatim: NOAA description (`3 hour fcst`, `anl`) or
    /// ECMWF `step` (`3`).
    pub forecast_time: Option<String>,
    /// Trailing NOAA fields verbatim (`ENS=+5`, probability info, ...).
    pub extra: Vec<String>,
    /// All ECMWF MARS keys verbatim (`levtype`, `stream`, `number`, ...).
    pub keys: BTreeMap<String, String>,
}

/// Parse a GRIB index file, auto-detecting NOAA `.idx` or ECMWF `.index`
/// format. `file_size` (if known) sizes the final entry of a NOAA index.
pub fn parse_index(text: &str, file_size: Option<u64>) -> Result<Vec<IndexEntry>, GribberishError> {
    match text.trim_start().chars().next() {
        Some('{') => parse_ecmwf_index(text),
        Some(_) => parse_noaa_index(text, file_size),
        None => Ok(Vec::new()),
    }
}

/// Parse a NOAA wgrib2-style `.idx` inventory. Message lengths are inferred
/// from the next entry's offset; submessage entries share their parent's
/// byte range.
pub fn parse_noaa_index(
    text: &str,
    file_size: Option<u64>,
) -> Result<Vec<IndexEntry>, GribberishError> {
    let mut entries = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let invalid =
            || GribberishError::IndexError(format!("invalid idx line {}: {line}", lineno + 1));
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 3 {
            return Err(invalid());
        }

        let (message_number, submessage) = match fields[0].split_once('.') {
            Some((msg, sub)) => (
                msg.parse().map_err(|_| invalid())?,
                Some(sub.parse().map_err(|_| invalid())?),
            ),
            None => (fields[0].parse().map_err(|_| invalid())?, None),
        };
        let offset = fields[1].parse().map_err(|_| invalid())?;
        let reference_date = fields[2].strip_prefix("d=").and_then(parse_noaa_date);

        let field = |i: usize| {
            fields
                .get(i)
                .filter(|f| !f.is_empty())
                .map(|f| f.to_string())
        };

        entries.push(IndexEntry {
            message_number,
            submessage,
            offset,
            length: None,
            reference_date,
            var: field(3),
            level: field(4),
            forecast_time: field(5),
            extra: fields[6.min(fields.len())..]
                .iter()
                .filter(|f| !f.is_empty())
                .map(|f| f.to_string())
                .collect(),
            keys: BTreeMap::new(),
        });
    }

    // A message runs from its offset to the next distinct offset (entries at
    // the same offset are submessages of one message), or to the end of file.
    let offsets: Vec<u64> = entries.iter().map(|e| e.offset).collect();
    for (i, entry) in entries.iter_mut().enumerate() {
        let end = offsets[i + 1..]
            .iter()
            .find(|&&o| o > entry.offset)
            .copied()
            .or(file_size);
        entry.length = end.map(|e| e.saturating_sub(entry.offset));
    }

    Ok(entries)
}

/// Parse an ECMWF open-data `.index` file: one JSON object per line with
/// MARS keys plus explicit `_offset`/`_length`.
pub fn parse_ecmwf_index(text: &str) -> Result<Vec<IndexEntry>, GribberishError> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(i, line)| {
            let invalid = |what: &str| {
                GribberishError::IndexError(format!("{what} in index line {}: {line}", i + 1))
            };
            let value: serde_json::Value =
                serde_json::from_str(line).map_err(|_| invalid("invalid json"))?;
            let object = value.as_object().ok_or_else(|| invalid("invalid json"))?;

            let int_field = |key: &str| match object.get(key) {
                Some(serde_json::Value::Number(n)) => n.as_u64(),
                Some(serde_json::Value::String(s)) => s.parse().ok(),
                _ => None,
            };
            let offset = int_field("_offset").ok_or_else(|| invalid("missing _offset"))?;
            let length = int_field("_length").ok_or_else(|| invalid("missing _length"))?;

            let keys: BTreeMap<String, String> = object
                .iter()
                .filter(|(k, _)| !k.starts_with('_'))
                .map(|(k, v)| {
                    let v = match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    (k.clone(), v)
                })
                .collect();

            Ok(IndexEntry {
                message_number: i + 1,
                submessage: None,
                offset,
                length: Some(length),
                reference_date: parse_ecmwf_date(keys.get("date"), keys.get("time")),
                var: keys.get("param").cloned(),
                level: keys.get("levelist").cloned(),
                forecast_time: keys.get("step").cloned(),
                extra: Vec::new(),
                keys,
            })
        })
        .collect()
}

/// `YYYYMMDDHH` with optional trailing minutes (`YYYYMMDDHHMM`).
fn parse_noaa_date(s: &str) -> Option<DateTime<Utc>> {
    if s.len() < 10 || !s.is_ascii() {
        return None;
    }
    let minute = if s.len() >= 12 {
        s[10..12].parse().ok()?
    } else {
        0
    };
    Utc.with_ymd_and_hms(
        s[0..4].parse().ok()?,
        s[4..6].parse().ok()?,
        s[6..8].parse().ok()?,
        s[8..10].parse().ok()?,
        minute,
        0,
    )
    .single()
}

/// ECMWF `date` (`20260610`) plus `time` (`0`, `1200`, `0000`, ...).
fn parse_ecmwf_date(date: Option<&String>, time: Option<&String>) -> Option<DateTime<Utc>> {
    let date = date?;
    let time = format!("{:0>4}", time.map(String::as_str).unwrap_or("0"));
    if date.len() != 8 || !date.is_ascii() || !time.is_ascii() {
        return None;
    }
    Utc.with_ymd_and_hms(
        date[0..4].parse().ok()?,
        date[4..6].parse().ok()?,
        date[6..8].parse().ok()?,
        time[0..2].parse().ok()?,
        time[2..4].parse().ok()?,
        0,
    )
    .single()
}
