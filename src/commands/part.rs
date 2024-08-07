use super::utils::{
    get_part_from_zoned_as_i16, get_unit_abbreviations, parse_datetime_string_add_nanos_optionally,
};
use crate::DtPlugin;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, SyntaxShape, Value};

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
        vec!["date", "time", "piece", "interval"]
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

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&Part)
}
