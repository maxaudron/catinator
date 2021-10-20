# catinator

catinator is an irc bot made by and for the `#gnulag` channel on snoonet. It
also tries to be usable as a general purpose higher level IRC bot making
library, based on the [irc crate](https://docs.rs/irc).

## Configuration

The main configuration file is [config.toml](file:config.toml) and gets loaded
from the current `$PWD`. The configuration file is using profiles, the `default`
profile is loaded as a base. If the binary is compiled in release mode the
`release` profile is merged. You can override any variables from `default`
profile in the `release` or `debug` profile. If the binary is compiled in debug
mode the `debug` profile is loaded from
[config.debug.toml](file:config.debug.toml).

All of the settings can also be set using environment variables. The options are
prefixed with `CATINATOR_`, nested variables are sepperated by `_`.

Common environment variables:

- `CATINATOR_USER_PASSWORD`
- `CATINATOR_WA_API_KEY`

## Developing & Running

```shell
# Compile binary
$ cargo build

# Run catinator
$ cargo run

# Run tests
$ cargo test
```

## Logging

you can change the log level by setting the `RUST_LOG` environment variable.
Available levels are in decreasing verbosity: `trace`, `debug`, `info`, `warn`,
`error`
