RUSTFLAGS='-C target-cpu=native' cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release
