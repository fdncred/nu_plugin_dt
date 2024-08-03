use super::utils::parse_datetime_string_add_nanos;
use crate::DtPlugin;
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

                let zdt = parse_datetime_string_add_nanos(val, duration_nanos)?;
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
