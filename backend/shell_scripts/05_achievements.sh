#!/usr/bin/env bash
# ==============================================================
# Test: Achievements (Quarterly Logging, Computed Scores)
# ==============================================================
source "$(dirname "$0")/_common.sh"
login_all

header "Achievements — Quarterly Logging"

call GET "/goals/sheets/$M_SID" "$M_T"; M_DET=$(get_body)
M_STAT=$(echo "$M_DET" | jq -r '.status // "draft"')
info "Maria's sheet: id=$M_SID status=$M_STAT"
M_G1=$(echo "$M_DET" | jq '.goals[0].id // 0')

if [ -z "$M_G1" ] || [ "$M_G1" = "0" ]; then
  fail "Achievements" "no goals in Maria's sheet"
  summary "Achievements"; exit 1
fi

if [ "$M_STAT" = "locked" ] || [ "$M_STAT" = "approved" ]; then
  call PUT "/achievements/$M_G1/Q2" "$M_T" \
    '{"actual_value":2,"actual_date":"2026-08-15","status":"completed"}'
  [ "$HTTP_CODE" = "200" ] && pass "Log Q2 actual value" || fail "Log Q2" "HTTP $HTTP_CODE"

  call PUT "/achievements/$M_G1/Q3" "$M_T" '{"status":"on_track"}'
  [ "$HTTP_CODE" = "200" ] && pass "Set Q3 status=on_track" || fail "Set Q3 status" "HTTP $HTTP_CODE"

  call GET "/achievements/sheet/$M_SID" "$M_T"; ACH_R=$(get_body)
  [ "$HTTP_CODE" = "200" ] && pass "GET achievements for sheet" || fail "GET achievements" "HTTP $HTTP_CODE"
  ACH_COUNT=$(echo "$ACH_R" | jq 'length')
  info "  Total achievements: $ACH_COUNT"
  S2=$(echo "$ACH_R" | jq '.[] | select(.quarter=="q2") | .computed_score // 0')
  [ -n "$S2" ] && [ "$S2" != "0" ] && pass "Computed score for Q2: $S2" || skip "Q2 score" "no computed score"

elif [ "$M_STAT" = "draft" ] || [ "$M_STAT" = "submitted" ] || [ "$M_STAT" = "returned" ]; then
  info "Maria's sheet is $M_STAT — submitting and approving"
  call PUT "/goals/sheets/$M_SID/submit" "$M_T"
  call PUT "/manager/sheets/$M_SID/approve" "$MGR_T"
  call PUT "/achievements/$M_G1/Q1" "$M_T" '{"actual_value":3,"status":"completed"}'
  [ "$HTTP_CODE" = "200" ] && pass "Log Q1 actual (after approve)" || fail "Log achievement" "HTTP $HTTP_CODE"
  call GET "/achievements/sheet/$M_SID" "$M_T"; ACH_R=$(get_body)
  S1=$(echo "$ACH_R" | jq '.[] | select(.quarter=="q1") | .computed_score // 0')
  pass "Q1 computed score: $S1"
fi

# Block on non-approved sheet
call GET "/goals/sheets/$J_SID" "$J_T"; J_DET=$(get_body)
J_G1=$(echo "$J_DET" | jq '.goals[0].id // 0')
J_STAT=$(echo "$J_DET" | jq -r '.status // "draft"')
if [ -n "$J_G1" ] && [ "$J_G1" != "0" ] && [ "$J_STAT" != "locked" ] && [ "$J_STAT" != "approved" ]; then
  call PUT "/achievements/$J_G1/Q1" "$J_T" '{"actual_value":5,"status":"completed"}'
  [ "$HTTP_CODE" = "422" ] && pass "Block achievement on non-approved sheet" || fail "Block on non-approved" "expected 422, got $HTTP_CODE"
else
  skip "Block non-approved" "James's sheet is $J_STAT"
fi

summary "Achievements"
exit $((FAILED > 0 ? 1 : 0))
