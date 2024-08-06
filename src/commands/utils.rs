use jiff::{fmt::temporal::DateTimeParser, ToSpan, Zoned};
use nu_plugin::{EngineInterface, EvaluatedCall};
use nu_protocol::{IntoSpanned, LabeledError, PipelineData, Span, Value};

// This is kind of a hack to convert jiff produced nanoseconds to Value::Date by
// converting nanos with the 'into datetime' nushell command
pub fn convert_nanos_to_nushell_datetime_value(
    nanos: i128,
    engine: &EngineInterface,
    span: Span,
    utc: bool,
) -> Result<Value, LabeledError> {
    let Some(decl_id) = engine.find_decl("into datetime")? else {
        return Err(LabeledError::new(
            "Could not find 'into datetime' declaration".to_string(),
        ));
    };
    let into_datetime = engine.call_decl(
        decl_id,
        if utc {
            EvaluatedCall::new(span)
                .with_named("timezone".into_spanned(span), Value::string("UTC", span))
        } else {
            EvaluatedCall::new(span)
                .with_named("timezone".into_spanned(span), Value::string("LOCAL", span))
        },
        PipelineData::Value(Value::int(nanos as i64, span), None),
        true,
        false,
    )?;
    let datetime = into_datetime.into_value(span)?;
    Ok(datetime)
}

// Parse a string into a jiff datetime and add nanoseconds to it optionally
pub fn parse_datetime_string_add_nanos_optionally(
    s: &str,
    duration_nanos: Option<i64>,
) -> Result<Zoned, LabeledError> {
    let cur_date_time_zone = Zoned::now();
    let tz = cur_date_time_zone.time_zone().clone();

    // A parser can be created in a const context.
    static PARSER: DateTimeParser = DateTimeParser::new();

    // Parse a civil datetime string into a civil::DateTime.
    let date_time = PARSER
        .parse_datetime(s)
        .map_err(|err| LabeledError::new(err.to_string()))?;
    // eprintln!("Date: {:?}", date);

    // If nanos are found, add them to the date
    if let Some(nanos) = duration_nanos {
        let date_plus_duration = date_time
            .checked_add(nanos.nanoseconds())
            .map_err(|err| LabeledError::new(err.to_string()))?;
        // eprintln!("Date + Duration: {:?}", date_plus_duration);

        let zdt = date_plus_duration
            .to_zoned(tz)
            .map_err(|err| LabeledError::new(err.to_string()))?;
        // eprintln!("Zoned: {:?}", zdt);

        Ok(zdt)
    } else {
        let zdt = date_time
            .to_zoned(tz)
            .map_err(|err| LabeledError::new(err.to_string()))?;
        Ok(zdt)
    }
}
