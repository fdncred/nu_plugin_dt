use super::utils::parse_datetime_string;
use crate::DtPlugin;
use jiff::civil;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{record, Category, Example, LabeledError, Signature, SyntaxShape, Value};

pub struct Part;

impl SimplePluginCommand for Part {
    type Plugin = DtPlugin;

    fn name(&self) -> &str {
        "dt part"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .rest(
                "unit",
                SyntaxShape::String,
                "Unit name to extract from a date/datetime.",
            )
            .switch("list", "List the unit name abbreviations", Some('l'))
            .category(Category::Date)
    }

    fn usage(&self) -> &str {
        "Return the specified part of a date and time provided"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "'2017-08-25' | dt part yy",
                description: "Return the year part of the provided date",
                result: Some(Value::test_int(2017)),
            },
            Example {
                example: "'2017-08-25T12:00:00' | dt part hh",
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
        if list {
            let rec = record! {
              "year" => Value::test_string("year, yyyy, yy"),
              "quarter" => Value::test_string("quarter, qq, q"),
              "month" => Value::test_string("month, mm, m"),
              "dayofyear" => Value::test_string("dayofyear, dy, y"),
              "day" => Value::test_string("day, dd, d"),
              "week" => Value::test_string("week, ww, wk, iso_week, isowk, isoww"),
              "weekday" => Value::test_string("weekday, dw, w"),
              "hour" => Value::test_string("hour, hh"),
              "minute" => Value::test_string("minute, mi, n"),
              "second" => Value::test_string("second, ss, s"),
              "millisecond" => Value::test_string("millisecond, ms"),
              "microsecond" => Value::test_string("microsecond, mcs"),
              "nanosecond" => Value::test_string("nanosecond, ns"),
              // "tzoffset" => Value::test_string("tzoffset, tz"),
            };

            Ok(Value::record(rec, call.head))
        } else {
            let unit: Vec<String> = call.rest(0)?;
            if unit.is_empty() {
                Err(LabeledError::new(
                    "please supply a unit name to extract from a date/datetime.",
                ))
            } else if unit.len() > 1 {
                Err(LabeledError::new(
                    "please supply only one unit name to extract from a date/datetime.",
                ))
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
                        parse_datetime_string(val)?
                    }
                    _ => return Err(LabeledError::new("Expected a date or datetime".to_string())),
                };

                let date = match unit[0].as_ref() {
                    "year" | "yyyy" | "yy" => datetime.year(),
                    "quarter" | "qq" | "q" => {
                      match datetime.month().into() {
                        1..=3 => 1,
                        4..=6 => 2,
                        7..=9 => 3,
                        10..=12 => 4,
                        _ => 0
                      }
                    }
                    "month" | "mm" | "m" => datetime.month().into(),
                    "dayofyear" | "dy" | "y" => datetime.day_of_year(),
                    "day" | "dd" | "d" => datetime.day().into(),
                    "week" | "ww" | "wk" | "iso_week" | "isowk" | "isoww" => {
                      let date = civil::Date::new(datetime.year(), datetime.month(), datetime.day())
                        .map_err(|err| LabeledError::new(err.to_string()))?;
                      date.to_iso_week_date().week() as i16
                    }
                    "weekday" | "dw" | "w" => datetime.weekday().to_sunday_zero_offset().into(),
                    "hour" | "hh" => datetime.hour().into(),
                    "minute" | "mi" | "n" => datetime.minute().into(),
                    "second" | "ss" | "s" => datetime.second().into(),
                    "millisecond" | "ms" => datetime.millisecond(),
                    "microsecond" | "mcs" => datetime.microsecond(),
                    "nanosecond" | "ns" => datetime.nanosecond(),
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
}
