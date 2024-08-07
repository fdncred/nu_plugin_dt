use super::utils::parse_datetime_string_add_nanos_optionally;
use crate::DtPlugin;
use jiff::civil;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{record, Category, Example, LabeledError, Signature, Span, SyntaxShape, Value};

pub struct Diff;

impl SimplePluginCommand for Diff {
    type Plugin = DtPlugin;

    fn name(&self) -> &str {
        "dt diff"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "date",
                SyntaxShape::String,
                "Date to return the difference from.",
            )
            .named(
                "smallest",
                SyntaxShape::String,
                "Smallest unit to return.",
                Some('s'),
            )
            .named(
                "larges",
                SyntaxShape::String,
                "Largest unit to return.",
                Some('l'),
            )
            .switch("list", "List the unit name abbreviations", Some('l'))
            .category(Category::Date)
    }

    fn usage(&self) -> &str {
        "Return the difference between two dates or datetimes provided"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "'2017-08-25' | dt diff '2017-08-24'",
                description: "Return the year part of the provided date",
                result: Some(Value::test_int(2017)),
            },
            Example {
                example: "'2017-08-25T12:00:00' | dt diff '2017-08-24T12:00:00' --smallest dd --largest yy",
                description: "Return the hour part of the provided datetime",
                result: Some(Value::test_int(12)),
            },
        ]
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time"]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let list = call.has_flag("list")?;
        let smallest: Option<String> = call.get_flag("smallest")?;
        let largest: Option<String> = call.get_flag("largest")?;
        let input_date: String = call.req(0)?;

        if list {
            Ok(Value::list(create_abbrev_list(), call.head))
        } else {
            let datetime = match input {
                Value::Date { val, .. } => {
                    eprintln!("Date: {:?}", val);
                    return Err(LabeledError::new(
                        "Expected a date or datetime string".to_string(),
                    ));
                }
                Value::String { val, .. } => {
                    // eprintln!("Zoned: {:?}", zdt);
                    parse_datetime_string_add_nanos_optionally(val, None)?
                }
                _ => return Err(LabeledError::new("Expected a date or datetime".to_string())),
            };

            let date = match unit[0].as_ref() {
                    "year" | "yyyy" | "yy" | "yr" => datetime.year(),
                    "quarter" | "qq" | "q" | "qtr" => {
                      match datetime.month().into() {
                        1..=3 => 1,
                        4..=6 => 2,
                        7..=9 => 3,
                        10..=12 => 4,
                        _ => 0
                      }
                    }
                    "month" | "mm" | "m" | "mon" => datetime.month().into(),
                    "dayofyear" | "dy" | "y" | "doy" => datetime.day_of_year(),
                    "day" | "dd" | "d" => datetime.day().into(),
                    "week" | "ww" | "wk" | "iso_week" | "isowk" | "isoww" => {
                      let date = civil::Date::new(datetime.year(), datetime.month(), datetime.day())
                        .map_err(|err| LabeledError::new(err.to_string()))?;
                      date.to_iso_week_date().week() as i16
                    }
                    "weekday" | "wd" | "w" => datetime.weekday().to_sunday_zero_offset().into(),
                    "hour" | "hh" | "hr" => datetime.hour().into(),
                    "minute" | "mi" | "n" | "min" => datetime.minute().into(),
                    "second" | "ss" | "s" | "sec" => datetime.second().into(),
                    "millisecond" | "ms" => datetime.millisecond(),
                    "microsecond" | "mcs" | "us" => datetime.microsecond(),
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
            Ok(Value::int(date.into(), call.head))
        }
    }
}

fn create_abbrev_list() -> Vec<Value> {
    let mut records = vec![];
    let rec = Value::record(
        record! {
        "name" => Value::test_string("year"),
        "abbreviations" => Value::test_string("year, yyyy, yy, yr"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("quarter"),
        "abbreviations" => Value::test_string("quarter, qq, q, qtr"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("month"),
        "abbreviations" => Value::test_string("month, mm, m, mon"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("dayofyear"),
        "abbreviations" => Value::test_string("dayofyear, dy, y, doy"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("day"),
        "abbreviations" => Value::test_string("day, dd, d"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("week"),
        "abbreviations" => Value::test_string("week, ww, wk, iso_week, isowk, isoww"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("weekday"),
        "abbreviations" => Value::test_string("weekday, wd, w"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("hour"),
        "abbreviations" => Value::test_string("hour, hh, hr"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("minute"),
        "abbreviations" => Value::test_string("minute, mi, n, min"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("second"),
        "abbreviations" => Value::test_string("second, ss, s, sec"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("millisecond"),
        "abbreviations" => Value::test_string("millisecond, ms"),
        },
        Span::unknown(),
    );
    records.push(rec);
    let rec = Value::record(
        record! {
        "name" => Value::test_string("microsecond"),
        "abbreviations" => Value::test_string("microsecond, mcs, us"),
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

#[test]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    use nu_plugin_test_support::PluginTest;

    // This will automatically run the examples specified in your command and compare their actual
    // output against what was specified in the example.
    //
    // We recommend you add this test to any other commands you create, or remove it if the examples
    // can't be tested this way.

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&Diff)
}
