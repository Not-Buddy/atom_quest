#!/usr/bin/env bash
# ==============================================================
# AtomQuest — Shared Test Helpers
# Source this in individual test scripts:
#   source "$(dirname "$0")/_common.sh"
# ==============================================================
set +e +u

CURL_OPTS="-s --max-time 30"
BASE_URL="${BASE_URL:-http://localhost:10000}"
PASS="password123"

CALL_TMP=$(mktemp -t atomquest-test.XXXXXX)
trap "rm -f $CALL_TMP" EXIT

PASSED=0; FAILED=0; SKIPPED=0; RESULTS=()

info()  { printf "\e[34m[INFO]\e[0m  %s\n" "$*"; }
pass()  { printf "\e[32m[PASS]\e[0m  %s\n" "$*"; ((PASSED++)); RESULTS+=("PASS:$*"); }
fail()  { printf "\e[31m[FAIL]\e[0m  %s — %s\n" "$1" "$2"; ((FAILED++)); RESULTS+=("FAIL:$1"); }
skip()  { printf "\e[33m[SKIP]\e[0m  %s — %s\n" "$1" "$2"; ((SKIPPED++)); RESULTS+=("SKIP:$1"); }
header(){ printf "\n\e[1;36m━━━ %s ━━━\e[0m\n\n" "$1"; }

login() {
  curl $CURL_OPTS "$BASE_URL/auth/login" -X POST \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$1\",\"password\":\"${2:-$PASS}\"}"
}

# call: writes response body to $CALL_TMP, sets global $HTTP_CODE
# Usage: call METHOD PATH TOKEN [BODY]
call() {
  local method="$1" path="$2" token="$3" body="${4:-}"
  local out
  if [ -n "$body" ]; then
    out=$(curl $CURL_OPTS -w "\n%{http_code}" "$BASE_URL$path" \
      -X "$method" -H "Content-Type: application/json" \
      -H "Authorization: Bearer $token" -d "$body")
  else
    out=$(curl $CURL_OPTS -w "\n%{http_code}" "$BASE_URL$path" \
      -X "$method" -H "Authorization: Bearer $token")
  fi
  HTTP_CODE=$(printf "%s" "$out" | tail -1)
  printf "%s" "$out" | head -n -1 > "$CALL_TMP"
}

# get_body: reads the body from the temp file (after a call)
get_body() { cat "$CALL_TMP"; }

# ── Login all roles ──────────────────────────────────────────
login_all() {
  EMP_R=$(login "employee@demo.com"); EMP_T=$(echo "$EMP_R" | jq -r '.token')
  MGR_R=$(login "manager@demo.com"); MGR_T=$(echo "$MGR_R" | jq -r '.token')
  ADM_R=$(login "admin@demo.com");   ADM_T=$(echo "$ADM_R" | jq -r '.token')
  J_T=$(login "james@demo.com"  | jq -r '.token')
  M_T=$(login "maria@demo.com"  | jq -r '.token')

  for t in EMP_T MGR_T ADM_T J_T M_T; do
    [ -z "${!t}" ] && { echo "ABORT: $t empty"; exit 1; }
  done
  pass "All 5 users authenticated"

  call GET /admin/cycles "$ADM_T";   CYCLE=$(get_body | jq '.[0].id')
  call GET /admin/thrust-areas "$ADM_T"; TA_ID=$(get_body | jq '.[0].id')
  info "cycle=$CYCLE thrust_area=$TA_ID"

  call GET /goals/sheets "$EMP_T"; EMP_SID=$(get_body | jq '.[0].id // 0')
  call GET /goals/sheets "$J_T";   J_SID=$(get_body | jq '.[0].id // 0')
  call GET /goals/sheets "$M_T";   M_SID=$(get_body | jq '.[0].id // 0')
  info "Sheets: emp=$EMP_SID james=$J_SID maria=$M_SID"
}

summary() {
  local name="${1:-Test}"
  header "$name Results"
  echo "  ✅ Passed:  $PASSED"
  echo "  ❌ Failed:  $FAILED"
  echo "  ⏭️  Skipped: $SKIPPED"
  echo "  ─────────────────"
  echo "  📊 Total:   $((PASSED + FAILED + SKIPPED))"
  echo ""
  for r in "${RESULTS[@]}"; do
    case "${r%%:*}" in PASS) echo "  ✅ ${r#*:}" ;; FAIL) echo "  ❌ ${r#*:}" ;; SKIP) echo "  ⏭️  ${r#*:}" ;; esac
  done
  echo ""
}
