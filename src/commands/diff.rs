use super::utils::{
    get_unit_abbreviations, get_unit_from_unit_string, parse_datetime_string_add_nanos_optionally,
};
use crate::DtPlugin;
use jiff::{civil, civil::DateTimeDifference, RoundMode, Unit};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, SyntaxShape, Value};

pub struct Diff;

impl SimplePluginCommand for Diff {
    type Plugin = DtPlugin;

    fn name(&self) -> &str {
        "dt diff"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "date",
                SyntaxShape::OneOf(vec![SyntaxShape::String, SyntaxShape::DateTime]),
                "Date to return the difference from.",
            )
            .named(
                "smallest",
                SyntaxShape::String,
                "Smallest unit to return.",
                Some('s'),
            )
            .named(
                "biggest",
                SyntaxShape::String,
                "Biggest unit to return.",
                Some('b'),
            )
            .named(
                "as",
                SyntaxShape::String,
                "Unit to return difference in.",
                Some('a'),
            )
            .switch("list", "List the unit name abbreviations", Some('l'))
            .category(Category::Date)
    }

    fn usage(&self) -> &str {
        "Return the difference between two dates or datetimes provided"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "'2019-05-10T09:59:12-07:00' | dt diff '2024-08-07T09:36:42.367322100-05:00'",
                description: "Return the difference in the iso8601 duration format",
                result: Some(Value::test_string("P5y2m27dT23h37m30.3673221s")),
            },
            Example {
                example: "'2019-05-10T09:59:12-07:00' | dt diff '2024-08-07T09:36:42.367322100-05:00' --as hr",
                description: "Return the difference as hours in the iso8601 duration format",
                result: Some(Value::test_string("PT45984h")),
            },
            Example {
                example: "'2019-05-10T09:59:12-07:00' | dt diff '2024-08-07T09:36:42.367322100-05:00' --smallest day --biggest year",
                description: "Return the difference as years, months, and days in the iso8601 duration format",
                result: Some(Value::test_string("P5y2m28d")),
            },
        ]
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time", "subtraction", "math"]
    }

    fn run(
        &self,
        _plugin: &DtPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let list = call.has_flag("list")?;
        let smallest_unit_opt: Option<String> = call.get_flag("smallest")?;
        let biggest_unit_opt: Option<String> = call.get_flag("biggest")?;
        let as_unit_opt: Option<String> = call.get_flag("as")?;
        let input_date_provided: Value = call.req(0)?;

        let input_date = match input_date_provided {
            Value::String { val, .. } => val,
            Value::Date { val, .. } => val.to_rfc3339(),
            _ => {
                return Err(
                    LabeledError::new("Expected a date or datetime string".to_string())
                        .with_label("Error", input_date_provided.span()),
                );
            }
        };

        // eprintln!("Input date: {:?}", input_date);

        if list {
            Ok(Value::list(get_unit_abbreviations(), call.head))
        } else {
            let provided_datetime = match input {
                Value::Date { val, .. } => {
                    eprintln!("Date: {:?}", val);
                    return Err(LabeledError::new(
                        "Expected a date or datetime string".to_string(),
                    ));
                }
                Value::String { val, .. } => parse_datetime_string_add_nanos_optionally(val, None)?,
                _ => return Err(LabeledError::new("Expected a date or datetime".to_string())),
            };

            let civil_date_provided = civil::DateTime::from(provided_datetime);
            let civil_input_datetime = input_date
                .parse::<civil::DateTime>()
                .map_err(|err| LabeledError::new(format!("Error parsing input date: {}", err)))?;

            if (biggest_unit_opt.is_some() || smallest_unit_opt.is_some()) && as_unit_opt.is_some()
            {
                return Err(LabeledError::new(
                    "Please provide either smallest, biggest or as unit. As unit is mutually exclusive from smallest and biggest.".to_string(),
                ));
            }

            // if there is an as_unit, use that unit as the smallest and biggest unit
            if let Some(as_unit_string) = as_unit_opt {
                let as_unit = get_unit_from_unit_string(as_unit_string.clone())?;
                let span = civil_date_provided
                    .until(
                        DateTimeDifference::new(civil_input_datetime)
                            .smallest(as_unit)
                            .largest(as_unit)
                            .mode(RoundMode::HalfExpand),
                    )
                    .map_err(|err| {
                        LabeledError::new(format!("Error calculating difference: {}", err))
                    })?;

                // We only want to return the span in the unit asked for
                let span_str = match as_unit {
                    Unit::Year => format!("{}yrs", span.get_years()),
                    Unit::Month => format!("{}mths", span.get_months()),
                    Unit::Week => format!("{}wks", span.get_weeks()),
                    Unit::Day => format!("{}days", span.get_days()),
                    Unit::Hour => format!("{}hrs", span.get_hours()),
                    Unit::Minute => format!("{}mins", span.get_minutes()),
                    Unit::Second => format!("{}secs", span.get_seconds()),
                    Unit::Millisecond => format!("{}ms", span.get_milliseconds()),
                    Unit::Microsecond => format!("{}µs", span.get_microseconds()),
                    Unit::Nanosecond => format!("{}ns", span.get_nanoseconds()),
                };
                Ok(Value::string(format!("{}\n{}", span, span_str), call.head))
            } else {
                // otherwise, use the smallest and biggest units provided
                let smallest_unit = if let Some(smallest_unit_string) = smallest_unit_opt {
                    get_unit_from_unit_string(smallest_unit_string.clone())?
                } else {
                    Unit::Nanosecond
                };

                let biggest_unit = if let Some(biggest_unit_string) = biggest_unit_opt {
                    get_unit_from_unit_string(biggest_unit_string.clone())?
                } else {
                    Unit::Year
                };

                let span = civil_date_provided
                    .until(
                        DateTimeDifference::new(civil_input_datetime)
                            .smallest(smallest_unit)
                            .largest(biggest_unit)
                            .mode(RoundMode::HalfExpand),
                    )
                    .map_err(|err| {
                        LabeledError::new(format!("Error calculating difference: {}", err))
                    })?;

                let span_str = create_nushelly_duration_string(span);
                Ok(Value::string(format!("{}\n{}", span, span_str), call.head))
            }
        }
    }
}

