# afkttv

> i want this to eventually auto-join streams to collect channel points and continue watch streaks or whatever they are called. may or maybe not be able to actually do that.

## `config.toml`

this config file is intended to provide authentication information for websocket connection:

```toml
[authorization]
auth = "auth-string-here"
user = "username"
```

> the application expects for this config at `$LOCALAPPDATA/afkttv/config.toml` (ie, `$HOME/.local/share/afkttv/config`, `%LOCALAPPDATA%/afkttv//config`, etc). currently panics if it doesnt exist.
