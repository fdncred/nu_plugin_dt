use crate::DtPlugin;
use jiff::{fmt::temporal::DateTimeParser, ToSpan, Zoned};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, SyntaxShape, Value};

pub struct Add;

impl SimplePluginCommand for Add {
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

    fn usage(&self) -> &str {
        "Add a duration to the provided in date and time"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "2017-08-25 | dt add 1day",
                description: "Add 1 day to the provided date",
                result: Some(Value::test_string(
                    "2017-08-26T00:00:00-05:00[America/Chicago]",
                )),
            },
            Example {
                example: "2017-08-25T12:00:00 | dt add 1hr",
                description: "Add 1 hour to the provided date and time",
                result: Some(Value::test_string(
                    "2017-08-25T13:00:00-05:00[America/Chicago]",
                )),
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

        let duration: Value = call.req(0)?;
        let duration_nanos = match duration.as_duration() {
            Ok(duration) => duration,
            Err(_) => {
                return Err(LabeledError::new("Expected a duration".to_string()));
            }
        };

        // eprintln!("Duration: {:?}", duration_nanos);

        match input {
            Value::Date { val, .. } => {
                eprintln!("Date: {:?}", val);
                return Err(LabeledError::new(
                    "Expected a date or datetime string".to_string(),
                ));
            }
            Value::String { val, .. } => {
                // eprintln!("String: {:?}", val);

                let cur_date_time_zone = Zoned::now();
                let tz = cur_date_time_zone.time_zone().clone();
                // let tz_name = tz.iana_name().unwrap_or_default();
                // let dt_with_tz = format!("{}[{}]", val, tz_name);

                // A parser can be created in a const context.
                static PARSER: DateTimeParser = DateTimeParser::new();
                // let date = PARSER
                //     .parse_zoned(dt_with_tz)
                //     .map_err(|err| LabeledError::new(err.to_string()))?;
                // let zdt = date.with_time_zone(tz);

                // Parse a civil datetime string into a civil::DateTime.
                let date = PARSER
                    .parse_datetime(val)
                    .map_err(|err| LabeledError::new(err.to_string()))?;
                // eprintln!("Date: {:?}", date);

                let date_plus_duration = date
                    .checked_add(duration_nanos.nanoseconds())
                    .map_err(|err| LabeledError::new(err.to_string()))?;
                // eprintln!("Date + Duration: {:?}", date_plus_duration);

                let zdt = date_plus_duration
                    .to_zoned(tz)
                    .map_err(|err| LabeledError::new(err.to_string()))?;
                // eprintln!("Zoned: {:?}", zdt);
                Ok(Value::string(zdt.to_string(), call.head))
            }
            _ => return Err(LabeledError::new("Expected a date or datetime".to_string())),
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

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&Add)
}
