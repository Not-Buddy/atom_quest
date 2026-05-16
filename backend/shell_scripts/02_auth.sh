#!/usr/bin/env bash
# ==============================================================
# Test: Authentication (login, me, password reset)
# ==============================================================
source "$(dirname "$0")/_common.sh"

header "Auth — Login"
login_all

# Login each role
for role in EMP MGR ADM; do
  resp="${role}_R"; token="${role}_T"
  [ -n "${!token}" ] && pass "Login $role — got token" || fail "Login $role" "no token"
done

# GET /auth/me
for role in EMP MGR ADM; do
  token_var="${role}_T"
  call GET /auth/me "${!token_var}"
  body=$(get_body)
  [ "$HTTP_CODE" = "200" ] && pass "GET /auth/me as $role" || fail "GET /auth/me as $role" "HTTP $HTTP_CODE"
  rid=$(echo "$body" | jq -r '.role // empty')
  info "  $role role=${rid}"
done

# Invalid credentials
call POST /auth/login "" '{"email":"bad@demo.com","password":"wrong"}'
[ "$HTTP_CODE" = "401" ] && pass "Bad credentials → 401" || fail "Bad credentials" "expected 401, got $HTTP_CODE"

summary "Auth"
exit $((FAILED > 0 ? 1 : 0))
