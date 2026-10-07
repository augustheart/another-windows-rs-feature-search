# another-windows-rs-feature-search

A better (in my opinion) `windows-rs` feature search.

When using the Windows API with Microsoft's `windows-rs` crate, we need to know which features to enable. There are two approaches: search for the API in the crate source and then find the feature tree, or use Microsoft's [windows-rs feature search](https://microsoft.github.io/windows-rs/features/). However, that page provides a poor experience. For example, if you want to search for `CreateFileW`, you type it into the input box, and it returns `fileapi`, `minwinbase`, and `winnt`. You add them to the `features` list for the `windows` dependency and compile, but you get errors. In fact, you need to enable `Win32`, `Win32_Storage`, `Win32_Storage_FileSystem`, and `Win32_Security` to use the API.

So I built this tool. It uses `syn` to analyze the required features from `.cargo/registry/src/index.crates.io-*/windows-<version>/` and then builds the correct feature mappings for APIs.

Known issues:

~~- Code needs cleanup; the tests currently only run on my machine.~~ add a static html page as search ui
- No search UI; so far, the tool only provides a JSON-generating feature.
- Not all APIs are recorded
- No release
- More to come
