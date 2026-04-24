import modules.env as env
import modules.tenant as tenants

manifests = tenants.build_tenant(tenants.TenantInput {
    tenant = "acme"
    owner = "payments-team"
    cost_center = "cc-1001"
    environment = env.dev
    workloads = [
        tenants.WorkloadInput {
            app_name = "billing-api"
            image = "ghcr.io/example/billing-api:1.4.2"
            port = 8080
            replicas = 1
            ingress_host = "billing.dev.acme.example.com"
            env_vars = {
                "LOG_LEVEL" = "debug"
                "FEATURE_FLAG_NEW_CHECKOUT" = "true"
            }
        },
        tenants.WorkloadInput {
            app_name = "billing-worker"
            image = "ghcr.io/example/billing-worker:1.4.2"
            port = 8081
            replicas = 1
            ingress_host = "worker.dev.acme.example.com"
            env_vars = {
                "QUEUE_NAME" = "billing-dev"
            }
        },
    ]
})
