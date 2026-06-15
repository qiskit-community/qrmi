# QRMI K8s Operator

This file is handwritten.

Idea is to enable QRMI workloads to run on k8s while delegating resource acquisition and management to the orchestrator, much in the same way as with the SPANK plugin.

Challenges:
1. Although Jobs are supported, kubernetes workloads are typically long-lived deployments. We have to choose the workloads we want to support
2. Cleanup and freeing of resources. k8s Jobs stick around after they've finished executing, so resources cannot be tied to the lifetime of a job.
3. How to conform to the "everything is an API resource" k8s reconciliation loop architecture
4. Avoiding overcomplicating the process of submitting a job 

## How it works birds-eye view

We define two custom resource types. These extend the k8s API and are implemented by a program called an operator (sometimes also referred to as a controller).

### Custom Resources

First, the `QuantumResource`. This is the equivalent of the SPANK plugin's [qrmi_config.json](https://github.com/qiskit-community/spank-plugins/blob/main/plugins/spank_qrmi/qrmi_config.json.example). 

```
apiVersion: quantum.qrmi.io/v1alpha1
kind: QuantumResource
metadata:
  name: ab-emu-1q-lescanne-2020
spec:
  resourceType: alice-bob-felis
  resourceId: ab_emu_1q_lescanne_2020
  envVars:
    QRMI_AB_FELIS_BASE_ENDPOINT: "https://api-staging.alice-bob.com"
  secretRefs:
    - secretName: alice-bob-felis-credentials
      secretKey: api-key
      envVarName: QRMI_AB_FELIS_API_KEY
```

This YAML defines Quantum Resource `ab_emu_1q_lescanne_2020` of type `alice-bob-felis`, and tells the operator about the appropriate env vars it will need to set when providing said resource to a workload. These are cluster-scoped resources, meaning they don't belong to any particular namespace. 

Second, the `QuantumResourceClaim`. This is perhaps the equivalent of Slurm directives when submitting a quantum job. 

```
apiVersion: quantum.qrmi.io/v1alpha1
kind: QuantumResourceClaim
metadata:
  name: ab-felis-claim
  namespace: platform-qrmi
spec:
  quantumResource: ab-emu-1q-lescanne-2020
  ttl: 3600
```

It tells the controller to acquire a Quantum Resource and inject the relevant environment variables into a kubernetes secret to be consumed by the workload. It can be in one of four phases:

- Pending: waiting for `qrmi.acquire()` to succeed. Maybe resource is locked
- Bound: token is acquired, secret containing token and the other relevant env vars exists
- Released: `qrmi.release()` has been called and cleanup is in progress. Never in this state for very long
- Failed. Not yet hooked up. Should add a max retries so we dont acquire on an infinite loop

This is the core of the architecture. The controller continuously monitors these resources and reconciles their state.

### Job Annotations

Without annotations, a job, such as the following:

```
apiVersion: batch/v1
kind: Job
metadata:
  name: alice-bob-felis-example
  namespace: platform-qrmi
spec:
  ttlSecondsAfterFinished: 300
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: felis-example
          image: felis-qrmi-example:latest
          imagePullPolicy: IfNotPresent
          envFrom:
            - secretRef:
                name: qrc-job-alice-bob-felis-example
```

is submittted to the k8s API directly following the submission `QuantumResourceClaim` which provides the job with a `Secret` from which it obtains the environment variables it needs. This `Secret`'s name can be customized or defaults to a predictable name derivable from the `Job` name.

This is all very well if the secret exists before the job is submitted, but definitions are often bundled together and created at the same time. It's also a bit of a PITA to submit a claim with every job. For these reasons the controller also supports automatic creation of the claim via monitoring for annotations. When the controller sees a job with the following annotation:

```
annotations:
  quantum.qrmi.io/resource: ab-emu-1q-lescanne-2020
```

it applies the `spec.suspend: true` to the job which tells the k8s job controller not to hold off creating the underlying pod workload. This gives the QRMI controller time to create the secret associated with a claim. Once it has done so, the suspended status is removed and things can procede as usual. In reality, things may function without this mechanism as the kubelet will retry on a backoff loop, but this logic handles things cleanly.

So in terms of this operator, kubernetes jobs are first-class citizens. Other workload types can manually create their claims for now, but the reality is that quantum workloads are overwhelmingly likely to be jobs.

The following annotations customize the other `QuantumResourceClaim` parameters.

```
annotations:
  quantum.qrmi.io/ttl: 500
  quantum.qrmi.io/secret-name: xxx
```

The secret name defaults to `qrc-<job-name>` if not specified. This be referenced explicitly in the job.

### Cleaning up 

Quantum Resources are freed when a `QuantumResourceClaim` is deleted. The way this behaviour is implemented is through finalizers. The controller adds a finalizer like this:

```
apiVersion: quantum.qrmi.io/v1alpha1
  kind: QuantumResourceClaim
  metadata:
    finalizers:
    - quantum.qrmi.io/finalizer
...
```
which causes deletion of the claim to wait for the QRMI controller to execute its cleanup logic i.e. `qrmi.release()` and remove the finalizer.

There are four main ways a `QuantumResourceClaim` might be deleted. One is obviously manual deletion.

The second is via cascading owner references. When a claim is created by the controller in response to an annotation, it is created with a reference to the requesting job

```
ownerReferences:
    - apiVersion: batch/v1
        blockOwnerDeletion: true
        controller: true
        kind: Job
        name: alice-bob-felis-example
        uid: c00c6c2a-9610-4444-b336-e37a1bb1a6ab
```

However, this is insufficent as k8s Jobs outlive the runtime of the workloads. We want to release Quantum Resources as soon as they've served their purpose. To achieve this, the controller hunts for jobs for `status.completionTime` or `status.failed` and deletes their claims. 

Lastly, the controller will delete claims when they exceed their `ttl` field.

## Misc comments/questions

- the operator itself is implemented in rust using [kube-rs](https://github.com/kube-rs/kube), which is decently mature. Normally operators are implemented in Go. We could have gone that route and consumed the C bindings.
- Should QuantumResource API define the parameters of the various quantum resources explicitly, then translate them into the appropriate env vars, rather than just allowing the user to set env vars which it passes on with a prefix?
- Are `QuantumResourceClaims` necessary, can't the controller just read and respond annotations directly? This is where we're conforming to existing models. It also helps with visibility.
- high availability of the operator
- implementation in Rust vs Go or another language
- Could've used mutating admissions webhook rather than secrets, or even init-containers. Thing is that webhooks don't solve the problem because you'd still need to create secrets, they'd just save the user from having to specify the `envFrom:` field as it would do that for them. But the cost is TLS cert management.
- local vs remote QPUs
- `QuantumJob` resource type? Not very extensible
- Reconciliation triggers when:
    - A watched object changes (watch event from API server)
    - A related object changes — e.g. kube-rs watches() configuration lets you trigger reconciliation of a parent when a child changes
    - Explicit Action::requeue() timer fires
    - Periodic full resync
- Bug: currently can only execute from operator namespace as operator searches for qr secret in the tenant namespace