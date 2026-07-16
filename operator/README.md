# QRMI K8s Operator

This file is written by hand.

For outline of how it works see [DESIGN_OUTLINE.md](./DESIGN_OUTLINE.md)

## Build

Cargo build is done as part of building the operator OCI image

```
docker build -f operator/docker/operator/Dockerfile -t qrmi-operator:latest .
```

However if you want to build outside the container, you run

```
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl # pulls precompiled rust std lib
cargo build --release --target x86_64-unknown-linux-musl -p qrmi-operator
```

To build the example workload container:

```
docker build -f operator/docker/alice-bob-felis-example/Dockerfile -t felis-qrmi-example:latest .
```

These images need to be pullable by the cluster nodes. For this prototype I'm running `task push` which pushes directly to the cluster nodes (requires direct ssh access to nodes)

## Install

Have kubeconfig point to a k8s cluster where you have admin permissions.

Generate and deploy CRDs:
```
cargo run -p qrmi-operator -- generate-crds > operator/deploy/crds/crds.yaml
kubectl apply -f operator/deploy/crds/crds.yaml
```

Deploy Operator
```
export QRMI_NAMESPACE=platform-qrmi # or whatever namespace you want
kubectl apply -f operator/deploy/operator.yaml -n $QRMI_NAMESPACE
```


## Configure a Quantum Resource

The quantum resource reads the API key from a secret that needs to be created first:

```
export QRMI_AB_FELIS_API_KEY=<your_api_key>
kubectl create secret generic alice-bob-felis-credentials --from-literal=api-key="${QRMI_AB_FELIS_API_KEY}" -n platform-qrmi
kubectl apply -f operator/deploy/resources/alice-bob-felis-qr.yaml
```

## Run a Job

For now we'll use the same namespace the operator is installed in 

```
kubectl apply -f operator/deploy/resources/alice-bob-felis-job-annotation.yaml -n $QRMI_NAMESPACE
```

View output:

```
kubectl logs jobs/alice-bob-felis-example -n $QRMI_NAMESPACE
```
