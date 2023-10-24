cargo build --release

./target/release/k8s-pod-exec-tcp-check --ports 22 443 8080 80 \
    --image alpine \
    --hosts 172.28.131.138 172.28.131.13 172.28.131.144 172.28.131.13 172.28.131.138 \
    --max-connections 15 \
    --namespace test