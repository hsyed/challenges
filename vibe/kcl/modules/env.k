schema Environment:
    name: "dev" | "prod"
    default_replicas: int
    min_prod_replicas: int
    quota_cpu_requests: str
    quota_memory_requests: str
    quota_cpu_limits: str
    quota_memory_limits: str
    default_cpu_request: str
    default_memory_request: str
    default_cpu_limit: str
    default_memory_limit: str
    pdb_min_available: str
    ingress_class: str

dev = Environment {
    name = "dev"
    default_replicas = 1
    min_prod_replicas = 1
    quota_cpu_requests = "4"
    quota_memory_requests = "8Gi"
    quota_cpu_limits = "8"
    quota_memory_limits = "16Gi"
    default_cpu_request = "200m"
    default_memory_request = "256Mi"
    default_cpu_limit = "500m"
    default_memory_limit = "512Mi"
    pdb_min_available = "50%"
    ingress_class = "nginx"
}

prod = Environment {
    name = "prod"
    default_replicas = 3
    min_prod_replicas = 3
    quota_cpu_requests = "16"
    quota_memory_requests = "32Gi"
    quota_cpu_limits = "32"
    quota_memory_limits = "64Gi"
    default_cpu_request = "500m"
    default_memory_request = "512Mi"
    default_cpu_limit = "1000m"
    default_memory_limit = "1Gi"
    pdb_min_available = "67%"
    ingress_class = "nginx"
}
