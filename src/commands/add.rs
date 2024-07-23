use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, SyntaxShape, Value};

use crate::DtPlugin;

pub struct Add;

impl SimplePluginCommand for Add {
    type Plugin = DtPlugin;

    fn name(&self) -> &str {
        "dt add"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "name",
                SyntaxShape::String,
                "(FIXME) A demo parameter - your name",
            )
            .switch("shout", "(FIXME) Yell it instead", None)
            .category(Category::Date)
    }

    fn usage(&self) -> &str {
        "(FIXME) help text for add"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "add Ellie",
                description: "Say hello to Ellie",
                result: Some(Value::test_string("Hello, Ellie. How are you today?")),
            },
            Example {
                example: "add --shout Ellie",
                description: "Shout hello to Ellie",
                result: Some(Value::test_string("HELLO, ELLIE. HOW ARE YOU TODAY?")),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let name: String = call.req(0)?;
        let mut greeting = format!("Hello, {name}. How are you today?");
        if call.has_flag("shout")? {
            greeting = greeting.to_uppercase();
        }
        Ok(Value::string(greeting, call.head))
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
