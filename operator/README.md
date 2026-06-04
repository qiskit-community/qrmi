# QRMI K8s Operator

This file is written by hand.

For outline of how it works see [DESIGN_OUTLINE.md](./DESIGN_OUTLINE.md)

## Build

Cargo build is done as part of building the operator OCI image

```
docker build -f operator/docker/operator/Dockerfile -t qrmi-operator:latest .
```

Build example:

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
kubectl apply -f operator/deploy/operator.yaml
```


## Configure a Quantum Resource

```
kubectl apply -f operator/deploy/resources/alice-bob-felis-qr.yaml
```

## Run a Job

```
kubectl apply -f operator/deploy/resources/alice-bob-felis-job-annotation.yaml
```
