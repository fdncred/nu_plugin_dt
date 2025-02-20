use jiff::{
    civil,
    //fmt::friendly::{Designator, Spacing, SpanPrinter},
    fmt::temporal::{DateTimeParser, Pieces},
    tz,
    tz::{OffsetConflict, TimeZone},
    Span as JiffSpan,
    Timestamp,
    ToSpan,
    Unit,
    Zoned,
};
use nu_plugin::{EngineInterface, EvaluatedCall};
use nu_protocol::{
    record, IntoSpanned, LabeledError, PipelineData, Span as NuSpan, Spanned, Value,
};

// Attribution: Borrowed these formats from here
// https://github.com/BurntSushi/gitoxide/blob/25a3f1b0b07c01dd44df254f46caa6f78a4d3014/gix-date/src/time/format.rs

// E.g. `2022-08-17T21:43:13+08:00`
pub const ISO8601_STRICT: &str = "%Y-%m-%dT%H:%M:%S%:z";
// E.g. `2022-08-17T21:43:13.123456789+08:00`
pub const ISO8601_STRICT_WITH_FRACTIONAL: &str = "%Y-%m-%dT%H:%M:%S%.f%:z";
// E.g. `Thu, 18 Aug 2022 12:45:06 +0800`
pub const RFC2822: &str = "%a, %d %b %Y %H:%M:%S %z";
// E.g. `Thu, 18 Aug 2022 12:45:06 +0800`. This is output by `git log --pretty=%aD`.
pub const GIT_RFC2822: &str = "%a, %-d %b %Y %H:%M:%S %z";
// E.g. `Thu Sep 04 2022 10:45:06 -0400`, like the git `DEFAULT`, but with the year and time fields swapped.
pub const GITOXIDE: &str = "%a %b %d %Y %H:%M:%S %z";
// E.g. `Thu Sep 4 10:45:06 2022 -0400`. This is output by `git log --pretty=%ad`.
pub const GITLOG_DEFAULT: &str = "%a %b %-d %H:%M:%S %Y %z";
// E.g. `2018-7-9` or `2018-07-09`
pub const SHORT_DATE: &str = "%Y-%m-%d";
// E.g. `7/9/24`` or `07/09/24`
pub const SHORT_DATE_USA_2YEAR: &str = "%m/%d/%y";
// E.g. `7/9/2024` or `07/09/2024`
pub const SHORT_DATE_USA_4YEAR: &str = "%m/%d/%Y";

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

