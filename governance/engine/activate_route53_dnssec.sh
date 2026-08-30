#!/bin/bash
# activate_route53_dnssec.sh

DOMAIN="catedral.os"
HOSTED_ZONE_ID=$(aws route53 list-hosted-zones --query "HostedZones[?Name=='$DOMAIN.'].Id" --output text | cut -d'/' -f3)

# 1. Criar KMS Key para assinatura DNSSEC
KMS_KEY_ID=$(aws kms create-key \
    --description "DNSSEC key for $DOMAIN" \
    --key-usage SIGN_VERIFY \
    --customer-master-key-spec ECC_NIST_P256 \
    --query 'KeyMetadata.KeyId' \
    --output text)

# 2. Ativar DNSSEC Signing na zona
aws route53 enable-hosted-zone-dnssec \
    --hosted-zone-id $HOSTED_ZONE_ID \
    --delegation-signer-name $DOMAIN

# 3. Criar chave de assinatura
aws route53 create-key-signing-key \
    --hosted-zone-id $HOSTED_ZONE_ID \
    --key-signing-key-name "KSK-$DOMAIN" \
    --kms-key-id $KMS_KEY_ID

# 4. Obter DS records
aws route53 get-dnssec --hosted-zone-id $HOSTED_ZONE_ID

echo "✅ DNSSEC ativado para $DOMAIN no Route 53"