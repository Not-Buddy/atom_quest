#!/usr/bin/env bash
# ==============================================================
# Test: Manager (Team sheets, Approve, Return, Edit, Shared)
# ==============================================================
source "$(dirname "$0")/_common.sh"
login_all

header "Manager Operations"

# List team sheets
call GET /manager/team/sheets "$MGR_T"
TEAM=$(get_body)
if [ "$HTTP_CODE" != "200" ]; then
  fail "List team sheets" "HTTP $HTTP_CODE"
  summary "Manager"; exit 1
fi
pass "List team sheets"

# Approve Alex's sheet
A_SID=$(echo "$TEAM" | jq '.[] | select(.user_id == 5) | .id // 0')
A_STAT=$(echo "$TEAM" | jq -r '.[] | select(.user_id == 5) | .status // "draft"')
info "Alex: id=$A_SID status=$A_STAT"

if [ "$A_STAT" = "submitted" ] || [ "$A_STAT" = "returned" ]; then
  call PUT "/manager/sheets/$A_SID/approve" "$MGR_T"
  [ "$HTTP_CODE" = "200" ] && pass "Approve sheet" || fail "Approve sheet" "HTTP $HTTP_CODE"
elif [ "$A_STAT" = "locked" ]; then
  pass "Sheet already locked"
else
  skip "Approve" "status=$A_STAT"
fi

# Return: submit James's sheet then return it
J_STAT=$(echo "$TEAM" | jq -r '.[] | select(.user_id == 7) | .status // "draft"')
info "James: sheet=$J_SID status=$J_STAT"

if [ "$J_STAT" = "draft" ]; then
  call PUT "/goals/sheets/$J_SID/submit" "$J_T"
  info "  Submitted James's sheet (HTTP $HTTP_CODE)"
fi

call PUT "/manager/sheets/$J_SID/return" "$MGR_T" '{"reason":"Revise targets for Q2"}'
[ "$HTTP_CODE" = "200" ] && pass "Return sheet with reason" || fail "Return sheet" "HTTP $HTTP_CODE"

# Manager inline edit goal
call GET "/goals/sheets/$EMP_SID" "$EMP_T"; A_DET=$(get_body)
A_G1=$(echo "$A_DET" | jq '.goals[0].id // 0')
if [ "$A_G1" -gt 0 ] 2>/dev/null; then
  call PUT "/manager/sheets/$EMP_SID/goals/$A_G1" "$MGR_T" '{"target_value":15,"weightage":40}'
  [ "$HTTP_CODE" = "200" ] && pass "Manager inline edit goal" || fail "Manager edit goal" "HTTP $HTTP_CODE"
else
  skip "Manager edit goal" "no goal in sheet $EMP_SID"
fi

# Shared goal push — find any draft sheet
DRAFT_SID=$(echo "$TEAM" | jq '.[] | select(.status == "draft") | .id // 0')
if [ -n "$DRAFT_SID" ] && [ "$DRAFT_SID" != "0" ]; then
  call POST /manager/shared-goals "$MGR_T" \
    "{\"sheet_ids\":[$DRAFT_SID],\"title\":\"Dept NPS\",\"description\":\"Team NPS target\",\"uom_type\":\"min_numeric\",\"target_value\":85,\"weightage\":15,\"thrust_area_id\":$TA_ID}"
  if [ "$HTTP_CODE" = "200" ]; then
    pass "Push shared goal"
    call GET "/goals/sheets/$DRAFT_SID" "$J_T"; D_DET=$(get_body)
    S_GID=$(echo "$D_DET" | jq '.goals[] | select(.is_shared == true) | .id // 0')
    if [ "$S_GID" -gt 0 ] 2>/dev/null; then
      pass "Shared goal visible on recipient"
      call PUT "/goals/$S_GID" "$J_T" '{"title":"Hacked shared"}'
      [ "$HTTP_CODE" = "422" ] && pass "Shared goal title edit rejected" || fail "Shared goal edit" "expected 422, got $HTTP_CODE"
    fi
  else
    fail "Shared goal" "HTTP $HTTP_CODE"
  fi
else
  skip "Shared goal" "no draft sheet in team"
fi

summary "Manager"
exit $((FAILED > 0 ? 1 : 0))
