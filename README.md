# afkttv

i want this to eventually auto-join streams to collect channel points and continue watch streaks or whatever they are called. may or may not be able to build this functionality but its a fun little project either way.

currently just a very very basic ttv websocket client.

## `config.toml`

this config file is intended to provide authentication information for websocket connection:

```toml
[authorization]
auth = "auth-string-here"
user = "username"
```

> the application expects for this config at `$LOCALAPPDATA/afkttv/config.toml` (ie, `$HOME/.local/share/afkttv/config`, `%LOCALAPPDATA%/afkttv//config`, etc). currently panics if it doesnt exist.
