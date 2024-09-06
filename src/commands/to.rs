use super::utils::convert_nanos_to_nushell_datetime_value;
use crate::DtPlugin;
use jiff::Zoned;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, Value};

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
        "Return the current date and time"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time", "current"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "dt to",
            description: "Return the current date and time",
            result: None,
        }]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        // Boilerplate code
        // [ ] dt to-rfc3339
        // [ ] dt to-rfc9557
        // [ ] dt to-rfc2822
        // [ ] dt to-iso8601

        let nanos = Zoned::now().timestamp().as_nanosecond();
        convert_nanos_to_nushell_datetime_value(nanos, engine, call.head, false)
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
