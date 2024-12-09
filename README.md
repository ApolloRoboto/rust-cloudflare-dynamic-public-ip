# rust-cloudflare-dynamic-public-ip

Update public ip in cloudflare's DNS records automatically. You never know when your ISP is pushing updates to your router and cycle your public IP, breaking DNS records, this is my solution.

## Build and test

```
cargo build
cargo test
```

## Run

Create a `.env` file with the following secrets:
```env
CLOUDFLARE_TOKEN=
CLOUDFLARE_ZONE_ID=123456,456789
```

```bash
# display help
cargo run -- --help

# get the current ip
cargo run -- current

# see the affected DNS records
cargo run -- check

# monitor changes and update cloudflare DNS record
cargo run -- monitor
```

### Docker

```
docker run --rm -it --env-file .env apollo-roboto/cfdpip:latest
```

## MQTT

MQTT can be configured with environment variables and is enabled with `MQTT_ENABLED=true`

All variables:

```env
MQTT_ENABLED
MQTT_HOST # required
MQTT_PORT # defaults to 1883
MQTT_ID # default will have a random id similar to cfdpip-xxxxxx
MQTT_BASE_TOPIC # defaults to cfdpip
```

### Topics

| Topic | Example Payload |
|-------|-----------------|
| `cfdpip/ipchange` | `{ "old": "1.2.3.4", "new": "1.2.3.5" }` |