pub fn parse_datetime_string_into_pieces(
    s: &str,
    duration_nanos: Option<i64>,
    span: NuSpan,
    jiff_span: Option<JiffSpan>,
) -> Result<Zoned, LabeledError> {
    let system_tz = TimeZone::system();
    // let timestamp = "2024-06-14T17:30-05[America/New_York]";
    let timestamp = s;
    // The default for conflict resolution when parsing into a `Zoned` is
    // actually `Reject`, but we use `AlwaysOffset` here to show a different
    // strategy. You'll want to pick the conflict resolution that suits your
    // needs. The `Reject` strategy is what you should pick if you aren't
    // sure.
    let conflict_resolution = OffsetConflict::AlwaysOffset;

    let pieces = Pieces::parse(timestamp).map_err(|err| {
        LabeledError::new(err.to_string()).with_label(
            format!("Could not parse pieces datetime string: {:?}", s),
            span,
        )
    })?;
    eprintln!("pieces: {:#?}", pieces);
    let time = pieces.time().unwrap_or_else(jiff::civil::Time::midnight);
    eprintln!("time: {:#?}", time);
    let dt = pieces.date().to_datetime(time);
    eprintln!("dt: {:#?}", dt);
    // let dt_wz = pieces.to_time_zone().unwrap_or_else(|_| Some(system_tz));
    let binding = system_tz.clone();
    let tz_name = binding.iana_name().unwrap_or("America/Chicago");
    let pieces = pieces.with_time_zone_name(tz_name);
    let ambiguous_zdt = match pieces.to_time_zone().unwrap_or_else(|_| Some(system_tz)) {
        Some(tz) => match pieces.to_numeric_offset() {
            None => tz.into_ambiguous_zoned(dt),
            Some(offset) => conflict_resolution.resolve(dt, offset, tz).map_err(|err| {
                LabeledError::new(err.to_string()).with_label(
                    format!(
                        "Could not parse conflict resolution for datetime string: {:?}",
                        s
                    ),
                    span,
                )
            })?,
        },
        None => {
            eprintln!("None: No time zone or offset found");
            let Some(offset) = pieces.to_numeric_offset() else {
                let msg = format!(
                    "timestamp `{timestamp}` has no time zone \
                 or offset, and thus cannot be parsed into \
                 an instant",
                );
                return Err(LabeledError::new(msg).with_label(
                    format!("Could not parse numeric offset of datetime string: {:?}", s),
                    span,
                ));
            };
            // Won't even be ambiguous, but gets us the same
            // type as the branch above.
            TimeZone::fixed(offset).into_ambiguous_zoned(dt)
        }
    };
    // We do compatible disambiguation here like we do in the previous
    // examples, but you could choose any strategy. As with offset conflict
    // resolution, if you aren't sure what to pick, a safe choice here would
    // be `ambiguous_zdt.unambiguous()`, which will return an error if the
    // datetime is ambiguous in any way. Then, if you ever hit an error, you
    // can examine the case to see if it should be handled in a different way.
    let zdt = ambiguous_zdt
        .compatible()
        .map_err(|err| LabeledError::new(err.to_string()))?;
    // Notice that we now have a different civil time and offset, but the
    // instant it corresponds to is the same as the one we started with.
    // assert_eq!(
    //     zdt.to_string(),
    //     "2024-06-14T18:30:00-04:00[America/New_York]"
    // );
    let date_time = zdt;
    eprintln!("date_time: {}", date_time);
    if let Some(nanos) = duration_nanos {
        let date_plus_duration = date_time
            .checked_add(nanos.nanoseconds())
            .map_err(|err| LabeledError::new(err.to_string()))?;
        Ok(date_plus_duration)
        // Ok(date_plus_duration.to_zoned(tz))
    } else if let Some(jiff_span) = jiff_span {
        let zdt2 = date_time
            .checked_add(jiff_span)
            .map_err(|err| LabeledError::new(err.to_string()))?;
        Ok(zdt2)
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

    // Ok(zdt)
}

// Parse a string into a jiff datetime and add nanoseconds to it optionally
pub fn parse_datetime_string_add_nanos_optionally(
    s: &str,
    duration_nanos: Option<i64>,
    span: NuSpan,
    jiff_span: Option<JiffSpan>,
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
    static PARSER: DateTimeParser =
        DateTimeParser::new().offset_conflict(tz::OffsetConflict::PreferOffset);

    // PARSER
    //     .parse_date(s)
    //     .map_err(|err| LabeledError::new(err.to_string()))?;

    // let z =
    //     civil::Date::strptime(SHORT_DATE, s).map_err(|err| LabeledError::new(err.to_string()))?;
    // dbg!(z);

    let see_debug_values = false;

    let date_time = if let Ok(date) = civil::Date::strptime(SHORT_DATE, s) {
        if see_debug_values {
            eprintln!("civil Date (SHORT_DATE): {:?}", date);
        }
        date.to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
    } else if let Ok(date) = civil::Date::strptime(SHORT_DATE_USA_2YEAR, s) {
        if see_debug_values {
            eprintln!("civil Date USA 2yr (SHORT_DATE_USA_2YEAR): {:?}", date);
        }
        date.to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
    } else if let Ok(date) = civil::Date::strptime(SHORT_DATE_USA_4YEAR, s) {
        if see_debug_values {
            eprintln!("civil Date USA 4yr (SHORT_DATE_USA_4YEAR): {:?}", date);
        }
        date.to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
    } else if let Ok(zdt) = PARSER.parse_zoned(s) {
        if see_debug_values {
            eprintln!("Zoned: {:?}", zdt);
        }
        zdt
    } else if let Ok(iso) = strptime_relaxed(ISO8601_STRICT, s) {
        if see_debug_values {
            eprintln!("ISO8601_STRICT: {:?}", iso);
        }
        iso
    } else if let Ok(iso) = strptime_relaxed(ISO8601_STRICT_WITH_FRACTIONAL, s) {
        if see_debug_values {
            eprintln!("ISO8601_STRICT_WITH_FRACTIONAL: {:?}", iso);
        }
        iso
    } else if let Ok(rfc) = strptime_relaxed(RFC2822, s) {
        if see_debug_values {
            eprintln!("RFC2822: {:?}", rfc);
        }
        rfc
    } else if let Ok(gitrfc) = strptime_relaxed(GIT_RFC2822, s) {
        if see_debug_values {
            eprintln!("GIT_RFC2822: {:?}", gitrfc);
        }
        gitrfc
    } else if let Ok(gitox) = strptime_relaxed(GITOXIDE, s) {
        if see_debug_values {
            eprintln!("GITOXIDE: {:?}", gitox);
        }
        gitox
    } else if let Ok(gitlog) = strptime_relaxed(GITLOG_DEFAULT, s) {
        if see_debug_values {
            eprintln!("GITLOG_DEFAULT: {:?}", gitlog);
        }
        gitlog
    } else if let Ok(ts) = PARSER.parse_timestamp(s) {
        if see_debug_values {
            eprintln!("Timestamp: {:?}", ts);
        }
        ts.to_zoned(TimeZone::system())
    } else if let Ok(dt) = PARSER.parse_datetime(s) {
        if see_debug_values {
            eprintln!("Datetime: {:?}", dt);
        }
        dt.to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
    } else if let Ok(date) = PARSER.parse_date(s) {
        if see_debug_values {
            eprintln!("Date: {:?}", date);
        }
        date.to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
    } else if let Ok(time) = PARSER.parse_time(s) {
        if see_debug_values {
            eprintln!("Time: {:?}", time);
        }
        time.to_datetime(Zoned::now().datetime().date())
            .to_zoned(TimeZone::system())
            .map_err(|err| LabeledError::new(err.to_string()))?
        // } else if let Ok(span) = Span::parse(s) {
        //     return span.to_zoned(local_tz);
        // } else if let Ok(tz) = TimeZone::parse(s) {
        //     return Ok(Zoned::now().to_zoned(tz));
    } else {
        return Err(
            LabeledError::new("Expected a date or datetime string in utils".to_string())
                .with_label(format!("Could not parse datetime string: {:?}", s), span),
        );
    };

    if see_debug_values {
        eprintln!("After Parsing Zoned: {:?}\n", date_time.clone());
    }

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
    } else if let Some(jiff_span) = jiff_span {
        let zdt = date_time
            .checked_add(jiff_span)
            .map_err(|err| LabeledError::new(err.to_string()))?;
        Ok(zdt)
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
            date.iso_week_date().week() as i16
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

#[allow(dead_code)]
pub fn parse_number_and_unit_string(
    input_value: Spanned<String>,
) -> Result<(i64, String), LabeledError> {
    let mut number_part = String::new();
    let mut string_part = String::new();

    let input_str = input_value.item;

    for (i, c) in input_str.char_indices() {
        if c.is_ascii_digit() {
            number_part.push(c);
        } else {
            string_part = input_str[i..].to_string();
            break;
        }
    }

    let number = number_part.parse::<i64>().map_err(|err| {
        LabeledError::new(format!("Could not parse number from string: {err}",))
            .with_label(format!("string input was {input_str}"), input_value.span)
    })?;

    Ok((number, string_part))
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
    // jiff's friendly format
    // format!("{span:#}")
    // with some configuration
    // Nushell               - 5yrs 2mths 3wks 6days 21hrs 37mins 30secs 367ms 322µs 100ns
    // Designator::Compact   - 5y 2mo 27d 21h 37m 30s 367ms 322µs 100ns
    // Designator::HumanTime - 5y 2months 27d 21h 37m 30s 367ms 322us 100ns
    // Designator::Short     - 5yrs 2mos 27days 21hrs 37mins 30secs 367msecs 322µsecs 100nsecs
    // Designator::Verbose   - 5years 2months 27days 21hours 37minutes 30seconds 367milliseconds 322microseconds 100nanoseconds
    //
    // let printer = SpanPrinter::new().designator(Designator::Verbose);
    // printer.span_to_string(&span)

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
    match as_unit {
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
    }
}

fn strptime_relaxed(fmt: &str, input: &str) -> Result<Zoned, jiff::Error> {
    let mut tm = jiff::fmt::strtime::parse(fmt, input)?;
    tm.set_weekday(None);
    tm.to_zoned()
}

pub fn unix_timestamp_in_seconds_to_local_zoned(
    unix_timestamp: i64,
) -> Result<String, LabeledError> {
    // Convert the Unix timestamp (in seconds) to a Jiff Timestamp (in nanoseconds)
    let timestamp = Timestamp::from_second(unix_timestamp)
        .map_err(|err| LabeledError::new(format!("Error converting Unix timestamp: {:?}", err)))?;

    // Format the timestamp as a datetime string without timezone
    let datetime = timestamp.strftime("%Y-%m-%d %H:%M:%S");

    Ok(datetime.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_protocol::Span as NuSpan;

    #[test]
    fn test_get_part_from_zoned_as_i16() {
        let datetime = Zoned::now();
        let part_string = "year".to_string();

        let result = get_part_from_zoned_as_i16(part_string, datetime.clone());
        assert!(result.is_ok());
        let year = result.unwrap();
        assert_eq!(year, datetime.year() as i16);
    }

    #[test]
    fn test_get_unit_from_unit_string() {
        let unit_name = "year".to_string();

        let result = get_unit_from_unit_string(unit_name);
        assert!(result.is_ok());
        let unit = result.unwrap();
        assert_eq!(unit, Unit::Year);
    }

    #[test]
    fn test_parse_number_and_unit_string() {
        let input_value = Spanned {
            item: "10days".to_string(),
            span: NuSpan::unknown(),
        };

        let result = parse_number_and_unit_string(input_value);
        assert!(result.is_ok());
        let (number, unit) = result.unwrap();
        assert_eq!(number, 10);
        assert_eq!(unit, "days");
    }

    #[test]
    fn test_get_unit_abbreviations() {
        let abbreviations = get_unit_abbreviations();
        assert_eq!(abbreviations.len(), 13);
    }

    #[test]
    fn test_create_nushelly_duration_string() {
        let span = jiff::Span::new()
            .years(1)
            .months(2)
            .days(3)
            .hours(4)
            .minutes(5)
            .seconds(6);
        let result = create_nushelly_duration_string(span);
        assert_eq!(result, "1yrs 2mths 3days 4hrs 5mins 6secs");
    }

    #[test]
    fn test_get_single_duration_unit_from_span() {
        let span = jiff::Span::new().years(1);
        let result = get_single_duration_unit_from_span(Unit::Year, span);
        assert_eq!(result, "1yrs");
    }

    #[test]
    fn test_strptime_relaxed() {
        let fmt = "%Y-%m-%dT%H:%M:%S%:z";
        let input = "2022-01-01T00:00:00+00:00";

        let result = strptime_relaxed(fmt, input);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.year(), 2022);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.day(), 1);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case1() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = Some(1_000_000_000); // 1 second
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 1);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case2() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = Some(60_000_000_000); // 1 minute
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.minute(), 1);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case3() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = Some(3_600_000_000_000); // 1 hour
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.hour(), 1);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case4() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = Some(86_400_000_000_000); // 1 day
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.day(), 2);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case5() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = None;
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 0);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case6() {
        let s = "2022-01-01";
        let duration_nanos = None;
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 0);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case7() {
        let s = "2022-01-01";
        let duration_nanos = Some(86_400_000_000_000); // 1 day
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.day(), 2);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case8() {
        let s = "2022-01-01T00:00:00.123456789+00:00";
        let duration_nanos = None;
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.hour(), 0);
        assert_eq!(datetime.minute(), 0);
        assert_eq!(datetime.second(), 0);
        assert_eq!(datetime.millisecond(), 123);
        assert_eq!(datetime.microsecond(), 456);
        assert_eq!(datetime.nanosecond(), 789);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case9() {
        let s = "2022-01-01T00:00:00.123456789+00:00";
        let duration_nanos = Some(1_000_000_000); // 1 second
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();

        assert_eq!(datetime.second(), 1);
        assert_eq!(datetime.nanosecond(), 789);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case10() {
        let s = "Thu, 18 Aug 2022 12:45:06 +0800";
        let duration_nanos = None;
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 6);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case11() {
        let s = "Thu, 18 Aug 2022 12:45:06 +0800";
        let duration_nanos = Some(1_000_000_000); // 1 second
        let span = NuSpan::unknown();

        let result = parse_datetime_string_add_nanos_optionally(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 7);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case1b() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = Some(1_000_000_000); // 1 second
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 1);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case2b() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = Some(60_000_000_000); // 1 minute
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.minute(), 1);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case3b() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = Some(3_600_000_000_000); // 1 hour
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.hour(), 1);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case4b() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = Some(86_400_000_000_000); // 1 day
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.day(), 2);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case5b() {
        let s = "2022-01-01T00:00:00+00:00";
        let duration_nanos = None;
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 0);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case6b() {
        let s = "2022-01-01";
        let duration_nanos = None;
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        eprint!("result: {:#?}", result);
        assert!(result.is_ok());

        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 0);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case7b() {
        let s = "2022-01-01";
        let duration_nanos = Some(86_400_000_000_000); // 1 day
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.day(), 2);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case8b() {
        let s = "2022-01-01T00:00:00.123456789+00:00";
        let duration_nanos = None;
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.hour(), 0);
        assert_eq!(datetime.minute(), 0);
        assert_eq!(datetime.second(), 0);
        assert_eq!(datetime.millisecond(), 123);
        assert_eq!(datetime.microsecond(), 456);
        assert_eq!(datetime.nanosecond(), 789);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case9b() {
        let s = "2022-01-01T00:00:00.123456789+00:00";
        let duration_nanos = Some(1_000_000_000); // 1 second
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();

        assert_eq!(datetime.second(), 1);
        assert_eq!(datetime.nanosecond(), 789);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case10b() {
        let s = "Thu, 18 Aug 2022 12:45:06 +0800";
        let duration_nanos = None;
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 6);
    }

    #[test]
    fn test_parse_datetime_string_add_nanos_optionally_case11b() {
        let s = "Thu, 18 Aug 2022 12:45:06 +0800";
        let duration_nanos = Some(1_000_000_000); // 1 second
        let span = NuSpan::unknown();

        let result = parse_datetime_string_into_pieces(s, duration_nanos, span, None);
        assert!(result.is_ok());
        let datetime = result.unwrap();
        assert_eq!(datetime.second(), 7);
    }
}
