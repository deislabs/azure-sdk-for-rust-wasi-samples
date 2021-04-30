# Using Azure services from WASI modules

This repo contains samples for using the [Azure Rust SDK][rust-sdk] from WASI
modules, using the [`wasi-experimental-http`][wasi-experimental-http] crate. The
samples are slightly modified versions of those from the [upstream SDK
repository][rust-sdk]. Note that not all SDKs are compilable to `wasm32-wasi`,
and this repo should be updated as more compatible SDKs are added.

Currently, only the Blob Storage SDK has been tested, and the _very_ early work
in progress can be tracked [here][fork].

### Building

```
$ cargo target add wasm32-wasi
$ cargo build --target wasm32-wasi
```

At this point, the easiest way to test the resulting module is to execute it
using the helper binary that can be found [here][bin]:

```
$ wasmtime-http target/wasm32-wasi/debug/wasi-azure-tests.wasm --env STORAGE_MASTER_KEY=<master-key> --env STORAGE_ACCOUNT=<storage-account> -a https://<storage-account>.blob.core.windows.net
```

[rust-sdk]: https://github.com/Azure/azure-sdk-for-rust
[wasi-experimental-http]: https://github.com/deislabs/wasi-experimental-http/
[bin]: https://github.com/deislabs/wasi-experimental-http/pull/57
[fork]:
  https://github.com/radu-matei/azure-sdk-for-rust/tree/enable-wasi-experimental-http
