use jiff::{civil, fmt::temporal::DateTimeParser, tz::TimeZone, ToSpan, Unit, Zoned};
use nu_plugin::{EngineInterface, EvaluatedCall};
use nu_protocol::{record, IntoSpanned, LabeledError, PipelineData, Span as NuSpan, Value};

// This is kind of a hack to convert jiff produced nanoseconds to Value::Date by
// converting nanos with the 'into datetime' nushell command
pub fn convert_nanos_to_nushell_datetime_value(
    nanos: i128,
    engine: &EngineInterface,
    span: NuSpan,
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
    // dbg!(s);
    // let local_now = Zoned::now();
    // let local_tz = local_now.time_zone().clone();

    // From https://github.com/BurntSushi/jiff/discussions/112#discussioncomment-10421693
    // Or alternatively, just try parsing in order of least flexible to most flexible:

    // Zoned - parse_zoned
    // Timestamp - parse_timestamp
    // DateTime - parse_datetime
    // Date - parse_date
    // Time - parse_time
    // Span
    // TimeZone
    // This is the easiest approach. And I believe it will get all cases correct. (Not 100% certain of that.)
    // But it may not necessarily be the fastest.
    static PARSER: DateTimeParser = DateTimeParser::new();

    let date_time = if let Ok(zdt) = PARSER.parse_zoned(s) {
        eprintln!("Zoned: {:?}", zdt);
        zdt
    } else if let Ok(ts) = PARSER.parse_timestamp(s) {
        eprintln!("Timestamp: {:?}", ts);
        ts.to_zoned(TimeZone::system())
    } else if let Ok(dt) = PARSER.parse_datetime(s) {
        eprint!("Datetime: {:?}", dt);
        dt.to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
    } else if let Ok(date) = PARSER.parse_date(s) {
        eprintln!("Date: {:?}", date);
        date.to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
    } else if let Ok(time) = PARSER.parse_time(s) {
        eprintln!("Time: {:?}", time);
        time.to_datetime(Zoned::now().datetime().date())
            .to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
        // } else if let Ok(span) = Span::parse(s) {
        //     return span.to_zoned(local_tz);
        // } else if let Ok(tz) = TimeZone::parse(s) {
        //     return Ok(Zoned::now().to_zoned(tz));
    } else {
        eprintln!("Could not parse datetime string: {:?}", s);
        return Err(LabeledError::new(
            "Expected a date or datetime string".to_string(),
        ));
    };

    // let zdt = strtime::parse("%a, %d %b %Y %T %z", "Mon, 15 Jul 2024 16:24:59 -0400")
    //     .map_err(|err| LabeledError::new(format!("Error parsing datetime string: {err}")))?
    //     .to_zoned()
    //     .map_err(|err2| LabeledError::new(format!("Error converting to zoned: {err2}")))?;
    // dbg!(zdt);

    // let a =
    //     BrokenDownTime::parse("%Y-%m-%d", s).map_err(|err| LabeledError::new(err.to_string()))?;
    // dbg!(a);

    // let spans = [
    //     ("P40D", 40.days()),
    //     ("P1y1d", 1.year().days(1)),
    //     ("P3dT4h59m", 3.days().hours(4).minutes(59)),
    //     ("PT2H30M", 2.hours().minutes(30)),
    //     ("P1m", 1.month()),
    //     ("P1w", 1.week()),
    //     ("P1w4d", 1.week().days(4)),
    //     ("PT1m", 1.minute()),
    //     ("PT0.0021s", 2.milliseconds().microseconds(100)),
    //     ("PT0s", 0.seconds()),
    //     ("P0d", 0.seconds()),
    //     (
    //         "P1y1m1dT1h1m1.1s",
    //         1.year()
    //             .months(1)
    //             .days(1)
    //             .hours(1)
    //             .minutes(1)
    //             .seconds(1)
    //             .milliseconds(100),
    //     ),
    // ];
    // for (string, span) in spans {
    //     let parsed: Span = string
    //         .parse()
    //         .map_err(|err| LabeledError::new(format!("{err}")))?;
    //     assert_eq!(span, parsed, "result of parsing {string:?}");
    // }

    // if len is 10 it's a date only 2024-08-09
    //   USE parse_date or parse_timestamp (appends 00:00:00)
    // if len is 8 it's time only 09:33:12
    //   USE parse_time
    // if it has 3 - or 1 + it's a datetime with a timezone
    // or if it is > 19 without . it's a datetime with a timezone
    //
    //   USE parse_timestamp (this works for date only too)
    // else
    //   USE parse_datetime

    // A parser can be created in a const context.
    // static PARSER: DateTimeParser = DateTimeParser::new();

    // // Parse a civil datetime string into a civil::DateTime.
    // let date_time = PARSER
    //     // .parse_timestamp(s)
    //     // .parse_datetime(s)
    //     .parse_zoned(s)
    //     .map_err(|err| LabeledError::new(err.to_string()))?;
    // // eprintln!("Date: {:?}", date);

    // dbg!(date_time.clone());
    // dbg!(date_time.clone().strftime("%Z"));

    // eprintln!(
    //     "BrokenDown: {}",
    //     BrokenDownTime::from(date_time)
    //         .to_string("%Z")
    //         .unwrap_or("no_tz".to_string())
    // );

    // If nanos are found, add them to the date
    if let Some(nanos) = duration_nanos {
        let date_plus_duration = date_time
            .checked_add(nanos.nanoseconds())
            .map_err(|err| LabeledError::new(err.to_string()))?;
        // eprintln!("Date + Duration: {:?}", date_plus_duration);

        // let zdt = date_plus_duration
        //     .to_zoned(local_tz)
        //     .map_err(|err| LabeledError::new(err.to_string()))?;
        // eprintln!("Zoned: {:?}", zdt);

        // let zdt = date_plus_duration.to_zoned(local_tz);
        // Ok(zdt)
        Ok(date_plus_duration)
        // Ok(date_plus_duration.to_zoned(tz))
    } else {
        // This is converting all dates to the current timezone, which is wrong
        // let zdt = date_time
        //     .to_zoned(local_tz)
        //     .map_err(|err| LabeledError::new(err.to_string()))?;

        // let zdt = date_time.to_zoned(local_tz);
        // Ok(zdt)
        Ok(date_time)
        // Ok(date_time.to_zoned(tz))
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
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("quarter"),
        "abbreviations" => Value::test_string("quarter, qq, q, qs, qtr"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("month"),
        "abbreviations" => Value::test_string("month, months, mth, mths, mm, m, mon"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("dayofyear"),
        "abbreviations" => Value::test_string("dayofyear, dy, doy"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("day"),
        "abbreviations" => Value::test_string("day, days, dd, d"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("week"),
        "abbreviations" => Value::test_string("week, weeks, ww, wk, wks, iso_week, isowk, isoww"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("weekday"),
        "abbreviations" => Value::test_string("weekday, wd, wds, w"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("hour"),
        "abbreviations" => Value::test_string("hour, hours, hh, hr, hrs"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("minute"),
        "abbreviations" => Value::test_string("minute, minutes, mi, n, min, mins"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("second"),
        "abbreviations" => Value::test_string("second, seconds, ss, s, sec, secs"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("millisecond"),
        "abbreviations" => Value::test_string("millisecond, ms, millis"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("microsecond"),
        "abbreviations" => Value::test_string("microsecond, mcs, us, micros"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
            "name" => Value::test_string("nanosecond"),
            "abbreviations" => Value::test_string("nanosecond, ns, nano, nanos"),
        },
        NuSpan::unknown(),
    );
    records.push(rec);

    records
}

pub fn create_nushelly_duration_string(span: jiff::Span) -> String {
    let mut span_vec = vec![];
    if span.get_years() > 0 {
        span_vec.push(format!("{}yrs", span.get_years()));
    }
    if span.get_months() > 0 {
        span_vec.push(format!("{}mths", span.get_months()));
    }
    // if we have more than 6 days, show weeks
    let days_span = span.get_days();
    if days_span > 6 {
        let weeks = span.get_weeks();
        if weeks == 0 {
            let (weeks, days) = (days_span / 7, days_span % 7);
            span_vec.push(format!("{}wks", weeks));
            if days > 0 {
                span_vec.push(format!("{}days", days));
            }
        } else if span.get_days() > 0 {
            span_vec.push(format!("{}days", span.get_days()));
        }
    } else if span.get_days() > 0 {
        span_vec.push(format!("{}days", span.get_days()));
    }
    if span.get_hours() > 0 {
        span_vec.push(format!("{}hrs", span.get_hours()));
    }
    if span.get_minutes() > 0 {
        span_vec.push(format!("{}mins", span.get_minutes()));
    }
    if span.get_seconds() > 0 {
        span_vec.push(format!("{}secs", span.get_seconds()));
    }
    if span.get_milliseconds() > 0 {
        span_vec.push(format!("{}ms", span.get_milliseconds()));
    }
    if span.get_microseconds() > 0 {
        span_vec.push(format!("{}µs", span.get_microseconds()));
    }
    if span.get_nanoseconds() > 0 {
        span_vec.push(format!("{}ns", span.get_nanoseconds()));
    }

    span_vec.join(" ").trim().to_string()
}

pub fn get_single_duration_unit_from_span(as_unit: Unit, span: jiff::Span) -> String {
    let span_str = match as_unit {
        Unit::Year => format!("{}yrs", span.get_years()),
        Unit::Month => format!("{}mths", span.get_months()),
        Unit::Week => format!("{}wks", span.get_weeks()),
        Unit::Day => format!("{}days", span.get_days()),
        Unit::Hour => format!("{}hrs", span.get_hours()),
        Unit::Minute => format!("{}mins", span.get_minutes()),
        Unit::Second => format!("{}secs", span.get_seconds()),
        Unit::Millisecond => format!("{}ms", span.get_milliseconds()),
        Unit::Microsecond => format!("{}µs", span.get_microseconds()),
        Unit::Nanosecond => format!("{}ns", span.get_nanoseconds()),
    };
    span_str
}
