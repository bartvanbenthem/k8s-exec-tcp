# k8s-exec-tcp
Command-line interface (CLI) client designed to perform TCP checks from an authenticated Kubernetes cluster. These checks are carried out by temporarily initializing a pod within the specified namespace. This pod allows for the concurrent execution of checks on multiple remote targets. Targets are a combination of hosts and ports.

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