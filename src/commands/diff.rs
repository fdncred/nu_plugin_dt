use super::utils::{
    create_nushelly_duration_string, get_single_duration_unit_from_span, get_unit_abbreviations,
    get_unit_from_unit_string, parse_datetime_string_add_nanos_optionally,
};
use crate::DtPlugin;
use jiff::{RoundMode, Unit, ZonedDifference, tz::TimeZone};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Signature, Span, SyntaxShape, Value};

pub struct DtDiff;

impl SimplePluginCommand for DtDiff {
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

    fn description(&self) -> &str {
        "Return the difference between two dates or datetimes provided"
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "'2019-05-10T09:59:12-07:00' | dt diff '2024-08-07T09:36:42.367322100-05:00'",
                description: "Return the difference in the iso8601 duration format",
                result: Some(Value::test_string(
                    "P5y2m27dT21h37m30.3673221s\n5yrs 2mths 3wks 6days 21hrs 37mins 30secs 367ms 322µs 100ns",
                )),
            },
            Example {
                example: "'2019-05-10T09:59:12-07:00' | dt diff '2024-08-07T09:36:42.367322100-05:00' --as hr",
                description: "Return the difference as hours in the iso8601 duration format",
                result: Some(Value::test_string("PT45982h\n45982hrs")),
            },
            Example {
                example: "'2019-05-10T09:59:12-07:00' | dt diff '2024-08-07T09:36:42.367322100-05:00' --smallest day --biggest year",
                description: "Return the difference as years, months, and days in the iso8601 duration format",
                result: Some(Value::test_string("P5y2m28d\n5yrs 2mths 4wks")),
            },
            Example {
                example: "'2019-05-10T09:59:12-07:00' | dt diff (dt now)",
                description: "Return the difference in the iso8601 duration format using the current datetime from dt as input",
                result: None,
            },
            Example {
                example: "'2019-05-10T09:59:12-07:00' | dt diff (date now)",
                description: "Return the difference in the iso8601 duration format using the current datetime from the date command as input",
                result: None,
            },
            Example {
                example: "(dt now) | dt diff (date now)",
                description: "Return the difference in the iso8601 duration format using the current datetime from both dt and date commands as input",
                result: None,
            },
        ]
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["date", "time", "subtraction", "math"]
    }

    // GOAL: Match https://www.timeanddate.com/date/timezoneduration.html?
    // https://www.timeanddate.com/date/timezoneduration.html?d1=10&m1=05&y1=2019&d2=20&m2=08&y2=2024&h1=09&i1=59&s1=12&h2=10&i2=24&s2=11&
    // '2019-05-10T09:59:12-07:00' | dt diff '2024-08-20T10:24:11.693666300-05:00'
    // P5y3m9dT22h24m59.6936663s
    // 5yrs 3mths 1wks 2days 22hrs 24mins 59secs 693ms 666µs 300ns

    // From: Friday, May 10, 2019 at 9:59:12 am Los Angeles time
    // To: Tuesday, August 20, 2024 at 10:24:11 am Little Rock time

    // Result: 1928 days, 22 hours, 24 minutes and 59 seconds
    // The duration is 1928 days, 22 hours, 24 minutes and 59 seconds

    // Or 5 years, 3 months, 9 days, 22 hours, 24 minutes, 59 seconds

    // Or 63 months, 9 days, 22 hours, 24 minutes, 59 seconds

    // Alternative time units
    // 1928 days, 22 hours, 24 minutes and 59 seconds can be converted to one of these units:

    // 166,659,899 seconds
    // 2,777,664 minutes (rounded down)
    // 46,294 hours (rounded down)
    // 1928 days (rounded down)
    // 275 weeks (rounded down)
    // 528.48% of a common year (365 days)
    // ◀ Make adjustment and calculate againStart Again ▶

    // Event	    Los Angeles time	        Little Rock time	        UTC time
    // Start time	May 10, 2019 at 9:59:12 am	May 10, 2019 at 11:59:12 am	May 10, 2019 at 4:59:12 pm
    // End time	    Aug 20, 2024 at 8:24:11 am	Aug 20, 2024 at 10:24:11 am	Aug 20, 2024 at 3:24:11 pm

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
        let parameter_datetime_provided: Value = call.req(0)?;
        let span = call.head;

        // eprintln!(" Input to pipeline date: {:?}", input.clone());
        // eprintln!(
        //     "Parameter provided date: {:?}",
        //     parameter_datetime_provided.clone()
        // );

        if list {
            Ok(Value::list(get_unit_abbreviations(), call.head))
        } else {
            calculate_date_diff(
                parameter_datetime_provided,
                input,
                biggest_unit_opt,
                smallest_unit_opt,
                as_unit_opt,
                span,
            )
        }
    }
}

