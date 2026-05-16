#!/usr/bin/env bash
# ==============================================================
# Test: Score Computation (min, max, zero UoMs)
# ==============================================================
source "$(dirname "$0")/_common.sh"
login_all

header "Score Computation"

call POST /goals/sheets "$M_T"; SC_SH=$(get_body); SC_ID=$(echo "$SC_SH" | jq '.id // 0')
if [ -z "$SC_ID" ] || [ "$SC_ID" = "0" ]; then
  skip "Score tests" "cannot create fresh sheet"
  summary "Score"; exit 0
fi

test_score() {
  local title="$1" uom="$2" target="$3" wt="$4" actual="$5" expected="$6"
  call POST "/goals/sheets/$SC_ID/goals" "$M_T" \
    "{\"title\":\"$title\",\"uom_type\":\"$uom\",\"target_value\":$target,\"weightage\":$wt,\"thrust_area_id\":$TA_ID}"
  local gid=$(get_body | jq '.id // 0')
  call PUT "/goals/sheets/$SC_ID/submit" "$M_T" >/dev/null
  call PUT "/manager/sheets/$SC_ID/approve" "$MGR_T" >/dev/null
  call PUT "/achievements/$gid/Q1" "$M_T" \
    "{\"actual_value\":$actual,\"status\":\"completed\"}" >/dev/null
  call GET "/achievements/sheet/$SC_ID" "$M_T"
  local score=$(get_body | jq '.[] | select(.goal_id=='"$gid"') | .computed_score // 0')
  if [ "$(echo "$score == $expected" | bc -l 2>/dev/null)" = "1" ]; then
    pass "$uom: $title → $score%"
  else
    fail "$uom: $title" "expected ${expected}%, got ${score}%"
  fi
}

test_score "min_numeric 10/10" "min_numeric" 10 50 10 100
test_score "max_numeric 3/5"   "max_numeric"  5 50  3 100
test_score "zero target 0/0"   "zero"          0 50  0 100

# min_numeric partial
call POST "/goals/sheets/$SC_ID/goals" "$M_T" \
  "{\"title\":\"Partial: 5/10\",\"uom_type\":\"min_numeric\",\"target_value\":10,\"weightage\":50,\"thrust_area_id\":$TA_ID}"
G4=$(get_body | jq '.id // 0')
call PUT "/goals/sheets/$SC_ID/submit" "$M_T" >/dev/null
call PUT "/manager/sheets/$SC_ID/approve" "$MGR_T" >/dev/null
call PUT "/achievements/$G4/Q1" "$M_T" '{"actual_value":5,"status":"completed"}' >/dev/null
call GET "/achievements/sheet/$SC_ID" "$M_T"
S4=$(get_body | jq '.[] | select(.goal_id=='"$G4"') | .computed_score // 0')
[ "$(echo "$S4 == 50" | bc -l 2>/dev/null)" = "1" ] && pass "min_numeric 5/10 → 50%" || fail "min_numeric 5/10" "expected 50%, got $S4"

skip "Timeline score" "not implemented yet"

summary "Score"
exit $((FAILED > 0 ? 1 : 0))
