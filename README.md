# mihomo rules reverse proxy

`mrrp` is a small reverse proxy for mihomo or Clash rule files.

It accepts requests like `/Google` or `/Google.list`, transforms the requested rule name, tries one or more configured upstreams, and returns the first matching rule content.

The service supports:

- Multiple upstreams
- URL and local file upstreams
- Rule name transforms with regular expressions
- Per-upstream proxy settings for URL upstreams
- Per-upstream timeout and request headers for URL upstreams
- Per-upstream comment filtering
- Per-upstream in-memory caching
- Two upstream selection modes: `race` and `sequential`

## Build

```bash
cargo build --release
```

## Run

By default, `mrrp` looks for `config.yml` in the current working directory.

```bash
cargo run --release
```

You can also specify a config file explicitly:

```bash
cargo run --release -- -f ./config.yml
```

Or if you run the compiled binary directly:

```bash
./target/release/mrrp -f ./config.yml
```

## Request Behavior

If you request:

- `/Google`
- `/Google.list`

then the raw request path segment is first turned into a rule name, then every `rule-transforms` entry is applied in order.

For example, with this transform:

```yml
rule-transforms:
  - pattern: "\\.list$"
    replace: ""
```

both `Google` and `Google.list` become the same final rule name: `Google`.

That final rule name is then inserted into upstream templates through the `{rule}` placeholder.

## Example Configuration

```yml
server-ip: 0.0.0.0
server-port: 8044
upstream-mode: race

rule-transforms:
  - pattern: "\\.list$"
    replace: ""

upstreams:
  - type: url
    template: "https://raw.githubusercontent.com/blackmatrix7/ios_rule_script/refs/heads/master/rule/Clash/{rule}/{rule}.list"
    cache: 30m
    remove-comments: true
    timeout-ms: 5000
    headers:
      - "User-Agent: mrrp"

  - type: file
    template: "./rules/{rule}.list"
    cache: 30m
    remove-comments: true
```

## Configuration Reference

## `server-ip`

The IP address the HTTP server listens on.

Example:

```yml
server-ip: 0.0.0.0
```

Default:

```yml
server-ip: 0.0.0.0
```

## `server-port`

The TCP port the HTTP server listens on.

Example:

```yml
server-port: 8044
```

Default:

```yml
server-port: 8044
```

## `upstream-mode`

Controls how multiple upstreams are tried.

Allowed values:

- `race`
- `sequential`

### `race`

All upstreams are started at the same time.

The first upstream that successfully returns content wins, and its content is returned immediately.

This is the default mode.

Example:

```yml
upstream-mode: race
```

### `sequential`

Upstreams are tried one by one in the order they appear in `upstreams`.

The first successful upstream wins.

Example:

```yml
upstream-mode: sequential
```

Default:

```yml
upstream-mode: race
```

## `rule-transforms`

A list of regex replacements applied to the requested rule name in order.

Each item supports:

- `pattern`: a Rust regular expression
- `replace`: the replacement string

Example:

```yml
rule-transforms:
  - pattern: "\\.list$"
    replace: ""
```

This is useful when you want `/Google.list` and `/Google` to resolve to the same rule name.

If `rule-transforms` is omitted, no transform is applied.

## `upstreams`

An ordered list of upstream sources.

Each upstream must have a `type` and a `template`.

Supported types:

- `url`
- `file`

The `{rule}` placeholder inside `template` is replaced with the final transformed rule name.

Example:

```yml
upstreams:
  - type: url
    template: "https://example.com/{rule}.list"

  - type: file
    template: "/data/rules/{rule}.list"
```

If all upstreams fail to produce a successful result, the final response is usually `404`.

## Common Upstream Fields

These fields are supported by both `url` and `file` upstreams.

## `type`

Selects the upstream backend.

Example:

```yml
type: url
```

or:

```yml
type: file
```

## `template`

Defines where the rule should be loaded from.

`{rule}` is replaced with the transformed rule name.

URL example:

```yml
template: "https://raw.githubusercontent.com/blackmatrix7/ios_rule_script/refs/heads/master/rule/Clash/{rule}/{rule}.list"
```

File example:

```yml
template: "./rules/{rule}.list"
```

Another file example:

```yml
template: "/data/rules/{rule}/{rule}.list"
```

## `remove-comments`

Controls whether comment lines and blank lines are removed from the upstream response.

When enabled:

- Lines starting with `#` are removed
- Blank lines are removed

Example:

```yml
remove-comments: true
```

Disable it if you want to return upstream content exactly as-is:

```yml
remove-comments: false
```

Default:

```yml
remove-comments: true
```

## `cache`

Controls the in-memory cache TTL for that upstream.

Each upstream has its own cache.

Only successful results are cached.
Failed requests are not cached.

Supported units:

- `s` for seconds
- `m` for minutes
- `h` for hours
- `d` for days

If no unit is provided, the value is treated as milliseconds.

Examples:

```yml
cache: 500
cache: 5s
cache: 30m
cache: 1h
cache: 1d
```

Default:

```yml
cache: 30m
```

## URL Upstream Fields

These fields are only valid when `type: url` is used.

## `proxy`

Sets a per-upstream outbound proxy.

Supported formats:

- `http://127.0.0.1:7897`
- `socks5://127.0.0.1:7897`

Example:

```yml
proxy: "http://127.0.0.1:7897"
```

or:

```yml
proxy: "socks5://127.0.0.1:7897"
```

This field is invalid for `type: file`.

## `timeout-ms`

Sets a request timeout in milliseconds for a URL upstream.

Example:

```yml
timeout-ms: 5000
```

This field is invalid for `type: file`.

## `headers`

Sets default request headers for a URL upstream.

This field accepts either:

- A single string
- An array of strings

Each entry must use this format:

```text
Header-Name: value
```

Single header example:

```yml
headers: "User-Agent: mrrp"
```

Multiple headers example:

```yml
headers:
  - "User-Agent: mrrp"
  - "Accept: text/plain"
```

This field is invalid for `type: file`.

## File Upstreams

For `type: file`, the `template` is treated as a local filesystem path template.

Example:

```yml
upstreams:
  - type: file
    template: "./rules/{rule}.list"
```

If `rule` is `Google`, the service tries to read:

```text
./rules/Google.list
```

## Response Rules

The server behavior is:

- A successful upstream response returns `200`
- A missing rule returns `404`
- URL upstream `404` is treated as not found
- URL upstream `5xx` is also treated as not found
- Failed records are not cached
- Only successful responses are cached

Network-level failures such as connection errors or timeouts are treated as upstream unavailability internally.

## Notes

- The service only handles one path segment like `/Google` or `/Google.list`
- Rule transforms are fully configuration-driven
- Comment filtering happens per upstream, not globally
- In `race` mode, all upstreams are attempted concurrently
- In `sequential` mode, upstream order matters directly
