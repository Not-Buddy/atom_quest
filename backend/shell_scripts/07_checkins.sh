#!/usr/bin/env bash
# ==============================================================
# Test: Manager Check-ins
# ==============================================================
source "$(dirname "$0")/_common.sh"
login_all

header "Manager Check-ins"

call GET /manager/team/checkins "$MGR_T"
[ "$HTTP_CODE" = "200" ] && pass "View team check-ins" || fail "View check-ins" "HTTP $HTTP_CODE"

call POST "/manager/checkins/$M_SID" "$MGR_T" '{"quarter":"Q2","comment":"Solid Q2. Keep it up."}'
[ "$HTTP_CODE" = "200" ] && pass "Add check-in (Q2)" || fail "Add check-in Q2" "HTTP $HTTP_CODE"

call POST "/manager/checkins/$M_SID" "$MGR_T" '{"quarter":"Q5","comment":"Bad quarter"}'
[ "$HTTP_CODE" = "422" ] && pass "Invalid quarter Q5 → rejected" || fail "Invalid quarter" "expected 422, got $HTTP_CODE"

call POST "/manager/checkins/$M_SID" "$MGR_T" '{"quarter":"Q3","comment":"Q3 comment"}'
[ "$HTTP_CODE" = "200" ] && pass "Add check-in (Q3)" || fail "Add check-in Q3" "HTTP $HTTP_CODE"

summary "Check-ins"
exit $((FAILED > 0 ? 1 : 0))
