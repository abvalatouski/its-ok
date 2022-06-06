# `its_ok`

Provides `ok` and `ok_unchecked` macros for replacing `?` with `unwrap` and
`unwrap_unchecked` calls.

# Example

```rust
use its_ok::ok;
use std::io::Write;

ok! {
    let mut buffer = Vec::new();
    buffer.write_all(b"bytes")?;
}

// The code above gets expanded into this.
let mut buffer = Vec::new();
buffer.write_all(b"bytes").unwrap();
```
