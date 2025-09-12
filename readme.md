kubectl get --raw /api/v1/nodes/kind-control-plane/proxy/metrics/cadvisor

kubectl get --raw /api/v1/nodes/<node-name>/proxy/stats/summary


the above is for the endpoint of cadvisor to get metrics.

normal single pod server
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml



high availability spawns more than one pod
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/high-availability-1.21+.yaml




kubectl -n kube-system patch deployment metrics-server --type='json'  -p='[{"op": "add", "path": "/spec/template/spec/containers/0/args/-", "value": "--kubelet-insecure-tls"}]'


kubectl rollout restart deployment metrics-server -n kube-system


worked so far:
use default kind cluster creation:
1. kind create cluster
2. pull the components.yaml file and add --kubelet-insecure-tls and change the secure port to 4443



now using k3d:
you can add or remove nodes at runtime (after cluster initialization)



