use crate::DtPlugin;
use jiff::Zoned;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, Value};

pub struct UtcNow;

impl SimplePluginCommand for UtcNow {
    type Plugin = DtPlugin;

    fn name(&self) -> &str {
        "dt utcnow"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).category(Category::Date)
    }

    fn usage(&self) -> &str {
        "Return the current date and time in UTC"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "dt utcnow",
            description: "Return the current date and time",
            result: None,
        }]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let now = Zoned::now();
        let nowutc = now
            .intz("UTC")
            .map_err(|err| LabeledError::new(err.to_string()))?;
        //FIXME: We can't return a Value::date because nushell's Value::Date is based on chrono's DateTime<FixedOffset>
        // We'll need to add something like Value::Dt that is based on Jiff
        Ok(Value::string(nowutc.to_string(), call.head))
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

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&UtcNow)
}
