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

./target/x86_64-unknown-linux-musl/release/k8stcp --ports 22 443 8080 80 \
    --image alpine \
    --hosts 192.168.63.64 172.28.131.13 172.28.131.144 172.28.131.13 192.168.63.65 \
    --max-connections 20 \
    --service-account default \
    --namespace test
