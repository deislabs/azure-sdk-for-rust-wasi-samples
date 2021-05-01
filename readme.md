# Using Azure services from WASI modules

This repo contains samples for using the [Azure Rust SDK][rust-sdk] from WASI
modules, using the [`wasi-experimental-http`][wasi-experimental-http] crate. The
samples are slightly modified versions of those from the [upstream SDK
repository][rust-sdk]. Note that not all SDKs are compilable to `wasm32-wasi`,
and this repo should be updated as more compatible SDKs are added.

The _very_ early work in progress can be tracked [here][fork].

### Building

```
$ cargo target add wasm32-wasi
$ cargo build --target wasm32-wasi --release --bin blob
$ cargo build --target wasm32-wasi --release --bin cosmos
$ cargo build --target wasm32-wasi --release --bin eventgrid
$ cargo build --target wasm32-wasi --release --bin iothub
```

At this point, the easiest way to test the resulting module is to execute it
using the helper binary that can be found [here][bin]:

```
$ wasmtime-http target/wasm32-wasi/release/blob.wasm --env STORAGE_MASTER_KEY=<master-key> --env STORAGE_ACCOUNT=<storage-account> -a https://<storage-account>.blob.core.windows.net

$ wasmtime-http target/wasm32-wasi/release/cosmos.wasm --env COSMOS_MASTER_KEY=<master-key> --env COSMOS_ACCOUNT=<cosmos-account> -a https://<cosmos-account>.documents.azure.com

$ wasmtime-http target/wasm32-wasi/release/eventgrid.wasm --env TOPIC_HOST_NAME=<full-topic-hostname> --env TOPIC_KEY=<topic-key> -a https://<topic>.<location>.eventgrid.azure.net

$ wasmtime-http target/wasm32-wasi/release/iothub.wasm --env IOTHUB_CONNECTION_STRING=<connection-string-in-quotes> -a https://<iothub-account>.azure-devices.net
```

[rust-sdk]: https://github.com/Azure/azure-sdk-for-rust
[wasi-experimental-http]: https://github.com/deislabs/wasi-experimental-http/
[bin]: https://github.com/deislabs/wasi-experimental-http/pull/57
[fork]:
  https://github.com/radu-matei/azure-sdk-for-rust/tree/enable-wasi-experimental-http
