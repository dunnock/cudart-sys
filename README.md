The file `src/generated.rs` was created with [bindgen](https://github.com/crabtw/rust-bindgen) using `build.rs`

CUDA Runtime bindings

# Cudart include files and library files located by following proirities:

1. CUDA_PATH with suffixes:
- `lib64` or `lib/x64` suffix for library 
- `include` suffix for headers 
2. pkg_config for `cuda` or `cudart` packages

List of libraries for linking can be customized via CUBLAS_LIBS env var, 
by default taking `cudart.lib`

Include file is defined in wrapper.h and is `<cuda_runtime_api.h>`.

## Static linking

When CUDA_STATIC parameter is set bindgen will try to link statically.





