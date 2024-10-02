use super::utils::{parse_datetime_string_add_nanos_optionally, ISO8601_STRICT};
use crate::DtPlugin;
use jiff::fmt::rfc2822;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{record, Category, Example, LabeledError, Signature, Value};

pub struct DtTo;

impl SimplePluginCommand for DtTo {
    type Plugin = DtPlugin;

    fn name(&self) -> &str {
        "dt to"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).category(Category::Date)
    }

    fn description(&self) -> &str {
        "Print the piped in date or datetime in various standard formats"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec![
            "date", "time", "current", "rfc3339", "rfc9557", "rfc2822", "iso8601",
        ]
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "'07/09/24' | dt to",
            description: "Print the piped in date or datetime in various standard formats",
            result: Some(Value::test_record(record! {
                "rfc9557" => Value::test_string("2024-07-09T00:00:00-05:00[America/Chicago]"),
                "rfc3339" => Value::test_string("2024-07-09T05:00:00Z"),
                "rfc2822" => Value::test_string("Tue, 9 Jul 2024 00:00:00 -0500"),
                "iso8601" => Value::test_string("2024-07-09T00:00:00-05:00"),
            })),
        }]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        _engine: &EngineInterface,
        _call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        // Boilerplate code
        // [ ] dt to-rfc3339
        // [ ] dt to-rfc9557
        // [ ] dt to-rfc2822
        // [ ] dt to-iso8601

        let span = input.span();
        let datetime = match input {
            Value::Date { val, .. } => {
                // so much easier just to output chrono as rfc 3339 and let jiff parse it
                parse_datetime_string_add_nanos_optionally(&val.to_rfc3339(), None, span, None)?
            }
            Value::String { val, .. } => {
                // eprintln!("String: {:?}", val);
                parse_datetime_string_add_nanos_optionally(val, None, span, None)?
            }
            _ => {
                return Err(LabeledError::new(
                    "Expected a date or datetime in add".to_string(),
                ))
            }
        };

        let rfc9557 = datetime.to_string();
        let rfc3339 = datetime.timestamp().to_string();
        let rfc2822 = rfc2822::to_string(&datetime)
            .map_err(|err| LabeledError::new(format!("Error converting to rfc2822: {}", err)))?;
        // let iso8601 = format!("{datetime:.0}");
        let iso8601 = datetime.strftime(ISO8601_STRICT).to_string();

        let rec = record!(
            "rfc9557" => Value::test_string(rfc9557),
            "rfc3339" => Value::test_string(rfc3339),
            "rfc2822" => Value::test_string(rfc2822),
            "iso8601" => Value::test_string(iso8601),
        );
        Ok(Value::test_record(rec))
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

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&DtTo)
}
