# QRMI Operator Architecture

Visual companion to [`DESIGN_OUTLINE.md`](./DESIGN_OUTLINE.md). The operator coordinates two
custom resources, annotated Jobs, a managed Secret, and the QRMI backend's `acquire()` /
`release()` calls.

## Resources at a glance

```mermaid
flowchart LR
    qr["QuantumResource<br/><i>cluster-scoped</i>"]:::crd
    qrc["QuantumResourceClaim<br/><i>namespaced</i>"]:::crd
    job["Job<br/><i>annotated</i>"]:::k8s
    secret["Secret<br/>qrc-&lt;claim&gt;"]:::k8s
    pod[Pod]:::k8s

    job -->|references| qr
    job -->|owns| qrc
    qrc -->|owns| secret
    pod -->|envFrom| secret

    classDef crd fill:#e3f2fd,stroke:#1565c0,color:#000;
    classDef k8s fill:#f1f8e9,stroke:#558b2f,color:#000;
```

## Happy path: how a claim gets bound

Two controllers run in the operator. The Job controller reacts to annotated Jobs; the Claim
controller does the actual `acquire()` and builds the Secret. Steps are numbered in order.

```mermaid
flowchart TB
    job["Job<br/>annotation:<br/>quantum.qrmi.io/resource"]:::k8s
    qrc[QuantumResourceClaim]:::crd
    secret["Secret qrc-&lt;claim&gt;"]:::k8s
    pod["Pod (envFrom)"]:::k8s
    backend["QRMI backend"]:::ext

    jc[Job controller]:::ctrl
    cc[Claim controller]:::ctrl

    jc -->|"1 · suspend=true"| job
    jc -->|"2 · create claim"| qrc
    cc -->|"3 · acquire token"| backend
    cc -->|"4 · create Secret"| secret
    cc -->|"5 · status=Bound"| qrc
    jc -->|"6 · suspend=false"| job
    job -->|"7 · creates"| pod
    pod -->|envFrom| secret

    classDef crd fill:#e3f2fd,stroke:#1565c0,color:#000;
    classDef k8s fill:#f1f8e9,stroke:#558b2f,color:#000;
    classDef ctrl fill:#fff3e0,stroke:#e65100,color:#000;
    classDef ext fill:#fce4ec,stroke:#ad1457,color:#000;
```

## Lifecycle sequence

```mermaid
sequenceDiagram
    actor U as User
    participant API as kube API
    participant JC as Job ctrl
    participant CC as Claim ctrl
    participant QRMI as QRMI backend

    U->>API: apply annotated Job

    API-->>JC: Job added
    JC->>API: suspend Job
    JC->>API: create Claim (ownerRef=Job)

    API-->>CC: Claim added (Pending)
    CC->>API: add finalizer
    CC->>QRMI: acquire()
    QRMI-->>CC: token
    CC->>API: create Secret (ownerRef=Claim)
    CC->>API: status=Bound

    API-->>JC: Claim Bound
    JC->>API: unsuspend Job
    API->>API: kubelet starts Pod (envFrom Secret)

    API-->>JC: Job completed / failed
    JC->>API: delete Claim

    API-->>CC: Claim deleting
    CC->>QRMI: release(token)
    CC->>API: delete Secret + remove finalizer
```

## Cleanup & deletion triggers

Deleting a `QuantumResourceClaim` runs the finalizer (`quantum.qrmi.io/finalizer`), which calls
`release()` and removes the Secret. Four things can trigger that deletion:

```mermaid
flowchart LR
    complete["Job complete / failed"] --> del[Delete Claim]
    manual["kubectl delete"] --> del
    cascade["Owning Job deleted<br/>(GC cascade)"] --> del
    ttl["TTL expired<br/>(status.expiresAt)"] --> del
    del --> release["finalizer:<br/>release() + drop Secret"]
```

