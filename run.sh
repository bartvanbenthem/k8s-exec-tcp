cargo build

./target/debug/k8s-pod-exec-tcp-check --ports 22 443 8080 80 \
    --image alpine \
    --hosts 172.22.128.32 172.22.128.33 172.22.128.34 172.22.128.32 172.22.128.99 \
    --max-connections 10 \
    --namespace test