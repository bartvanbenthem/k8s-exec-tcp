#sudo apt update
#sudo apt install musl-tools

#cargo check
cargo fmt
cargo test
#cargo tarpaulin --ignore-tests
#cargo clippy
#cargo audit

#cargo build --release
export CC=musl-gcc
cargo build --target x86_64-unknown-linux-musl --release

./target/x86_64-unknown-linux-musl/release/k8stcp --ports 22 443 53 37 \
    --image alpine \
    --hosts 216.146.35.35 132.163.97.6 129.6.15.28 dns.google \
    --max-connections 20 \
    --service-account default \
    --namespace test
