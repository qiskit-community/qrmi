# IBM Quantum Compute Service example

This example targets proto/k8s-operator. The prototype uses the legacy
qiskit-runtime-service resource type and QRMI_IBM_QRS_* variable names.

The operator acquires an IBM resource and creates a Secret containing
backend-scoped credentials and an acquisition token. The example Job
checks their presence without displaying their values.

No quantum circuit is submitted.

## Prerequisites

Build and install the operator, CRDs and RBAC using operator/README.md.
Make the operator image available to the Kubernetes nodes.

An IBM service instance, IAM API key and access to ibm_fez are required.
For another backend, update resourceId in ibm-qcs-qr.yaml and
QRMI_JOB_QPU_RESOURCES in ibm-qcs-job.yaml consistently.


## Credentials

Create files outside the repository containing the IAM API key and
service CRN, without trailing newlines. Do not commit these files.
Create the Secret in the Job/claim namespace:

    kubectl create namespace qrmi-example
    kubectl create secret generic ibm-qcs-credentials -n qrmi-example \
      --from-file=api-key=/secure/path/api-key \
      --from-file=service-crn=/secure/path/service-crn

## Run

From the repository root:

    kubectl apply -f operator/deploy/resources/ibm-qcs-qr.yaml
    kubectl apply -n qrmi-example -f operator/deploy/resources/ibm-qcs-job.yaml
    kubectl wait -n qrmi-example --for=condition=complete job/ibm-qcs-example --timeout=180s
    kubectl logs -n qrmi-example job/ibm-qcs-example
    kubectl logs -n platform-qrmi deployment/qrmi-operator --since=10m
    kubectl get quantumresourceclaims -n qrmi-example
    kubectl get secret -n qrmi-example qrc-job-ibm-qcs-example --ignore-not-found

Expected output confirms that credentials and an acquisition token are
present without displaying their values. Verify successful resource release
in operator logs and removal of the claim and generated Secret.
Job completion alone does not prove release.

## Validation

Tested on Kubernetes v1.35.9 with containerd v2.3.6.
The operator uses this branch's QRMI Rust crate v0.20.0.
The image was built with Podman on RHEL x86_64.

- Successful Job: acquisition, credential injection and release passed.
- Application failure: container exited 42; release passed.
- Job deletion after claim binding: release and Secret cleanup passed.

The deletion test did not conclusively establish workload startup before
deletion. No quantum circuit was submitted; an optional circuit example
is not included.

## Prototype limitations

The claim controller logs release failures and continues finalizer cleanup.
Confirm release explicitly in operator logs.

Deletion produced NotFound reconciliation warnings after successful release.
The unavailable legacy direct-access backend returns an explicit error.

These tests cover resource lifecycle management, not quantum execution
or remote quantum-task cancellation.

## Cleanup

Delete example Jobs first. Keep the QuantumResource and source credential
Secret available until their claims finish cleanup and release is confirmed.

    kubectl delete job ibm-qcs-example -n qrmi-example
    kubectl wait -n qrmi-example --for=delete quantumresourceclaim/job-ibm-qcs-example --timeout=90s
    kubectl delete quantumresource ibm-fez
    kubectl delete secret ibm-qcs-credentials -n qrmi-example
