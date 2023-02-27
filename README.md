# moss-wasm

wasm serverless platform

## generate orm entity

```bash
cargo install sea-orm-cli
sea-orm-cli generate entity -u=mysql://root:@localhost/moss-serverless -o moss-lib/service/src/entity
```
