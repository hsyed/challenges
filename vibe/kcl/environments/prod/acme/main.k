import modules.env as env
import modules.tenant as tenants

manifests = tenants.build_tenant(tenants.TenantInput {
    tenant = "acme"
    owner = "payments-team"
    cost_center = "cc-1001"
    environment = env.prod
    quota_cpu_requests = "24"
    quota_memory_requests = "48Gi"
    quota_cpu_limits = "40"
    quota_memory_limits = "80Gi"
    workloads = [
        tenants.WorkloadInput {
            app_name = "billing-api"
            image = "ghcr.io/example/billing-api:1.4.2"
            port = 8080
            replicas = 4
            ingress_host = "billing.acme.example.com"
            env_vars = {
                "LOG_LEVEL" = "info"
                "FEATURE_FLAG_NEW_CHECKOUT" = "false"
            }
            cpu_request = "1000m"
            memory_request = "1Gi"
            cpu_limit = "2000m"
            memory_limit = "2Gi"
        },
        tenants.WorkloadInput {
            app_name = "billing-worker"
            image = "ghcr.io/example/billing-worker:1.4.2"
            port = 8081
            replicas = 3
            ingress_host = "worker.acme.example.com"
            env_vars = {
                "QUEUE_NAME" = "billing-prod"
            }
            cpu_request = "500m"
            memory_request = "1Gi"
            cpu_limit = "1000m"
            memory_limit = "2Gi"
        },
    ]
})
