use super::utils::parse_datetime_string_add_nanos_optionally;
use crate::DtPlugin;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, SyntaxShape, Value};

pub struct DtAdd;

impl SimplePluginCommand for DtAdd {
    type Plugin = DtPlugin;

    fn name(&self) -> &str {
        "dt add"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "duration",
                SyntaxShape::Duration,
                "Duration to add to the provided in date and time",
            )
            .category(Category::Date)
    }

    fn description(&self) -> &str {
        "Add a duration to the provided in date and time"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time", "addition", "math"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "'2017-08-25' | dt add 1day",
                description: "Add nushell duration of 1 day to the provided date string in the local timezone",
                result: Some(Value::test_string(
                    "2017-08-26T00:00:00-05:00[America/Chicago]",
                )),
            },
            Example {
                example: "'2017-08-25T12:00:00' | dt add 1hr",
                description: "Add nushell duration of 1 hour to the provided date and time string in the local timezone",
                result: Some(Value::test_string(
                    "2017-08-25T13:00:00-05:00[America/Chicago]",
                )),
            },
            Example {
                example: "2017-08-25 | dt add 2wk",
                description: "Add nushell duration of 2 weeks to the provided nushell date in the local timezone",
                result: Some(Value::test_string(
                    "2017-09-07T19:00:00-05:00[America/Chicago]",
                )),
            },
            Example {
                example: "(dt now) | dt add 2wk",
                description: "Add nushell duration of 2 weeks to the provided dt command date in the local timezone",
                result: None,
            },
            Example {
                example: "(date now) | dt add 2wk",
                description: "Add nushell duration of 2 weeks to the provided date command date in the local timezone",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        // Currently, the duration is a Value::Duration from nushell which is limited to only a few types of durations.
        // This is a limitation of nushell and not jiff.
        // In order to use jiff, we need to create a new duration type that can be parsed by jiff.

        // From weirdan
        // If no timezone is specified, assume local tz. Provide a way to override that. Alternatively, reject dates without a timezone.

        let span = input.span();
        let duration: Value = call.req(0)?;
        let duration_nanos = match duration.as_duration() {
            Ok(duration) => duration,
            Err(_) => {
                return Err(LabeledError::new("Expected a duration".to_string()));
            }
        };

        // eprintln!("Duration: {:?}", duration_nanos);

        let datetime = match input {
            Value::Date { val, .. } => {
                // dbg!(val.timezone());
                // dbg!(val.offset());
                // dbg!(val.fixed_offset());
                // dbg!(val.to_rfc2822());
                // dbg!(val.to_rfc3339());

                // eprintln!("Date: {:?}", val);
                // let local_tz = Zoned::now().time_zone().clone();

                // // get chrono nanoseconds
                // let chrono_nanos = val.timestamp_nanos_opt().ok_or_else(|| {
                //     LabeledError::new("Expected a date or datetime string".to_string())
                // })?;
                // // create jiff timestamp from chrono nanoseconds
                // let jd = Timestamp::from_nanosecond(chrono_nanos as i128)
                //     .map_err(|err| LabeledError::new(err.to_string()))?;
                // // add the duration nanoseconds to the jiff timestamp
                // let jd_plus_nanos = jd
                //     .checked_add(duration_nanos.nanoseconds())
                //     .map_err(|err| LabeledError::new(format!("Error adding duration: {err}")))?;
                // // get the chrono timezone
                // let tz_fixed = val.timezone();
                // // eprintln!("tz_fixed: {:?}", tz_fixed);
                // // TODO: This is a little wonky. If the timezone is UTC, we set the jiff local timezone.
                // // but what happens if the user wants UTC?
                // // we should probably allow the user to specify the timezone or just always output in UTC.
                // if tz_fixed.to_string() == "+00:00" {
                //     // set the jiff local timezone
                //     jd_plus_nanos.to_zoned(local_tz)
                // } else {
                //     // set the jiff zoned timezone
                //     jd_plus_nanos.to_zoned(TimeZone::fixed(tz::offset(
                //         (tz_fixed.local_minus_utc() / 3600) as i8,
                //     )))
                // }

                // so much easier just to output chrono as rfc 3339 and let jiff parse it
                parse_datetime_string_add_nanos_optionally(
                    &val.to_rfc3339(),
                    Some(duration_nanos),
                    span,
                )?
            }
            Value::String { val, .. } => {
                // eprintln!("String: {:?}", val);
                parse_datetime_string_add_nanos_optionally(val, Some(duration_nanos), span)?
            }
            _ => {
                return Err(LabeledError::new(
                    "Expected a date or datetime in add".to_string(),
                ))
            }
        };
        Ok(Value::string(datetime.to_string(), call.head))
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

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&DtAdd)
}
