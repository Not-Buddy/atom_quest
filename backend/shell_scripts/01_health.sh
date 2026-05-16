#!/usr/bin/env bash
# ==============================================================
# Test: Health Check
# ==============================================================
source "$(dirname "$0")/_common.sh"

header "Health Check"
HC=$(curl $CURL_OPTS "$BASE_URL/health")
[ -n "$HC" ] && echo "  Response: $HC" && pass "Health endpoint responds" || fail "Health" "no response"

summary "Health"
exit $((FAILED > 0 ? 1 : 0))
