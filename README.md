# k8s-exec-tcp
This test project aims to evaluate the effectiveness of Rust for developing command-line interface (CLI) tools for Kubernetes. The primary focus is a CLI client designed to perform concurrent TCP checks from within an authenticated Kubernetes cluster.

## usage
```bash
USAGE:
    k8s-pod-exec-tcp-check [OPTIONS] --ports <ports>...

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --max-connections <connections>    Port that remote host listens on
    -h, --hosts <hosts>...                 Space separated list of hosts
    -i, --image <image>                    Override alpine container image
    -n, --namespace <namespace>            Kubernetes Namespace
    -p, --ports <ports>...                 Port that remote host listens on
```
