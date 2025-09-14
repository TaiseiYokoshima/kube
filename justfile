check:
   @RUSTFLAGS="-Awarnings" cargo check --quiet 

run:
   @RUSTFLAGS="-Awarnings" cargo run


replica:
   @kubectl get --raw /apis/apps/v1/namespaces/default/replicasets/stress-test-f9c8c6c75 | jq -r '.metadata | "uid: \(.uid)\nname: \(.name)\napp: \(.labels.app)\npod-template-hash: \(.labels."pod-template-hash")"'

deploy:
   @kubectl get --raw /apis/apps/v1/namespaces/default/deployments/stress-test | jq -r ".metadata.uid"


