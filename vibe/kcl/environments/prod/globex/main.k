import modules.env as env
import modules.tenant as tenants

manifests = tenants.build_tenant(tenants.TenantInput {
    tenant = "globex"
    owner = "analytics-team"
    cost_center = "cc-2040"
    environment = env.prod
    workloads = [
        tenants.WorkloadInput {
            app_name = "analytics-api"
            image = "ghcr.io/example/analytics-api:3.2.0"
            port = 8090
            replicas = 5
            ingress_host = "analytics.globex.example.com"
            env_vars = {
                "LOG_LEVEL" = "warn"
                "CACHE_MODE" = "redis"
            }
            cpu_request = "1500m"
            memory_request = "2Gi"
            cpu_limit = "3000m"
            memory_limit = "4Gi"
        },
        tenants.WorkloadInput {
            app_name = "reporting-ui"
            image = "ghcr.io/example/reporting-ui:3.2.0"
            port = 3000
            replicas = 3
            ingress_host = "reports.globex.example.com"
            env_vars = {
                "API_BASE_URL" = "https://analytics.globex.example.com"
            }
            cpu_request = "500m"
            memory_request = "512Mi"
            cpu_limit = "1000m"
            memory_limit = "1Gi"
        },
    ]
})