fn calculate_date_diff(
    parameter_datetime_provided: Value,
    piped_in_input: &Value,
    biggest_unit_opt: Option<String>,
    smallest_unit_opt: Option<String>,
    as_unit_opt: Option<String>,
    call_span: Span,
) -> Result<Value, LabeledError> {
    let param_span = parameter_datetime_provided.span();
    let piped_span = piped_in_input.span();
    let parameter_datetime = match parameter_datetime_provided {
        Value::String { val, .. } => val,
        Value::Date { val, .. } => val.to_rfc3339(),
        _ => {
            return Err(LabeledError::new(
                "Expected a date or datetime string in diff".to_string(),
            )
            .with_label("Error", parameter_datetime_provided.span()));
        }
    };

    // convert piped_in_input into a jiff::Zoned
    let mut zoned_input_datetime = match piped_in_input {
        Value::Date { val, .. } => {
            // eprintln!("Date rfc3339: {:?}", &val.to_rfc3339());
            parse_datetime_string_add_nanos_optionally(&val.to_rfc3339(), None, piped_span, None)?
        }
        Value::String { val, .. } => {
            parse_datetime_string_add_nanos_optionally(val, None, piped_span, None)?
        }
        _ => return Err(LabeledError::new("Expected a date or datetime".to_string())),
    };

    // convert parameter_datetime into a jiff::Zoned
    let mut zoned_parameter_datetime =
        parse_datetime_string_add_nanos_optionally(&parameter_datetime, None, param_span, None)?;

    // Check to see if biggest_unit_opt and smallest_unit_opt are both provided or as_unit_opt is provided
    if (biggest_unit_opt.is_some() || smallest_unit_opt.is_some()) && as_unit_opt.is_some() {
        return Err(LabeledError::new(
            "Please provide either smallest, biggest or as unit. As unit is mutually exclusive from smallest and biggest.".to_string(),
        ));
    }

    if zoned_input_datetime.time_zone() == zoned_parameter_datetime.time_zone() {
        // eprintln!("Timezones are the same");
    } else {
        // eprintln!("Timezones are different");
        // let zdt_input_ts = zoned_input_datetime.timestamp();
        // let zdt_parameter_ts = zoned_parameter_datetime.timestamp();
        // eprintln!(
        //     "Timestamp diff input - param: {}",
        //     zdt_input_ts - zdt_parameter_ts
        // );
        // eprintln!(
        //     "Timestamp diff param - input: {}",
        //     zdt_parameter_ts - zdt_input_ts
        // );
        // eprintln!(
        //     "Changed input timezone {:?} to match parameter timezone {:?}",
        //     zoned_input_datetime.time_zone(),
        //     zoned_parameter_datetime.time_zone(),
        // );
        zoned_input_datetime = zoned_input_datetime.with_time_zone(TimeZone::UTC);
        zoned_parameter_datetime = zoned_parameter_datetime.with_time_zone(TimeZone::UTC);
        // eprintln!("    New input datetime: {zoned_input_datetime:?}");
        // eprintln!("New parameter datetime: {zoned_parameter_datetime:?}");
    }

    // if there is an as_unit, use that unit as the smallest and biggest unit
    if let Some(as_unit_string) = as_unit_opt {
        let as_unit = get_unit_from_unit_string(as_unit_string.clone())?;
        // Since we have jiff::Zoned datetime types, use ZonedDifference to calculate the difference
        let span = zoned_input_datetime
            .until(
                ZonedDifference::new(&zoned_parameter_datetime)
                    .smallest(as_unit)
                    .largest(as_unit)
                    .mode(RoundMode::HalfExpand),
            )
            .map_err(|err| LabeledError::new(format!("Error calculating difference1: {}", err)))?;

        // We only want to return the span in the unit asked for
        let span_str = get_single_duration_unit_from_span(as_unit, span);
        Ok(Value::string(format!("{}\n{}", span, span_str), call_span))
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

        // Since we have jiff::Zoned datetime types, use ZonedDifference to calculate the difference
        let span = zoned_input_datetime
            .until(
                ZonedDifference::new(&zoned_parameter_datetime)
                    .smallest(smallest_unit)
                    .largest(biggest_unit)
                    .mode(RoundMode::HalfExpand),
            )
            .map_err(|err| LabeledError::new(format!("Error calculating difference2: {}", err)))?;

        // We want to return a nushelly duration string with all units
        let span_str = create_nushelly_duration_string(span);
        Ok(Value::string(format!("{}\n{}", span, span_str), call_span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nu_plugin_test_support::PluginTest;
    use nu_protocol::{Span, Value};

    #[test]
    fn test_examples() -> Result<(), nu_protocol::ShellError> {
        PluginTest::new("dt", DtPlugin.into())?.test_command_examples(&DtDiff)
    }

    #[test]
    fn test_calculate_date_diff() -> Result<(), LabeledError> {
        let parameter_datetime_provided = Value::test_string("2024-08-20T10:24:11.693666300-05:00");
        let piped_in_input = Value::test_string("2019-05-10T09:59:12-07:00");

        let result = calculate_date_diff(
            parameter_datetime_provided,
            &piped_in_input,
            None,
            None,
            None,
            Span::unknown(),
        )?;

        assert_eq!(
            result.into_string()?,
            "P5y3m9dT22h24m59.6936663s\n5yrs 3mths 1wks 2days 22hrs 24mins 59secs 693ms 666µs 300ns"
        );

        Ok(())
    }

    #[test]
    fn test_calculate_date_diff_with_as_unit() -> Result<(), LabeledError> {
        let parameter_datetime_provided = Value::test_string("2024-08-20T10:24:11.693666300-05:00");
        let piped_in_input = Value::test_string("2019-05-10T09:59:12-07:00");

        let result = calculate_date_diff(
            parameter_datetime_provided,
            &piped_in_input,
            None,
            None,
            Some("hr".to_string()),
            Span::unknown(),
        )?;

        assert_eq!(result.into_string()?, "PT46294h\n46294hrs");

        Ok(())
    }

    #[test]
    fn test_calculate_date_diff_with_smallest_and_biggest() -> Result<(), LabeledError> {
        let parameter_datetime_provided = Value::test_string("2024-08-20T10:24:11.693666300-05:00");
        let piped_in_input = Value::test_string("2019-05-10T09:59:12-07:00");

        let result = calculate_date_diff(
            parameter_datetime_provided,
            &piped_in_input,
            Some("year".to_string()),
            Some("day".to_string()),
            None,
            Span::unknown(),
        )?;

        assert_eq!(result.into_string()?, "P5y3m10d\n5yrs 3mths 1wks 3days");

        Ok(())
    }

    #[test]
    fn test_calculate_date_diff_with_invalid_input() {
        let parameter_datetime_provided = Value::test_string("2024-08-20T10:24:11.693666300-05:00");
        let piped_in_input = Value::test_string("Invalid Date");

        let result = calculate_date_diff(
            parameter_datetime_provided,
            &piped_in_input,
            None,
            None,
            None,
            Span::unknown(),
        );

        assert!(result.is_err());
    }
}
