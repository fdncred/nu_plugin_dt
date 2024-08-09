use jiff::{civil, fmt::temporal::DateTimeParser, ToSpan, Unit, Zoned};
use nu_plugin::{EngineInterface, EvaluatedCall};
use nu_protocol::{record, IntoSpanned, LabeledError, PipelineData, Span, Value};

// This is kind of a hack to convert jiff produced nanoseconds to Value::Date by
// converting nanos with the 'into datetime' nushell command
pub fn convert_nanos_to_nushell_datetime_value(
    nanos: i128,
    engine: &EngineInterface,
    span: Span,
    utc: bool,
) -> Result<Value, LabeledError> {
    let Some(decl_id) = engine.find_decl("into datetime")? else {
        return Err(LabeledError::new(
            "Could not find 'into datetime' declaration".to_string(),
        ));
    };
    let into_datetime = engine.call_decl(
        decl_id,
        if utc {
            EvaluatedCall::new(span)
                .with_named("timezone".into_spanned(span), Value::string("UTC", span))
        } else {
            EvaluatedCall::new(span)
                .with_named("timezone".into_spanned(span), Value::string("LOCAL", span))
        },
        PipelineData::Value(Value::int(nanos as i64, span), None),
        true,
        false,
    )?;
    let datetime = into_datetime.into_value(span)?;
    Ok(datetime)
}

// Parse a string into a jiff datetime and add nanoseconds to it optionally
pub fn parse_datetime_string_add_nanos_optionally(
    s: &str,
    duration_nanos: Option<i64>,
) -> Result<Zoned, LabeledError> {
    let cur_date_time_zone = Zoned::now();
    let tz = cur_date_time_zone.time_zone().clone();

    // A parser can be created in a const context.
    static PARSER: DateTimeParser = DateTimeParser::new();

    // Parse a civil datetime string into a civil::DateTime.
    let date_time = PARSER
        .parse_datetime(s)
        .map_err(|err| LabeledError::new(err.to_string()))?;
    // eprintln!("Date: {:?}", date);

    // If nanos are found, add them to the date
    if let Some(nanos) = duration_nanos {
        let date_plus_duration = date_time
            .checked_add(nanos.nanoseconds())
            .map_err(|err| LabeledError::new(err.to_string()))?;
        // eprintln!("Date + Duration: {:?}", date_plus_duration);

        let zdt = date_plus_duration
            .to_zoned(tz)
            .map_err(|err| LabeledError::new(err.to_string()))?;
        // eprintln!("Zoned: {:?}", zdt);

        Ok(zdt)
    } else {
        let zdt = date_time
            .to_zoned(tz)
            .map_err(|err| LabeledError::new(err.to_string()))?;
        Ok(zdt)
    }
}

pub fn get_part_from_zoned_as_i16(
    part_string: String,
    datetime: Zoned,
) -> Result<i16, LabeledError> {
    let date = match part_string.as_ref() {
        "year" | "years" | "yyyy" | "yy" | "yr" | "yrs" => datetime.year(),
        "quarter" | "qq" | "q" | "qs" | "qtr" => {
            match datetime.month().into() {
            1..=3 => 1,
            4..=6 => 2,
            7..=9 => 3,
            10..=12 => 4,
            _ => 0
            }
        }
        "month" | "months" | "mth" | "mths" | "mm" | "m" | "mon" => datetime.month().into(),
        "dayofyear" | "dy" | "doy" => datetime.day_of_year(),
        "day" | "days" | "dd" | "d" => datetime.day().into(),
        "week" | "weeks" | "ww" | "wk" | "wks" | "iso_week" | "isowk" | "isoww" => {
            let date = civil::Date::new(datetime.year(), datetime.month(), datetime.day())
            .map_err(|err| LabeledError::new(err.to_string()))?;
            date.to_iso_week_date().week() as i16
        }
        "weekday" | "wd" | "wds" | "w" => datetime.weekday().to_sunday_zero_offset().into(),
        "hour" | "hours" | "hh" | "hr" | "hrs" => datetime.hour().into(),
        "minute" | "minutes" | "mi" | "n" | "min" | "mins" => datetime.minute().into(),
        "second" | "seconds" | "ss" | "s" | "sec" | "secs" => datetime.second().into(),
        "millisecond" | "ms" | "millis" => datetime.millisecond(),
        "microsecond" | "mcs" | "us" | "micros" => datetime.microsecond(),
        "nanosecond" | "ns" | "nano" | "nanos" => datetime.nanosecond(),
        // TODO: Fix this
        // Not sure there's a way to return an tz as an i16
        // "tzoffset" | "tz" => datetime.offset().seconds().try_into().unwrap(),
        _ => {
            return Err(LabeledError::new(
                "please supply a valid unit name to extract from a date/datetime. see dt part --list for list of abbreviations.",
            ))
        }
    };

    Ok(date)
}

