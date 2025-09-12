### APIS for metrics

raw prometheus like metrics output (point in timej):
kubectl get --raw /api/v1/nodes/<node-name>/proxy/metrics/cadvisor

json summarized output (point in time):
kubectl get --raw /api/v1/nodes/<node-name>/proxy/stats/summary


### Endpoints to get metrics server
normal single pod server
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml


high availability spawns more than one pod
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/high-availability-1.21+.yaml

worked so far for kind:
use default kind cluster creation:
1. kind create cluster
2. pull the components.yaml file and add --kubelet-insecure-tls and change the secure port to 4443 both as args and container port and run it


now using k3d:
you can add or remove nodes at runtime (after cluster initialization)
metrics server and hpa works but for hpa you have to use the older api version
