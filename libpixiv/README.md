# libpixiv

Client library for [pixiv](https://pixiv.net)

## Usage

For now, no OAuth2 login will be provided, so please make sure to provide refresh tokens yourself.

## Testing

You can run the following command to test existing features:
```bash
PIXIV_REFRESH_TOKEN= cargo test --lib
```