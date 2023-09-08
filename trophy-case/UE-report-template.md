```toml
[advisory]
id = "RUSTSEC-0000-0000"
package = ""
date = ""
url = ""
informational = "unsound"
keywords = ["type-confusion"]

[versions]
patched = []
```

# Allows uninitialized memory exposure in safe function
The safe function `func` allows users to cast arbitrary types as bytes. If user provides a `struct` type with padding bytes, it could violate the safety guarantee of `func` and expose the uninitialized memory.