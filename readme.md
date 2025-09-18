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


cardamon plan:
1. get deployment name from Cardamon.toml
2. query deployment (depl) uid.
3. get the replicaset via depl uid
4. get the pod-hash from the replicaset
5. watch the replicaset for pods count
6. one task watches replicaset to add or remove pods, one thread reads node sumarry, one thread filters the summary via the pods dsa
7. for each node there would be 2 tasks. One to query and get the string, and another to filter out the necessary cpu metrics
8. one task to gather all the metrics produced by each node and sum it and collect it into a buffer
9. each round of query is synchronized by a signal


get deployment ids
get all replicasets and their template hash


set up watchers per hash
set up watchers on deployment to spawn more replica watchers

