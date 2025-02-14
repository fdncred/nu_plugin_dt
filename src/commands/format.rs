use super::utils::{
    parse_datetime_string_add_nanos_optionally, unix_timestamp_in_seconds_to_local_zoned,
};
use crate::DtPlugin;
use jiff::{fmt::rfc2822, Zoned};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{
    record, Category, Example, LabeledError, Signature, Span as NuSpan, SyntaxShape, Value,
};

pub struct DtFormat;

impl SimplePluginCommand for DtFormat {
    type Plugin = DtPlugin;

    fn name(&self) -> &str {
        "dt format"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "format",
                SyntaxShape::String,
                "Format string to use to format the date/datetime",
            )
            .switch(
                "list",
                "List of Display Formats supported by Jiff",
                Some('l'),
            )
            .category(Category::Date)
    }

    fn description(&self) -> &str {
        "Print the date or datetime in the specified format"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time", "current", "print", "strftime", "strptime"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "'07/09/24' | dt format %A",
            description: "Print the full weekday",
            result: Some(Value::test_string("Tuesday".to_string())),
        }]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let span = input.span();
        let list = call.has_flag("list")?;
        if list {
            return Ok(generate_strftime_list(span, false));
        }

        let format_param: Option<String> = call.req(0)?;
        let format_string = match format_param {
            Some(format) => format,
            None => {
                return Err(LabeledError::new(
                    "Expected a format string in format".to_string(),
                ))
            }
        };

        let datetime = match input {
            Value::Date { val, .. } => {
                // so much easier just to output chrono as rfc 3339 and let jiff parse it
                parse_datetime_string_add_nanos_optionally(&val.to_rfc3339(), None, span, None)?
            }
            Value::String { val, .. } => {
                // eprintln!("String: {:?}", val);
                parse_datetime_string_add_nanos_optionally(val, None, span, None)?
            }
            Value::Int { val, .. } => {
                // eprintln!("Int: {:?}", val);
                let dt_str = unix_timestamp_in_seconds_to_local_zoned(*val)?;
                parse_datetime_string_add_nanos_optionally(&dt_str, None, span, None)?
            }
            _ => {
                return Err(LabeledError::new(
                    "Expected a date or datetime in add".to_string(),
                ))
            }
        };

        let formatted_dt = if format_string == "%+" {
            datetime.strftime("%FT%T%:z").to_string() // RFC 3339
        } else if format_string == "%c" {
            rfc2822::to_string(&datetime).unwrap_or_default()
        } else {
            datetime.strftime(&format_string).to_string()
        };

        // let formatted_dt = strtime::format(format_string.clone(), &datetime).map_err(|err| {
        //     LabeledError::new(format!("Error formatting datetime: {:?}", err)).with_label(
        //         format!("Error with format string: {}", format_string.clone()),
        //         span,
        //     )
        // })?;
        Ok(Value::string(formatted_dt, span))
    }
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

