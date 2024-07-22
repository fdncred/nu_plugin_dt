# nu_plugin_dt

This is a [Nushell](https://nushell.sh/) plugin called "dt".

## Installing

```nushell
> cargo install --path .
```

## Usage

FIXME: This reflects the demo functionality generated with the template. Update this documentation
once you have implemented the actual plugin functionality.

```nushell
> plugin add ~/.cargo/bin/nu_plugin_dt
```

# Brainstorming datetime plugin

What would a new datetime plugin look like in nushell?
Name: nu_plugin_dt

Maybe create with an eye toward replacing chrono in nushell eventually. I'm not really sure what that means for a plugin but this experiement will be a good introduction to jiff. If the experiment is successful, then it may be good enough to replace chrono in nushell.

# Requirements

This is not meant to be an exhaustive list of requirments but enough to get started. We can add more here as we progress.

- use [jiff](https://github.com/BurntSushi/jiff) crate and [docs](https://docs.rs/jiff/latest/jiff/)
- date math
    - dt add duration
    - dt sub duration
    - dt sub date
    - dt sub time
    - dt sub datetime
    - ability to express durations in a similar way that nushell does or better. specifically better means the ability to return durations in these representations while accounting for leap year math so that any date math operation is accurate. Note: it's known that leap-seconds do not exist in jiff (yet) so it doesn't have to be that accurate but more accurate than nushell currently is without having to average days for months or years.
        - years
        - months
        - weeks
        - days
        - hours
        - minutes
        - seconds
        - milliseconds
        - microseconds
        - nanoseconds
    - should date math be more [sql like](https://www.sqlshack.com/how-to-add-or-subtract-dates-in-sql-server/) where you have a `date add` and `date diff` function that takes a positive or negative number and a unit?
        - [dt add](https://www.w3schools.com/sql/func_sqlserver_dateadd.asp)
        - [dt diff](https://www.w3schools.com/sql/func_sqlserver_datediff.asp)
        - [dt part](https://www.w3schools.com/sql/func_sqlserver_datepart.asp)
        - [dt date](https://www.w3schools.com/sql/func_sqlserver_getdate.asp)
        - [dt utcdate](https://www.w3schools.com/sql/func_sqlserver_getutcdate.asp)
- date parsing
    - dt parse
        - separate date
            - provide date and assume 00:00:00 time
        - separate time
            - provide time and assume local date
- date formatting
    - typical [strftime](https://pubs.opengroup.org/onlinepubs/009695399/functions/strftime.html) formatting
    - nushell default
    - dt to-rfc3339
    - dt to-rfc9557
    - dt to-rfc2822
    - dt to-iso8601
- support round trip serialization, perhaps with serde
