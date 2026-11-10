# Run Project

This project combines multiple `crates` into one master crate. <br>
Thus, using `cargo run` won't work as `cargo` does not know which crate to run!

Specify the crate you want to run with:
```bash
cargo run --bin [crate]
```

For example, if you want to run the `kuhkey_cli` crate, run:
```bash
cargo run --bin kuhkey_cli
```
