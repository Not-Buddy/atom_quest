#!/usr/bin/env bash
# ==============================================================
# Test: Score Computation (min, max, zero UoMs)
# ==============================================================
source "$(dirname "$0")/_common.sh"
login_all

header "Score Computation"

# 1. Create a fresh test user so we have a clean slate (no existing sheets)
# Using $RANDOM so it's fresh every time we run the test
# Manager ID 4 is Sarah Chen (MGR_T)
RND=$RANDOM
TEST_EMAIL="score_tester_$RND@demo.com"

call POST /admin/users "$ADM_T" "{\"email\":\"$TEST_EMAIL\",\"password\":\"password123\",\"full_name\":\"Score Tester $RND\",\"role\":\"employee\",\"manager_id\":4}" >/dev/null
ST_R=$(login "$TEST_EMAIL")
ST_T=$(echo "$ST_R" | jq -r '.token')

if [ -z "$ST_T" ] || [ "$ST_T" = "null" ]; then
  skip "Score tests" "cannot create/login fresh score tester"
  summary "Score"; exit 0
fi

# 2. Create a fresh draft sheet
call POST /goals/sheets "$ST_T"; SC_SH=$(get_body); SC_ID=$(echo "$SC_SH" | jq '.id // 0')
if [ -z "$SC_ID" ] || [ "$SC_ID" = "0" ]; then
  skip "Score tests" "cannot create fresh sheet"
  summary "Score"; exit 0
fi

# 3. Add all goals FIRST (weightage = 25 each to sum to 100)
add_goal() {
  local title="$1" uom="$2" target="$3" wt="$4"
  call POST "/goals/sheets/$SC_ID/goals" "$ST_T" \
    "{\"title\":\"$title\",\"uom_type\":\"$uom\",\"target_value\":$target,\"weightage\":$wt,\"thrust_area_id\":$TA_ID}"
  get_body | jq '.id // 0'
}

G1=$(add_goal "min_numeric 10/10" "min_numeric" 10 25)
G2=$(add_goal "max_numeric 3/5"   "max_numeric"  5 25)
G3=$(add_goal "zero target 0/0"   "zero"          0 25)
G4=$(add_goal "Partial: 5/10"     "min_numeric" 10 25)

# 4. Submit & Approve sheet ONCE
call PUT "/goals/sheets/$SC_ID/submit" "$ST_T" >/dev/null
call PUT "/manager/sheets/$SC_ID/approve" "$MGR_T" >/dev/null

# 5. Log achievements
call PUT "/achievements/$G1/Q1" "$ST_T" '{"actual_value":10,"status":"completed"}' >/dev/null
call PUT "/achievements/$G2/Q1" "$ST_T" '{"actual_value":3,"status":"completed"}' >/dev/null
call PUT "/achievements/$G3/Q1" "$ST_T" '{"actual_value":0,"status":"completed"}' >/dev/null
call PUT "/achievements/$G4/Q1" "$ST_T" '{"actual_value":5,"status":"completed"}' >/dev/null

# 6. Fetch achievements and verify
call GET "/achievements/sheet/$SC_ID" "$ST_T"
ACH_DATA=$(get_body)

verify_score() {
  local gid="$1" uom="$2" title="$3" expected="$4"
  local score=$(echo "$ACH_DATA" | jq '.[] | select(.goal_id=='"$gid"') | .computed_score // 0')
  if [ -z "$score" ]; then
    fail "$uom: $title" "expected ${expected}%, got <empty>"
  elif [ "$(echo "$score == $expected" | bc -l 2>/dev/null)" = "1" ]; then
    pass "$uom: $title → $score%"
  else
    fail "$uom: $title" "expected ${expected}%, got ${score}%"
  fi
}

verify_score "$G1" "min_numeric" "min_numeric 10/10" 100
verify_score "$G2" "max_numeric" "max_numeric 3/5"   100
verify_score "$G3" "zero"        "zero target 0/0"   100
verify_score "$G4" "min_numeric" "min_numeric 5/10"  50

skip "Timeline score" "not implemented yet"

summary "Score"
exit $((FAILED > 0 ? 1 : 0))
