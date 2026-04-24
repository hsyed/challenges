make_metadata = lambda name, namespace, labels {
    {
        "name": name
        "labels": labels
    } | ({"namespace": namespace} if namespace != None else {})
}

namespace = lambda name, labels {
    {
        "apiVersion": "v1"
        "kind": "Namespace"
        "metadata": make_metadata(name, None, labels)
    }
}

resource_quota = lambda name, namespace, labels, hard {
    {
        "apiVersion": "v1"
        "kind": "ResourceQuota"
        "metadata": make_metadata(name, namespace, labels)
        "spec": {
            "hard": hard
        }
    }
}

limit_range = lambda name, namespace, labels, limits {
    {
        "apiVersion": "v1"
        "kind": "LimitRange"
        "metadata": make_metadata(name, namespace, labels)
        "spec": {
            "limits": limits
        }
    }
}

network_policy = lambda name, namespace, labels, spec {
    {
        "apiVersion": "networking.k8s.io/v1"
        "kind": "NetworkPolicy"
        "metadata": make_metadata(name, namespace, labels)
        "spec": spec
    }
}

service_account = lambda name, namespace, labels {
    {
        "apiVersion": "v1"
        "kind": "ServiceAccount"
        "metadata": make_metadata(name, namespace, labels)
    }
}

config_map = lambda name, namespace, labels, data {
    {
        "apiVersion": "v1"
        "kind": "ConfigMap"
        "metadata": make_metadata(name, namespace, labels)
        "data": data
    }
}

deployment = lambda name, namespace, labels, spec {
    {
        "apiVersion": "apps/v1"
        "kind": "Deployment"
        "metadata": make_metadata(name, namespace, labels)
        "spec": spec
    }
}

service = lambda name, namespace, labels, spec {
    {
        "apiVersion": "v1"
        "kind": "Service"
        "metadata": make_metadata(name, namespace, labels)
        "spec": spec
    }
}

ingress = lambda name, namespace, labels, spec {
    {
        "apiVersion": "networking.k8s.io/v1"
        "kind": "Ingress"
        "metadata": make_metadata(name, namespace, labels)
        "spec": spec
    }
}

pdb = lambda name, namespace, labels, spec {
    {
        "apiVersion": "policy/v1"
        "kind": "PodDisruptionBudget"
        "metadata": make_metadata(name, namespace, labels)
        "spec": spec
    }
}
