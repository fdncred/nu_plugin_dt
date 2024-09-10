use super::utils::{parse_datetime_string_add_nanos_optionally, ISO8601_STRICT};
use crate::DtPlugin;
// use jiff::Zoned;
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
            example: "dt to",
            description: "Print the piped in date or datetime in various standard formats",
            result: None,
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
                parse_datetime_string_add_nanos_optionally(&val.to_rfc3339(), None, span)?
            }
            Value::String { val, .. } => {
                // eprintln!("String: {:?}", val);
                parse_datetime_string_add_nanos_optionally(val, None, span)?
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

// ❯ http get "https://docs.rs/jiff/latest/jiff/fmt/strtime/index.html" | query web -t [Specifier Example Description]                                                                            34  01:34:51 PM
// ╭─#──┬─Specifier──┬─────────Example──────────┬────────────────────────────Description─────────────────────────────╮
// │ 0  │ %%         │ %%                       │ A literal %.                                                       │
// │ 1  │ %A, %a     │ Sunday, Sun              │ The full and abbreviated weekday, respectively.                    │
// │ 2  │ %B, %b, %h │ June, Jun, Jun           │ The full and abbreviated month name, respectively.                 │
// │ 3  │ %D         │ 7/14/24                  │ Equivalent to %m/%d/%y.                                            │
// │ 4  │ %d, %e     │ 25,  5                   │ The day of the month. %d is zero-padded, %e is space padded.       │
// │ 5  │ %F         │ 2024-07-14               │ Equivalent to %Y-%m-%d.                                            │
// │ 6  │ %f         │ 000456                   │ Fractional seconds, up to nanosecond precision.                    │
// │ 7  │ %.f        │ .000456                  │ Optional fractional seconds, with dot, up to nanosecond precision. │
// │ 8  │ %H         │ 23                       │ The hour in a 24 hour clock. Zero padded.                          │
// │ 9  │ %I         │ 11                       │ The hour in a 12 hour clock. Zero padded.                          │
// │ 10 │ %M         │ 04                       │ The minute. Zero padded.                                           │
// │ 11 │ %m         │ 01                       │ The month. Zero padded.                                            │
// │ 12 │ %P         │ am                       │ Whether the time is in the AM or PM, lowercase.                    │
// │ 13 │ %p         │ PM                       │ Whether the time is in the AM or PM, uppercase.                    │
// │ 14 │ %S         │ 59                       │ The second. Zero padded.                                           │
// │ 15 │ %T         │ 23:30:59                 │ Equivalent to %H:%M:%S.                                            │
// │ 16 │ %V         │ America/New_York, +0530  │ An IANA time zone identifier, or %z if one doesn’t exist.          │
// │ 17 │ %:V        │ America/New_York, +05:30 │ An IANA time zone identifier, or %:z if one doesn’t exist.         │
// │ 18 │ %Y         │ 2024                     │ A full year, including century. Zero padded to 4 digits.           │
// │ 19 │ %y         │ 24                       │ A two-digit year. Represents only 1969-2068. Zero padded.          │
// │ 20 │ %Z         │ EDT                      │ A time zone abbreviation. Supported when formatting only.          │
// │ 21 │ %z         │ +0530                    │ A time zone offset in the format [+-]HHMM[SS].                     │
// │ 22 │ %:z        │ +05:30                   │ A time zone offset in the format [+-]HH:MM[:SS].                   │
// ╰─#──┴─Specifier──┴─────────Example──────────┴────────────────────────────Description─────────────────────────────╯
