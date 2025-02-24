#!/bin/sh

(cat <<'EOF'
error: no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait

error: `#[panic_handler]` function required, but not found

error: unwinding panics are not supported without std
  |
  = help: using nightly cargo, use -Zbuild-std with panic="abort" to avoid unwinding
  = note: since the core library is usually precompiled with panic="unwind", rebuilding your crate with panic="abort" may not be enough to fix the problem

error: could not compile `jsonrs` (lib) due to 3 previous errors
EOF
) &> /tmp/expected

cargo --color=never check -q --no-default-features --features=bindings &> /tmp/actual

if ! diff -q -Z /tmp/expected /tmp/actual ; then
    cat /tmp/actual
    exit 1
fi
