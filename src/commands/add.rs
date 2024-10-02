use super::utils::parse_datetime_string_add_nanos_optionally;
use crate::DtPlugin;
use jiff::Span as JiffSpan;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, Signature, Span as NuSpan, Spanned, SyntaxShape, Value,
};
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
                SyntaxShape::String,
                "Duration to add to the provided in date and time",
            )
            .category(Category::Date)
    }

    fn description(&self) -> &str {
        "Add a jiff duration to the provided in date and time"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time", "addition", "math"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "'2017-08-25' | dt add 1d",
                description: "Add jiff duration of 1 day to the provided date string in the local timezone",
                result: Some(Value::test_string(
                    "2017-08-26T00:00:00-05:00[America/Chicago]",
                )),
            },
            Example {
                example: "'2017-08-25T12:00:00' | dt add T1h",
                description: "Add jiff duration of 1 hour to the provided date and time string in the local timezone",
                result: Some(Value::test_string(
                    "2017-08-25T13:00:00-05:00[America/Chicago]",
                )),
            },
            Example {
                example: "2017-08-25 | dt add 2w",
                description: "Add jiff duration of 2 weeks to the provided nushell date in the local timezone",
                result: Some(Value::test_string(
                    "2017-09-08T00:00:00-05:00[America/Chicago]",
                )),
            },
            Example {
                example: "dt now | dt add 2w",
                description: "Add jiff duration of 2 weeks to the provided dt command date in the local timezone",
                result: None,
            },
            Example {
                example: "date now | dt add 2w",
                description: "Add jiff duration of 2 weeks to the provided date command date in the local timezone",
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
        // From weirdan
        // If no timezone is specified, assume local tz. Provide a way to override that. Alternatively, reject dates without a timezone.

        let span: NuSpan = input.span();
        // TODO: Accomodate negative jiff spans like -P1d
        let mut duration_string: Spanned<String> = call.req(0)?;

        // The jiff span is roughly comparable to this
        // P(\d+y)?(\d+m)?(\d+w)?(\d+d)?(T(\d+h)?(\d+m)?(\d+s)?)?
        // P = date designator
        // y = years
        // m = months
        // w = weeks
        // d = days
        // T = time designator
        // h = hours
        // m = minutes
        // s = seconds
        // .000 = milliseconds
        // .000000 = microseconds
        // .000000000 = nanoseconds

        // is it negative
        if (duration_string.item.starts_with('-')
            || duration_string.item.starts_with('+'))
            // is it longer than 2
            && duration_string.item.len() > 2
            // is the 2nd character not a P
            && duration_string.item.chars().nth(1).unwrap() != 'P'
        {
            duration_string.item.insert(1, 'P');
        } else if !duration_string.item.starts_with('P') {
            // else if it doesn't start with P, add it because jiff's span parser requires it
            duration_string.item.insert(0, 'P');
        }

        let jiff_span: JiffSpan = duration_string.item.parse().map_err(|err| {
            LabeledError::new(format!("Error parsing duration: {err}"))
                .with_label(
                    format!("error parsing {:?} as a jiff span", duration_string.item),
                    duration_string.span,
                )
                .with_help(
                    r#"These are the valid component abbreviations:
+ = positive
- = negative
P = date designator (inferred if you forget it)
y = years
m = months
w = weeks
d = days
T = time designator
h = hours
m = minutes
s = seconds
.000 = milliseconds
.000000 = microseconds
.000000000 = nanoseconds
"#,
                )
        })?;

        // eprintln!("Jiff span: {:?}", jiff_span);

        let datetime = match input {
            Value::Date { val, .. } => {
                // so much easier just to output chrono as rfc 3339 and let jiff parse it

                // parse_datetime_string_add_nanos_optionally(
                //     &val.to_rfc3339(),
                //     Some(duration_nanos as i64),
                //     span,
                //     None,
                // )?

                // Check for just a date like 2017-08-25 being passed in. If it is, then the time will be 00:00:00+00:00
                let mut rfc3399 = val.to_rfc3339();
                if rfc3399.ends_with("T00:00:00+00:00") {
                    if let Some(empty_time) = rfc3399.rfind("T00:00:00+00:00") {
                        rfc3399 = rfc3399[0..empty_time].to_string();
                    }
                }
                parse_datetime_string_add_nanos_optionally(&rfc3399, None, span, Some(jiff_span))?
            }
            Value::String { val, .. } => {
                // eprintln!("String: {:?}", val);
                // parse_datetime_string_add_nanos_optionally(
                //     val,
                //     Some(duration_nanos as i64),
                //     span,
                //     None,
                // )?
                parse_datetime_string_add_nanos_optionally(val, None, span, Some(jiff_span))?
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
