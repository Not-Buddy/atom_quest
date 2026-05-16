#!/usr/bin/env bash
# ==============================================================
# Test: Goal Sheets & Goals (Employee: CRUD, Submit, Validation)
# ==============================================================
source "$(dirname "$0")/_common.sh"
login_all

header "Goal Sheets & Goals"

# Get James's sheet status
call GET "/goals/sheets/$J_SID" "$J_T"
J_DET=$(get_body)
J_STATUS=$(echo "$J_DET" | jq -r '.status // "draft"')

# If sheet is not draft, create fresh one
if [ "$J_STATUS" != "draft" ]; then
  info "James's sheet is $J_STATUS — creating fresh sheet"
  call POST /goals/sheets "$J_T"
  TSH_BODY=$(get_body)
  TSH_ID=$(echo "$TSH_BODY" | jq '.id // 0')
  if [ "$TSH_ID" -gt 0 ] 2>/dev/null; then
    J_SID=$TSH_ID
    pass "Created fresh draft sheet (id=$J_SID)"
  else
    fail "Create fresh sheet" "HTTP $HTTP_CODE"
    summary "Goals"; exit 1
  fi
else
  # Clear existing goals
  for gid in $(echo "$J_DET" | jq '.goals[].id // empty'); do
    call DELETE "/goals/$gid" "$J_T" >/dev/null 2>&1
  done
fi

# Add goals
call POST "/goals/sheets/$J_SID/goals" "$J_T" \
  "{\"title\":\"Ship features\",\"uom_type\":\"min_numeric\",\"target_value\":10,\"weightage\":40,\"thrust_area_id\":$TA_ID}"
G1=$(get_body); G1_ID=$(echo "$G1" | jq '.id // 0')
[ "$G1_ID" -gt 0 ] 2>/dev/null && pass "Add goal: min_numeric (id=$G1_ID)" || fail "Add goal min_numeric" "HTTP=$HTTP_CODE"

call POST "/goals/sheets/$J_SID/goals" "$J_T" \
  "{\"title\":\"Coverage\",\"uom_type\":\"min_percent\",\"target_value\":95,\"weightage\":30,\"thrust_area_id\":$TA_ID}"
G2=$(get_body); G2_ID=$(echo "$G2" | jq '.id // 0')
[ "$G2_ID" -gt 0 ] && pass "Add goal: min_percent (id=$G2_ID)" || fail "Add goal min_percent" "HTTP=$HTTP_CODE"

call POST "/goals/sheets/$J_SID/goals" "$J_T" \
  "{\"title\":\"Q1 launch\",\"description\":\"Ship on time\",\"uom_type\":\"timeline\",\"target_value\":0,\"weightage\":30,\"thrust_area_id\":$TA_ID}"
G3=$(get_body); G3_ID=$(echo "$G3" | jq '.id // 0')
[ "$G3_ID" -gt 0 ] && pass "Add goal: timeline + description (id=$G3_ID)" || fail "Add goal timeline" "HTTP=$HTTP_CODE"

# Submit
call PUT "/goals/sheets/$J_SID/submit" "$J_T"
[ "$HTTP_CODE" = "200" ] && pass "Submit sheet (weightage=100)" || fail "Submit sheet" "HTTP $HTTP_CODE"

# Min weightage test
call POST /goals/sheets "$J_T"; TSH2=$(get_body); TSH2_ID=$(echo "$TSH2" | jq '.id // 0')
if [ "$TSH2_ID" -gt 0 ] 2>/dev/null; then
  call POST "/goals/sheets/$TSH2_ID/goals" "$J_T" \
    "{\"title\":\"Too low\",\"uom_type\":\"min_numeric\",\"target_value\":1,\"weightage\":5,\"thrust_area_id\":$TA_ID}"
  [ "$HTTP_CODE" = "422" ] && pass "Weightage 5% rejected" || fail "Min weightage" "expected 422, got $HTTP_CODE"
else
  skip "Min weightage" "cannot create test sheet"
fi

# Max 8 goals
call POST /goals/sheets "$J_T"; TSH3=$(get_body); TSH3_ID=$(echo "$TSH3" | jq '.id // 0')
if [ "$TSH3_ID" -gt 0 ] 2>/dev/null; then
  for i in $(seq 1 8); do
    call POST "/goals/sheets/$TSH3_ID/goals" "$J_T" \
      "{\"title\":\"G $i\",\"uom_type\":\"min_numeric\",\"target_value\":10,\"weightage\":12,\"thrust_area_id\":$TA_ID}" >/dev/null
  done
  call POST "/goals/sheets/$TSH3_ID/goals" "$J_T" \
    "{\"title\":\"G 9\",\"uom_type\":\"min_numeric\",\"target_value\":10,\"weightage\":4,\"thrust_area_id\":$TA_ID}"
  [ "$HTTP_CODE" = "422" ] && pass "Max 8 goals enforced" || fail "Max 8 goals" "expected 422, got $HTTP_CODE"
else
  skip "Max 8 goals" "cannot create test sheet"
fi

# Edit on submitted sheet should fail
call PUT "/goals/$G1_ID" "$J_T" '{"title":"Hack"}'
[ "$HTTP_CODE" = "422" ] && pass "Edit on submitted sheet → rejected" || fail "Edit on submitted" "expected 422, got $HTTP_CODE"

summary "Goals"
exit $((FAILED > 0 ? 1 : 0))