pub fn get_unit_from_unit_string(unit_name: String) -> Result<Unit, LabeledError> {
    let unit = match unit_name.as_ref() {
        "year" | "years" | "yyyy" | "yy" | "yr" | "yrs" => Ok(Unit::Year),
        "month" | "months" | "mth" | "mths" | "mm" | "m" | "mon" => Ok(Unit::Month),
        "day" | "days" | "dd" | "d" => Ok(Unit::Day),
        "week" | "weeks" | "ww" | "wk" | "wks" | "iso_week" | "isowk" | "isoww" => Ok(Unit::Week),
        "hour" | "hours" | "hh" | "hr" | "hrs" => Ok(Unit::Hour),
        "minute" | "minutes" | "mi" | "n" | "min" | "mins" => Ok(Unit::Minute),
        "second" | "seconds" | "ss" | "s" | "sec" | "secs" => Ok(Unit::Second),
        "millisecond" | "ms" | "millis" => Ok(Unit::Millisecond),
        "microsecond" | "mcs" | "us" | "micros" => Ok(Unit::Microsecond),
        "nanosecond" | "ns" | "nano" | "nanos" => Ok(Unit::Nanosecond),
        _ => {
            return Err(LabeledError::new(
                "please supply a valid unit name to extract from a date/datetime. see dt part --list for list of abbreviations.",
            ))
        }
    };

    unit
}

pub fn get_unit_abbreviations() -> Vec<Value> {
    let mut records = vec![];
    let rec = Value::record(
        record! {
        "name" => Value::test_string("year"),
        "abbreviations" => Value::test_string("year, years, yyyy, yy, yr, yrs"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("quarter"),
        "abbreviations" => Value::test_string("quarter, qq, q, qs, qtr"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("month"),
        "abbreviations" => Value::test_string("month, months, mth, mths, mm, m, mon"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("dayofyear"),
        "abbreviations" => Value::test_string("dayofyear, dy, doy"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("day"),
        "abbreviations" => Value::test_string("day, days, dd, d"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("week"),
        "abbreviations" => Value::test_string("week, weeks, ww, wk, wks, iso_week, isowk, isoww"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("weekday"),
        "abbreviations" => Value::test_string("weekday, wd, wds, w"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("hour"),
        "abbreviations" => Value::test_string("hour, hours, hh, hr, hrs"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("minute"),
        "abbreviations" => Value::test_string("minute, minutes, mi, n, min, mins"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("second"),
        "abbreviations" => Value::test_string("second, seconds, ss, s, sec, secs"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("millisecond"),
        "abbreviations" => Value::test_string("millisecond, ms, millis"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("microsecond"),
        "abbreviations" => Value::test_string("microsecond, mcs, us, micros"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
            "name" => Value::test_string("nanosecond"),
            "abbreviations" => Value::test_string("nanosecond, ns, nano, nanos"),
        },
        Span::unknown(),
    );
    records.push(rec);

    records
}
