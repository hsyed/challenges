# KCL Multi-Tenant Kubernetes PoC

This repository is a proof of concept showing how KCL can model a serious multi-tenant Kubernetes platform with separate `dev` and `prod` environments.

It demonstrates:

- Environment overlays with stricter defaults in `prod`
- Multiple tenants with per-tenant namespaces, quotas, network policy, and workload config
- Shared platform policy encoded once and reused everywhere
- Promotion-style rendering where the same tenant shape is instantiated differently per environment
- Deterministic manifest generation suitable for GitOps workflows

## Layout

```text
.
├── README.md
├── environments
│   ├── dev
│   │   ├── acme
│   │   │   └── main.k
│   │   └── globex
│   │       └── main.k
│   └── prod
│       ├── acme
│       │   └── main.k
│       └── globex
│           └── main.k
├── modules
│   ├── env.k
│   ├── platform.k
│   └── tenant.k
└── rendered
    ├── dev-acme.yaml
    ├── dev-globex.yaml
    ├── prod-acme.yaml
    └── prod-globex.yaml
```

## Render examples

```bash
kcl run environments/dev/acme/main.k -S manifests -o rendered/dev-acme.yaml
kcl run environments/dev/globex/main.k -S manifests -o rendered/dev-globex.yaml
kcl run environments/prod/acme/main.k -S manifests -o rendered/prod-acme.yaml
kcl run environments/prod/globex/main.k -S manifests -o rendered/prod-globex.yaml
```

## Enterprise characteristics shown in this PoC

### 1. Shared controls, local overrides

The platform team defines:

- required labels
- baseline network isolation
- resource quota and limit range defaults
- workload security defaults
- ingress policy defaults

Each tenant can still override:

- namespace owner and cost center labels
- replica counts
- image versions
- ingress hostnames
- environment variables
- quota sizing

### 2. Dev vs prod without copy/paste

The environment layer changes behavior globally:

- `prod` enforces stronger quotas and higher minimum replicas
- `prod` defaults to stricter pod disruption budgets
- `dev` allows smaller workloads and lower resource allocations

This is the core argument for KCL in enterprise use: one typed model, many safe instantiations.

### 3. GitOps-friendly output

The KCL programs render plain Kubernetes manifests:

- `Namespace`
- `ResourceQuota`
- `LimitRange`
- `NetworkPolicy`
- `ServiceAccount`
- `ConfigMap`
- `Deployment`
- `Service`
- `Ingress`
- `PodDisruptionBudget`

That output can be committed and applied by Argo CD or Flux.

## Files to inspect

- [modules/platform.k](/home/hsyed/code/github.com/hsyed/kcl-testing/modules/platform.k)
- [modules/env.k](/home/hsyed/code/github.com/hsyed/kcl-testing/modules/env.k)
- [modules/tenant.k](/home/hsyed/code/github.com/hsyed/kcl-testing/modules/tenant.k)
- [environments/prod/acme/main.k](/home/hsyed/code/github.com/hsyed/kcl-testing/environments/prod/acme/main.k)

## What this proves

This PoC is not just templating a Deployment. It encodes policy, tenancy boundaries, environment promotion, and application delivery in a single KCL model. That is the pattern needed for enterprise Kubernetes platforms.

The next logical extensions would be:

- cluster-specific overlays
- RBAC role bindings per tenant team
- external secrets integration
- service mesh policy
- admission policy generation
- multi-region rollout targets
