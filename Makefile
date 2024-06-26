info:
	dust && tokei

unuse:
	cargo +nightly udeps --all-targets

test:
	cargo test

run:
	cargo run

repl:
	cargo run -- --port 6380 --replicaof "127.0.0.1 6379"

watch:
	cargo watch -x run

release:
	cargo run --release

update:
	rustup update && cargo outdated && cargo update && cargo upgrade && cargo audit

clean:
	cargo sweep --time 7 && cargo sweep --toolchains nightly-aarch64-apple-darwin && cargo sweep --installed