fn create_nushelly_duration_string(span: jiff::Span) -> String {
    let mut span_vec = vec![];
    if span.get_years() > 0 {
        span_vec.push(format!("{}yrs", span.get_years()));
    }
    if span.get_months() > 0 {
        span_vec.push(format!("{}mths", span.get_months()));
    }
    // if we have more than 6 days, show weeks
    let days_span = span.get_days();
    if days_span > 6 {
        let weeks = span.get_weeks();
        if weeks == 0 {
            let (weeks, days) = (days_span / 7, days_span % 7);
            span_vec.push(format!("{}wks", weeks));
            if days > 0 {
                span_vec.push(format!("{}days", days));
            }
        } else {
            if span.get_days() > 0 {
                span_vec.push(format!("{}days", span.get_days()));
            }
        }
    } else {
        if span.get_days() > 0 {
            span_vec.push(format!("{}days", span.get_days()));
        }
    }
    if span.get_hours() > 0 {
        span_vec.push(format!("{}hrs", span.get_hours()));
    }
    if span.get_minutes() > 0 {
        span_vec.push(format!("{}mins", span.get_minutes()));
    }
    if span.get_seconds() > 0 {
        span_vec.push(format!("{}secs", span.get_seconds()));
    }
    if span.get_milliseconds() > 0 {
        span_vec.push(format!("{}ms", span.get_milliseconds()));
    }
    if span.get_microseconds() > 0 {
        span_vec.push(format!("{}µs", span.get_microseconds()));
    }
    if span.get_nanoseconds() > 0 {
        span_vec.push(format!("{}ns", span.get_nanoseconds()));
    }

    span_vec.join(" ").trim().to_string()
}

#[test]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    use nu_plugin_test_support::PluginTest;

    // This will automatically run the examples specified in your command and compare their actual
    // output against what was specified in the example.
    //
    // We recommend you add this test to any other commands you create, or remove it if the examples
    // can't be tested this way.

    PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&Diff)
}
