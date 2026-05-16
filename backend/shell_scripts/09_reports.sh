#!/usr/bin/env bash
# ==============================================================
# Test: Reports (Achievement Report, Completion Dashboard)
# ==============================================================
source "$(dirname "$0")/_common.sh"
login_all

header "Reports"

call GET /reports/achievement "$MGR_T"
[ "$HTTP_CODE" = "200" ] && pass "Achievement report (JSON)" || fail "Achievement report" "HTTP $HTTP_CODE"

call GET /reports/completion-dashboard "$MGR_T"
[ "$HTTP_CODE" = "200" ] && pass "Completion dashboard" || fail "Dashboard" "HTTP $HTTP_CODE"

call GET "/reports/achievement?format=excel" "$MGR_T"
[ "$HTTP_CODE" = "200" ] && pass "Achievement export (Excel)" || fail "Achievement Excel" "HTTP $HTTP_CODE"

call GET "/reports/completion-dashboard?format=excel" "$MGR_T"
[ "$HTTP_CODE" = "200" ] && pass "Dashboard export (Excel)" || fail "Dashboard Excel" "HTTP $HTTP_CODE"

summary "Reports"
exit $((FAILED > 0 ? 1 : 0))
