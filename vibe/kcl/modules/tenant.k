import .platform as p
import .env as env

schema WorkloadInput:
    app_name: str
    image: str
    port: int = 8080
    replicas?: int
    env_vars?: {str: str} = {}
    cpu_request?: str
    memory_request?: str
    cpu_limit?: str
    memory_limit?: str
    ingress_host: str

schema TenantInput:
    tenant: str
    owner: str
    cost_center: str
    environment: env.Environment
    namespace_suffix?: str
    quota_cpu_requests?: str
    quota_memory_requests?: str
    quota_cpu_limits?: str
    quota_memory_limits?: str
    workloads: [WorkloadInput]

labels = lambda tenant_name, environment_name, owner, cost_center {
    {
        "app.kubernetes.io/managed-by": "kcl"
        "platform.example.com/tenant": tenant_name
        "platform.example.com/environment": environment_name
        "platform.example.com/owner": owner
        "platform.example.com/cost-center": cost_center
    }
}

is_set = lambda value {
    value != Undefined and value != None
}

pick = lambda value, default_value {
    value if is_set(value) else default_value
}

namespace_name = lambda t {
    "${t.tenant}-${t.environment.name}" if not is_set(t.namespace_suffix) else "${t.tenant}-${t.namespace_suffix}"
}

merged_labels = lambda t, app_name {
    labels(t.tenant, t.environment.name, t.owner, t.cost_center) | {
        "app.kubernetes.io/name": app_name
    }
}

replica_count = lambda environment, requested {
    environment.min_prod_replicas if environment.name == "prod" and (not is_set(requested) or requested < environment.min_prod_replicas) else pick(requested, environment.default_replicas)
}

workload_env = lambda t, w {
    {
        "TENANT": t.tenant
        "ENVIRONMENT": t.environment.name
        "APP_NAME": w.app_name
    } | w.env_vars
}

env_list = lambda env_vars {
    [{"name": k, "value": v} for k, v in env_vars]
}

quota = lambda t {
    p.resource_quota(
        "tenant-quota",
        namespace_name(t),
        labels(t.tenant, t.environment.name, t.owner, t.cost_center),
        {
            "requests.cpu": pick(t.quota_cpu_requests, t.environment.quota_cpu_requests)
            "requests.memory": pick(t.quota_memory_requests, t.environment.quota_memory_requests)
            "limits.cpu": pick(t.quota_cpu_limits, t.environment.quota_cpu_limits)
            "limits.memory": pick(t.quota_memory_limits, t.environment.quota_memory_limits)
            "pods": "100"
            "services": "20"
        }
    )
}

limit_range = lambda t {
    p.limit_range(
        "tenant-default-limits",
        namespace_name(t),
        labels(t.tenant, t.environment.name, t.owner, t.cost_center),
        [
            {
                "type": "Container"
                "default": {
                    "cpu": t.environment.default_cpu_limit
                    "memory": t.environment.default_memory_limit
                }
                "defaultRequest": {
                    "cpu": t.environment.default_cpu_request
                    "memory": t.environment.default_memory_request
                }
            }
        ]
    )
}

network_policy = lambda t {
    p.network_policy(
        "tenant-default-deny",
        namespace_name(t),
        labels(t.tenant, t.environment.name, t.owner, t.cost_center),
        {
            "podSelector": {}
            "policyTypes": ["Ingress", "Egress"]
            "ingress": [
                {
                    "from": [
                        {
                            "namespaceSelector": {
                                "matchLabels": {
                                    "kubernetes.io/metadata.name": "ingress-system"
                                }
                            }
                        }
                    ]
                }
            ]
            "egress": [
                {
                    "to": [
                        {
                            "namespaceSelector": {}
                        }
                    ]
                }
            ]
        }
    )
}

service_account = lambda t, w {
    p.service_account("${w.app_name}-sa", namespace_name(t), merged_labels(t, w.app_name))
}

config_map = lambda t, w {
    p.config_map("${w.app_name}-config", namespace_name(t), merged_labels(t, w.app_name), workload_env(t, w))
}

deployment = lambda t, w {
    p.deployment(
        w.app_name,
        namespace_name(t),
        merged_labels(t, w.app_name),
        {
            "replicas": replica_count(t.environment, w.replicas)
            "selector": {
                "matchLabels": {
                    "app.kubernetes.io/name": w.app_name
                }
            }
            "template": {
                "metadata": {
                    "labels": merged_labels(t, w.app_name)
                }
                "spec": {
                    "serviceAccountName": "${w.app_name}-sa"
                    "containers": [
                        {
                            "name": w.app_name
                            "image": w.image
                            "ports": [
                                {
                                    "containerPort": w.port
                                }
                            ]
                            "env": env_list(workload_env(t, w))
                            "resources": {
                                "requests": {
                                    "cpu": pick(w.cpu_request, t.environment.default_cpu_request)
                                    "memory": pick(w.memory_request, t.environment.default_memory_request)
                                }
                                "limits": {
                                    "cpu": pick(w.cpu_limit, t.environment.default_cpu_limit)
                                    "memory": pick(w.memory_limit, t.environment.default_memory_limit)
                                }
                            }
                            "securityContext": {
                                "allowPrivilegeEscalation": False
                                "readOnlyRootFilesystem": True
                                "runAsNonRoot": True
                                "capabilities": {
                                    "drop": ["ALL"]
                                }
                            }
                        }
                    ]
                }
            }
        }
    )
}

service = lambda t, w {
    p.service(
        w.app_name,
        namespace_name(t),
        merged_labels(t, w.app_name),
        {
            "selector": {
                "app.kubernetes.io/name": w.app_name
            }
            "ports": [
                {
                    "port": 80
                    "targetPort": w.port
                }
            ]
        }
    )
}

ingress = lambda t, w {
    p.ingress(
        "${w.app_name}-ingress",
        namespace_name(t),
        merged_labels(t, w.app_name),
        {
            "ingressClassName": t.environment.ingress_class
            "rules": [
                {
                    "host": w.ingress_host
                    "http": {
                        "paths": [
                            {
                                "path": "/"
                                "pathType": "Prefix"
                                "backend": {
                                    "service": {
                                        "name": w.app_name
                                        "port": {
                                            "number": 80
                                        }
                                    }
                                }
                            }
                        ]
                    }
                }
            ]
        }
    )
}

pdb = lambda t, w {
    p.pdb(
        "${w.app_name}-pdb",
        namespace_name(t),
        merged_labels(t, w.app_name),
        {
            "minAvailable": t.environment.pdb_min_available
            "selector": {
                "matchLabels": {
                    "app.kubernetes.io/name": w.app_name
                }
            }
        }
    )
}

workload_resources = lambda t, w {
    [
        service_account(t, w),
        config_map(t, w),
        deployment(t, w),
        service(t, w),
        ingress(t, w),
        pdb(t, w),
    ]
}

build_tenant = lambda t {
    [
        p.namespace(namespace_name(t), labels(t.tenant, t.environment.name, t.owner, t.cost_center)),
        quota(t),
        limit_range(t),
        network_policy(t),
    ] + [resource for w in t.workloads for resource in workload_resources(t, w)]
}
