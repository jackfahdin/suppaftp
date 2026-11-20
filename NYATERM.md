# NyaTerm patch series

Base: upstream `3e2090a622e85613becd285710aa373a5b7fd26f` (suppaftp 13.0.0).
Branch: `nyaterm`.

## Configured implicit FTPS sockets

Add `connect_secure_implicit_with_stream`, with the existing implicit constructor
delegating to it. NyaTerm needs to establish TCP under its own deadline and attach
cancellation and read/write timeouts before TLS and the initial FTP greeting.
The change preserves the original constructor and TLS connector behavior.

Validation on Windows: `cargo test -p suppaftp --features native-tls,deprecated
implicit_socket_tests`; `cargo fmt --all -- --check`.
The tests cover socket options at TLS setup and a stalled greeting without an
external server or changes to the system certificate store.