/// Generates a table containing available datetime format specifiers
///
/// # Arguments
/// * `head` - use the call's head
/// * `show_parse_only_formats` - whether parse-only format specifiers (that can't be outputted) should be shown. Should only be used for `into datetime`, not `format date`
pub fn generate_strftime_list(head: NuSpan, show_parse_only_formats: bool) -> Value {
    // let now = Local::now();
    let now = Zoned::now();

    struct FormatSpecification<'a> {
        spec: &'a str,
        description: &'a str,
    }

    let specifications = [
        FormatSpecification {
            spec: "%%",
            description: "A literal %.",
        },
        FormatSpecification {
            spec: "%+",
            description: "ISO 8601 / RFC 3339 date & time format.",
        },
        FormatSpecification {
            spec: "%A",
            description: "The full weekday.",
        },
        FormatSpecification {
            spec: "%a",
            description: "The abbreviated weekday.",
        },
        FormatSpecification {
            spec: "%B",
            description: "The full month name.",
        },
        FormatSpecification {
            spec: "%b",
            description: "The abbreviated month name.",
        },
        FormatSpecification {
            spec: "%h",
            description: "The abbreviated month name.",
        },
        FormatSpecification {
            spec: "%C",
            description: "The century of the year. No padding.",
        },
        FormatSpecification {
            spec: "%c",
            description:
                "Locale's date and time in rfc2822 format (e.g., Wed, 2 Oct 2024 12:53:44 -0500).",
        },
        FormatSpecification {
            spec: "%D",
            description: "Equivalent to %m/%d/%y.",
        },
        FormatSpecification {
            spec: "%d",
            description: "The day of the month. Zero-padded.",
        },
        FormatSpecification {
            spec: "%e",
            description: "The day of the month. Space padded.",
        },
        FormatSpecification {
            spec: "%F",
            description: "Equivalent to %Y-%m-%d.",
        },
        FormatSpecification {
            spec: "%f",
            description: "Fractional seconds, up to nanosecond precision.",
        },
        FormatSpecification {
            spec: "%.f",
            description: "Optional fractional seconds, with dot, up to nanosecond precision..",
        },
        FormatSpecification {
            spec: "%.3f",
            description: "Similar to .%f but left-aligned but fixed to a length of 3.",
        },
        FormatSpecification {
            spec: "%.6f",
            description: "Similar to .%f but left-aligned but fixed to a length of 6.",
        },
        FormatSpecification {
            spec: "%.9f",
            description: "Similar to .%f but left-aligned but fixed to a length of 9.",
        },
        FormatSpecification {
            spec: "%3f",
            description: "Similar to %.3f but without the leading dot.",
        },
        FormatSpecification {
            spec: "%6f",
            description: "Similar to %.6f but without the leading dot.",
        },
        FormatSpecification {
            spec: "%9f",
            description: "Similar to %.9f but without the leading dot.",
        },
        FormatSpecification {
            spec: "%G",
            description: "An ISO 8601 week-based year. Zero padded to 4 digits.",
        },
        FormatSpecification {
            spec: "%g",
            description:
                "A two-digit ISO 8601 week-based year. Represents only 1969-2068. Zero padded.",
        },
        FormatSpecification {
            spec: "%H",
            description: "The hour in a 24 hour clock. Zero padded.",
        },
        FormatSpecification {
            spec: "%I",
            description: "The hour in a 12 hour clock. Zero padded.",
        },
        FormatSpecification {
            spec: "%j",
            description: "The day of the year. Range is 1..=366. Zero padded to 3 digits.",
        },
        FormatSpecification {
            spec: "%k",
            description: "The hour in a 24 hour clock. Space padded.",
        },
        FormatSpecification {
            spec: "%l",
            description: "The hour in a 12 hour clock. Space padded.",
        },
        FormatSpecification {
            spec: "%M",
            description: "The minute. Zero padded.",
        },
        FormatSpecification {
            spec: "%m",
            description: "The month. Zero padded.",
        },
        FormatSpecification {
            spec: "%n",
            description: "Formats as a newline character. Parses arbitrary whitespace.",
        },
        FormatSpecification {
            spec: "%P",
            description: "Whether the time is in the am or pm, lowercase. (am/pm)",
        },
        FormatSpecification {
            spec: "%p",
            description: "Whether the time is in the AM or PM, uppercase. (AM/PM)",
        },
        FormatSpecification {
            spec: "%Q",
            description: "An IANA time zone identifier, or %z if one doesn’t exist.",
        },
        FormatSpecification {
            spec: "%:Q",
            description: "An IANA time zone identifier, or %:z if one doesn’t exist.",
        },
        FormatSpecification {
            spec: "%R",
            description: "Equivalent to %H:%M.",
        },
        FormatSpecification {
            spec: "%S",
            description: "The second. Zero padded.",
        },
        FormatSpecification {
            spec: "%s",
            description: "A Unix timestamp, in seconds.",
        },
        FormatSpecification {
            spec: "%T",
            description: "Equivalent to %H:%M:%S.",
        },
        FormatSpecification {
            spec: "%t",
            description: "Formats as a tab character. Parses arbitrary whitespace.",
        },
        FormatSpecification {
            spec: "%U",
            description:
                "Week number. Week 1 is the first week starting with a Sunday. Zero padded.",
        },
        FormatSpecification {
            spec: "%u",
            description: "The day of the week beginning with Monday at 1.",
        },
        FormatSpecification {
            spec: "%V",
            description: "Week number in the ISO 8601 week-based calendar. Zero padded.",
        },
        FormatSpecification {
            spec: "%W",
            description:
                "Week number. Week 1 is the first week starting with a Monday. Zero padded.",
        },
        FormatSpecification {
            spec: "%w",
            description: "The day of the week beginning with Sunday at 0.",
        },
        FormatSpecification {
            spec: "%Y",
            description: "A full year, including century. Zero padded to 4 digits.",
        },
        FormatSpecification {
            spec: "%y",
            description: "A two-digit year. Represents only 1969-2068. Zero padded.",
        },
        FormatSpecification {
            spec: "%Z",
            description: "A time zone abbreviation. Supported when formatting only.",
        },
        FormatSpecification {
            spec: "%z",
            description: "A time zone offset in the format [+-]HHMM[SS].",
        },
        FormatSpecification {
            spec: "%:z",
            description: "A time zone offset in the format [+-]HH:MM[:SS].",
        },
        // FormatSpecification {
        //     spec: "%C",
        //     description: "The proleptic Gregorian year divided by 100, zero-padded to 2 digits.",
        // },
        // FormatSpecification {
        //     spec: "%w",
        //     description: "Sunday = 0, Monday = 1, ..., Saturday = 6.",
        // },
        // FormatSpecification {
        //     spec: "%u",
        //     description: "Monday = 1, Tuesday = 2, ..., Sunday = 7. (ISO 8601)",
        // },
        // FormatSpecification {
        //     spec: "%U",
        //     description: "Week number starting with Sunday (00--53), zero-padded to 2 digits.",
        // },
        // FormatSpecification {
        //     spec: "%W",
        //     description:
        //         "Same as %U, but week 1 starts with the first Monday in that year instead.",
        // },
        // FormatSpecification {
        //     spec: "%G",
        //     description: "Same as %Y but uses the year number in ISO 8601 week date.",
        // },
        // FormatSpecification {
        //     spec: "%g",
        //     description: "Same as %y but uses the year number in ISO 8601 week date.",
        // },
        // FormatSpecification {
        //     spec: "%j",
        //     description: "Day of the year (001--366), zero-padded to 3 digits.",
        // },
        // FormatSpecification {
        //     spec: "%x",
        //     description: "Locale's date representation (e.g., 12/31/99).",
        // },
        // FormatSpecification {
        //     spec: "%v",
        //     description: "Day-month-year format. Same as %e-%b-%Y.",
        // },
        // FormatSpecification {
        //     spec: "%k",
        //     description: "Same as %H but space-padded. Same as %_H.",
        // },
        // FormatSpecification {
        //     spec: "%l",
        //     description: "Same as %I but space-padded. Same as %_I.",
        // },
        // FormatSpecification {
        //     spec: "%R",
        //     description: "Hour-minute format. Same as %H:%M.",
        // },
        // FormatSpecification {
        //     spec: "%X",
        //     description: "Locale's time representation (e.g., 23:13:48).",
        // },
        // FormatSpecification {
        //     spec: "%r",
        //     description: "Hour-minute-second format in 12-hour clocks. Same as %I:%M:%S %p.",
        // },
        // FormatSpecification {
        //     spec: "%c",
        //     description: "Locale's date and time (e.g., Thu Mar 3 23:05:25 2005).",
        // },
        // FormatSpecification {
        //     spec: "%+",
        //     description: "ISO 8601 / RFC 3339 date & time format.",
        // },
        // FormatSpecification {
        //     spec: "%s",
        //     description: "UNIX timestamp, the number of seconds since 1970-01-01",
        // },
        // FormatSpecification {
        //     spec: "%t",
        //     description: "Literal tab (\\t).",
        // },
        // FormatSpecification {
        //     spec: "%n",
        //     description: "Literal newline (\\n).",
        // },
    ];

    let mut records = specifications
        .iter()
        .map(|s| {
            Value::record(
                record! {
                    "Specification" => Value::string(s.spec, head),
                    "Example" => {
                        if s.spec == "%+" {
                            Value::string(now.strftime("%FT%T%:z").to_string(), head)
                        } else if s.spec == "%c" {
                            let rfc2822 = rfc2822::to_string(&now).unwrap_or_default();
                            Value::string(rfc2822, head)
                        } else if s.spec == "%n" {
                            Value::string("\\n", head)
                        } else if s.spec == "%t" {
                            Value::string("\\t", head)
                        }
                        else {
                            Value::string(now.strftime(s.spec).to_string(), head)
                        }
                    },
                    "Description" => Value::string(s.description, head),
                },
                head,
            )
        })
        .collect::<Vec<Value>>();

    if show_parse_only_formats {
        // now.format("%#z") will panic since it is parse-only
        // so here we emulate how it will look:
        let example = now
            .strftime("%:z") // e.g. +09:30
            .to_string()
            .get(0..3) // +09:30 -> +09
            .unwrap_or("")
            .to_string();

        let description = "Parsing only: Same as %z but allows minutes to be missing or present.";

        records.push(Value::record(
            record! {
                "Specification" => Value::string("%#z", head),
                "Example" => Value::string(example, head),
                "Description" => Value::string(description, head),
            },
            head,
        ));
    }

    Value::list(records, head)
}

#[test]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    use nu_plugin_test_support::PluginTest;

    // This will automatically run the examples specified in your command and compare their actual
    // output against what was specified in the example.
    //
    // We recommend you add this test to any other commands you create, or remove it if the examples
    // can't be tested this way.

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&DtFormat)
}
