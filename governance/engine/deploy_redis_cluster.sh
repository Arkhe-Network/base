#!/bin/bash
# deploy_redis_cluster.sh

NAMESPACE="catedral-cache"

# 1. Adicionar repositório Bitnami
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo update

# 2. Criar namespace
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# 3. Deploy Redis Cluster
helm upgrade --install redis-catedral bitnami/redis \
    --namespace $NAMESPACE \
    --values redis-cluster-values.yaml \
    --set auth.password="${REDIS_PASSWORD}" \
    --set sentinel.master="mymaster" \
    --set replica.replicaCount=3 \
    --set sentinel.replicas=3 \
    --wait

# 4. Verificar deployment
kubectl get pods -n $NAMESPACE
kubectl get svc -n $NAMESPACE

echo "✅ Redis Cluster implantado com sucesso"
echo "Master: redis-catedral-master.$NAMESPACE.svc.cluster.local:6379"
echo "Sentinel: redis-catedral-sentinel.$NAMESPACE.svc.cluster.local:26379"