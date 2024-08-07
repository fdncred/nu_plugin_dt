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
    - [x] `dt add` duration
    - [ ] `dt sub` duration
    - [ ] `dt sub` date
    - [ ] `dt sub` time
    - [ ] `dt sub` datetime
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
    - [x] `dt now`
    - [x] `dt utcnow`
    - [x] `dt part`

    - should date math be more [sql like](https://www.sqlshack.com/how-to-add-or-subtract-dates-in-sql-server/) where you have a `date add` and `date diff` function that takes a positive or negative number and a unit?
        - [dt add](https://www.w3schools.com/sql/func_sqlserver_dateadd.asp) SQL: `SELECT DATEADD(year, 1, '2017/08/25') AS DateAdd;`
        - [dt diff](https://www.w3schools.com/sql/func_sqlserver_datediff.asp) SQL: `SELECT DATEDIFF(year, '2017/08/25', '2011/08/25') AS DateDiff;`
        - [dt part](https://www.w3schools.com/sql/func_sqlserver_datepart.asp) SQL: `SELECT DATEPART(year, '2017/08/25') AS DatePartInt;`
        - [dt date](https://www.w3schools.com/sql/func_sqlserver_getdate.asp) SQL: `SELECT GETDATE();`
        - [dt utcdate](https://www.w3schools.com/sql/func_sqlserver_getutcdate.asp) SQL: `SELECT GETUTCDATE();`
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
- support current nushell date commands
    - `dt now`
    - `dt list-timezones`
    - the others but with less priority
- able to consume/understand nushell date/datetime literals
- i'm not sure if it's possible to get operators to work in a plugin like `+`, and `-`

# Use cases / Examples

I'm not sure all of these will be possible but I'm just documenting some common use cases.

## If Date Math were Nushell style

These could take a date or date time piped in.

```nushell
# dt add (add a duration to a date)
'2017-08-25' | dt add 1day
# dt sub (subtract a duration from a date)
'2017-08-25' | dt sub 1day
# dt sub (subtract two dates)
'2017-08-25' | dt sub 2024-07-01
# dt sub (subtract time from a date)
'2017-08-25' | dt sub 00:02:00
# dt sub (subtrace datetime from a date)
'2017-08-25' | dt sub 2024-07-01T00:02:00
# dt part (get the part of the current datetime)
'2017-08-25' | dt part year
# dt now (get the current local datetime)
dt now
# dt utcnow (get the current utc datetime)
dt utcnow
# dt diff (get the difference between two dates)
'2017-08-25' | dt diff '2024-07-01' --smallest unit --largest unit
```

## If Date Math were SQL Style

- [dt add](https://www.w3schools.com/sql/func_sqlserver_dateadd.asp) SQL: `SELECT DATEADD(year, 1, '2017/08/25') AS DateAdd;`
- [dt diff](https://www.w3schools.com/sql/func_sqlserver_datediff.asp) SQL: `SELECT DATEDIFF(year, '2017/08/25', '2011/08/25') AS DateDiff;`
- [dt part](https://www.w3schools.com/sql/func_sqlserver_datepart.asp) SQL: `SELECT DATEPART(year, '2017/08/25') AS DatePartInt;`
- [dt date](https://www.w3schools.com/sql/func_sqlserver_getdate.asp) SQL: `SELECT GETDATE();`
- [dt utcdate](https://www.w3schools.com/sql/func_sqlserver_getutcdate.asp) SQL: `SELECT GETUTCDATE();`

Where `unit` is this sql list of intervals. This may be limited by what `jiff` accepts.
```
year, yyyy, yy = Year
quarter, qq, q = Quarter
month, mm, m = month
dayofyear, dy, y = Day of the year
day, dd, d = Day of the month
week, ww, wk = Week
weekday, dw, w = Weekday
hour, hh = hour
minute, mi, n = Minute
second, ss, s = Second
millisecond, ms = Millisecond
microsecond, mcs = Microsecond
nanosecond, ns = Nanosecond
tzoffset, tz = Timezone offset
iso_week, isowk, isoww = ISO week
```

```nushell
# dt add (add 1 year)
'2017-08-25' | dt add --unit year --amount 1
# dt add (subtract 1 year)
'2017-08-25' | dt add --unit year --amount -1
# dt diff (subtract two dates)
'2017-08-25' | dt diff --unit year --date 2011-08-25
# dt part (get the year part of the date/datetime passed in)
'2017-08-25' | dt part --unit year (may not need the --unit here but leaving to be consistent for now)
# dt now (get the current local datetime)
dt now
# dt utcnow (get the current utc datetime)
dt utcnow
```
