default: (run)

alias r := run

log_level := "info"

run:
  RUST_LOG={{log_level}} cargo run --release
