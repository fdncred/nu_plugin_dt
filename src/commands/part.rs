use super::utils::{
    get_part_from_zoned_as_i16, get_unit_abbreviations, parse_datetime_string_add_nanos_optionally,
};
use crate::DtPlugin;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, SyntaxShape, Value};

pub struct DtPart;

impl SimplePluginCommand for DtPart {
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

    fn description(&self) -> &str {
        "Return the specified part of a date and time provided"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "'2017-08-25' | dt part yy",
                description: "Return the year part of the provided date string",
                result: Some(Value::test_int(2017)),
            },
            Example {
                example: "'2017-08-25T12:00:00' | dt part hh",
                description: "Return the hour part of the provided datetime string",
                result: Some(Value::test_int(12)),
            },
            Example {
                example: "2017-08-25T12:00:00 | dt part mon",
                description: "Return the month part of the provided nushell datetime",
                result: Some(Value::test_int(8)),
            },
            Example {
                example: "(date now) | dt part mon",
                description: "Return the month part of the provided nushell datetime from the date command",
                result: None,
            },
            Example {
                example: "(dt now) | dt part wk",
                description: "Return the week part of the provided datetime from the dt command",
                result: None,
            },
        ]
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time", "piece", "interval"]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let span = input.span();
        let list = call.has_flag("list")?;
        if list {
            Ok(Value::list(get_unit_abbreviations(), call.head))
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
                        // // get chrono nanoseconds
                        // let chrono_nanos = val.timestamp_nanos_opt().ok_or_else(|| {
                        //     LabeledError::new("Expected a date or datetime string".to_string())
                        // })?;
                        // // create jiff timestamp from chrono nanoseconds
                        // let jd = Timestamp::from_nanosecond(chrono_nanos as i128)
                        //     .map_err(|err| LabeledError::new(err.to_string()))?;
                        // // get the chrono timezone
                        // let tz_fixed = val.timezone();
                        // // set the jiff zoned timezone
                        // jd.to_zoned(TimeZone::fixed(tz::offset(
                        //     (tz_fixed.local_minus_utc() / 3600) as i8,
                        // )))

                        // so much easier just to output chrono as rfc 3339 and let jiff parse it

                        parse_datetime_string_add_nanos_optionally(
                            &val.to_rfc3339(),
                            None,
                            span,
                            None,
                        )?
                    }
                    Value::String { val, .. } => {
                        // eprintln!("Zoned: {:?}", zdt);
                        parse_datetime_string_add_nanos_optionally(val, None, span, None)?
                    }
                    _ => return Err(LabeledError::new("Expected a date or datetime".to_string())),
                };

                let date_part = get_part_from_zoned_as_i16(unit[0].clone(), datetime)?;

                Ok(Value::int(date_part.into(), call.head))
            }
        }
    }
}

#[test]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    use nu_plugin_test_support::PluginTest;

    // This will automatically run the examples specified in your command and compare their actual
    // output against what was specified in the example.
    //
    // We recommend you add this test to any other commands you create, or remove it if the examples
    // can't be tested this way.

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&DtPart)
}
