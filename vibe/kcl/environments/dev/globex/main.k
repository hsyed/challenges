import modules.env as env
import modules.tenant as tenants

manifests = tenants.build_tenant(tenants.TenantInput {
    tenant = "globex"
    owner = "analytics-team"
    cost_center = "cc-2040"
    environment = env.dev
    workloads = [
        tenants.WorkloadInput {
            app_name = "analytics-api"
            image = "ghcr.io/example/analytics-api:3.2.0"
            port = 8090
            replicas = 1
            ingress_host = "analytics.dev.globex.example.com"
            env_vars = {
                "LOG_LEVEL" = "debug"
                "CACHE_MODE" = "disabled"
            }
        },
        tenants.WorkloadInput {
            app_name = "reporting-ui"
            image = "ghcr.io/example/reporting-ui:3.2.0"
            port = 3000
            replicas = 1
            ingress_host = "reports.dev.globex.example.com"
            env_vars = {
                "API_BASE_URL" = "https://analytics.dev.globex.example.com"
            }
        },
    ]
})
