# rs-webhook-cli
CLI tool to call endpoints via template files.

## Usage
Given an `example.json` template in `./inventory/`:

```bash
webhook_cli -u "http://example.com" -i "Injected text" example
```

## Parameters
```bash
webhook_cli [OPTIONS] [WEBHOOK]
```

```bash
-l, --list      Lists all available webhooks
-s, --simulate  Do not send the actual request
-u <LINK>       URL value, replaces $URL on the template
-i <VALUE>      Value to inject, starting from $1...$n
-v, --verbose   Enables enhanced logging
-h, --help      Print help
-V, --version   Print version
```
