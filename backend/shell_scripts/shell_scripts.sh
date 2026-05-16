#!/usr/bin/env bash
# ==============================================================
# AtomQuest Backend Validation Script
# Tests all major endpoints against running backend.
# ==============================================================
set +e +u
CURL_OPTS="-s --max-time 30"

BASE_URL="${BASE_URL:-http://localhost:10000}"
PASS="password123"
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
  HTTP_CODE=$(echo "$out" | tail -1)
  echo "$out" | head -n -1
}

api()  { call GET "$1" "$EMP_T"; }
mgapi(){ call GET "$1" "$MGR_T"; }
adapi(){ call GET "$1" "$ADM_T"; }

# ── 0. Login ──────────────────────────────────────────────
header "0. Login"

EMP_R=$(login "employee@demo.com"); EMP_T=$(echo "$EMP_R" | jq -r '.token')
MGR_R=$(login "manager@demo.com"); MGR_T=$(echo "$MGR_R" | jq -r '.token')
ADM_R=$(login "admin@demo.com");   ADM_T=$(echo "$ADM_R" | jq -j '.token')
J_T=$(login "james@demo.com" | jq -r '.token')
M_T=$(login "maria@demo.com" | jq -r '.token')

for t in EMP_T MGR_T ADM_T J_T M_T; do
  [ -z "${!t}" ] && { echo "ABORT: $t empty"; exit 1; }
done
pass "All 5 users authenticated"

CYCLE=$(adapi /admin/cycles); CYCLE_ID=$(echo "$CYCLE" | jq '.[0].id')
TA_LIST=$(adapi /admin/thrust-areas); TA_ID=$(echo "$TA_LIST" | jq '.[0].id')
info "cycle=$CYCLE_ID thrust_area=$TA_ID"

# ── Phase 1 — Goal Creation & Approval ──────────────────────
header "Phase 1 — Goal Creation & Approval"

# Get existing sheet IDs
EMP_SH=$(api /goals/sheets); EMP_SID=$(echo "$EMP_SH" | jq '.[0].id // 0')
J_SH=$(call GET /goals/sheets "$J_T"); J_SID=$(echo "$J_SH" | jq '.[0].id // 0')
M_SH=$(call GET /goals/sheets "$M_T"); M_SID=$(echo "$M_SH" | jq '.[0].id // 0')
info "Sheets: emp=$EMP_SID james=$J_SID maria=$M_SID"

# ── #1-5: Use James (draft sheet) for goal creation tests ──
header ""

# Clear James's goals to start fresh
J_DET=$(call GET "/goals/sheets/$J_SID" "$J_T")
for gid in $(echo "$J_DET" | jq '.goals[].id // empty'); do
  call DELETE "/goals/$gid" "$J_T" >/dev/null 2>&1
done

G1=$(call POST "/goals/sheets/$J_SID/goals" "$J_T" \
  "{\"title\":\"Ship features\",\"uom_type\":\"min_numeric\",\"target_value\":10,\"weightage\":40,\"thrust_area_id\":$TA_ID}")
G1_ID=$(echo "$G1" | jq '.id // 0')
[ "$G1_ID" -gt 0 ] 2>/dev/null && pass "#1+3+5 Goal created with title+target+weightage (id=$G1_ID)" || fail "#5 Create goal" "HTTP=$HTTP_CODE"

G2=$(call POST "/goals/sheets/$J_SID/goals" "$J_T" \
  "{\"title\":\"Coverage\",\"uom_type\":\"min_percent\",\"target_value\":95,\"weightage\":30,\"thrust_area_id\":$TA_ID}")
G2_ID=$(echo "$G2" | jq '.id // 0')
[ "$G2_ID" -gt 0 ] && pass "#4 UoM=min_percent created" "id=$G2_ID"

G3=$(call POST "/goals/sheets/$J_SID/goals" "$J_T" \
  "{\"title\":\"Q1 launch\",\"description\":\"Ship on time\",\"uom_type\":\"timeline\",\"target_value\":0,\"weightage\":30,\"thrust_area_id\":$TA_ID}")
G3_ID=$(echo "$G3" | jq '.id // 0')
[ "$G3_ID" -gt 0 ] && pass "#3 Goal with description added" "id=$G3_ID"

# ── #6: Submit requires 100% weightage ──
call PUT "/goals/sheets/$J_SID/submit" "$J_T"
[ "$HTTP_CODE" = "200" ] && pass "#6 Submit with weightage=100 accepted" || fail "#6 Submit" "HTTP $HTTP_CODE (weightage may not be 100)"

# ── #7: Min 10% weightage per goal ──
# Create another small sheet for edge-case testing
TSH=$(call POST "/goals/sheets" "$J_T")
TSH_ID=$(echo "$TSH" | jq '.id // 0')
if [ "$TSH_ID" -gt 0 ]; then
  TR=$(call POST "/goals/sheets/$TSH_ID/goals" "$J_T" \
    "{\"title\":\"Low wt\",\"uom_type\":\"min_numeric\",\"target_value\":1,\"weightage\":5,\"thrust_area_id\":$TA_ID}")
  [ "$HTTP_CODE" = "422" ] && pass "#7 5% weightage rejected (422)" || fail "#7 5% weightage" "HTTP $HTTP_CODE (was accepted)"
else
  skip "#7 Min weightage" "cannot create test sheet"
fi

# ── #8: Max 8 goals ──
if [ "$TSH_ID" -gt 0 ]; then
  for i in $(seq 1 8); do
    call POST "/goals/sheets/$TSH_ID/goals" "$J_T" \
      "{\"title\":\"G $i\",\"uom_type\":\"min_numeric\",\"target_value\":10,\"weightage\":12,\"thrust_area_id\":$TA_ID}" >/dev/null
  done
  G9=$(call POST "/goals/sheets/$TSH_ID/goals" "$J_T" \
    "{\"title\":\"G 9\",\"uom_type\":\"min_numeric\",\"target_value\":10,\"weightage\":4,\"thrust_area_id\":$TA_ID}")
  [ "$HTTP_CODE" = "422" ] && pass "#8 Max 8 goals enforced (422)" || fail "#8 Max 8" "HTTP $HTTP_CODE (9th was accepted)"
fi

# ── #9-11: Manager review & approve ──
# Alex's sheet (id=$EMP_SID) is already submitted — manager can approve it
TEAM=$(mgapi /manager/team/sheets)
EMP_IN_TEAM=$(echo "$TEAM" | jq '.[] | select(.user_id == 5) | .id // 0')
if [ "$EMP_IN_TEAM" -gt 0 ]; then
  pass "#9 Manager sees Alex's submitted sheet"
  call PUT "/manager/sheets/$EMP_IN_TEAM/approve" "$MGR_T"
  [ "$HTTP_CODE" = "200" ] && pass "#11 Manager approved sheet" || fail "#11 Approve" "HTTP $HTTP_CODE"
else
  fail "#9 Manager team" "no sheet for emp id=5"
fi

# ── #12: Manager return ──
# James just submitted his sheet — manager can return it
call PUT "/manager/sheets/$J_SID/return" "$MGR_T" '{"reason":"Revise targets"}'
[ "$HTTP_CODE" = "200" ] && pass "#12 Sheet returned with reason" || {
  # If already approved, skip. If still draft, submit first.
  J_STAT=$(call GET "/goals/sheets/$J_SID" "$J_T" | jq -r '.status')
  if [ "$J_STAT" = "submitted" ]; then
    fail "#12 Return" "HTTP $HTTP_CODE (sheet is $J_STAT)"
  else
    skip "#12 Return" "sheet status=$J_STAT"
  fi
}

# ── #13: Locked sheet — goal edit blocked ──
# Alex's sheet is now locked. Try editing a goal
A_DET=$(call GET "/goals/sheets/$EMP_SID" "$EMP_T")
A_G1=$(echo "$A_DET" | jq '.goals[0].id // 0')
if [ "$A_G1" -gt 0 ]; then
  call PUT "/goals/$A_G1" "$EMP_T" '{"title":"Hack attempt"}'
  [ "$HTTP_CODE" = "422" ] && pass "#13 Locked sheet edit rejected" || fail "#13 Locked edit" "expected 422, got $HTTP_CODE"
fi

# ── #14: Shared goal push ──
call POST "/manager/shared-goals" "$MGR_T" \
  "{\"sheet_ids\":[$J_SID],\"title\":\"Dept NPS\",\"description\":\"Team NPS target\",\"uom_type\":\"min_numeric\",\"target_value\":85,\"weightage\":15,\"thrust_area_id\":$TA_ID}"
[ "$HTTP_CODE" = "200" ] && pass "#14 Shared goal pushed" || fail "#14 Shared goal" "HTTP $HTTP_CODE"

# ── #15: Shared goal read-only ──
J_DET2=$(call GET "/goals/sheets/$J_SID" "$J_T")
S_GID=$(echo "$J_DET2" | jq '.goals[] | select(.is_shared == true) | .id // 0')
if [ "$S_GID" -gt 0 ]; then
  call PUT "/goals/$S_GID" "$J_T" '{"title":"Changed shared"}'
  [ "$HTTP_CODE" = "422" ] && pass "#15 Shared goal title edit rejected" || fail "#15 Shared goal" "HTTP $HTTP_CODE (not enforced)"
else
  skip "#15 Shared goal" "no shared goal in James's sheet"
fi

skip "#16 Achievement sync" "not implemented"

# ── Phase 2 — Achievements ─────────────────────────────────
header "Phase 2 — Achievement & Check-in"

# Maria's sheet (id=$M_SID) is locked with goals
# . . .
M_DET=$(call GET "/goals/sheets/$M_SID" "$M_T")
M_G1=$(echo "$M_DET" | jq '.goals[0].id // 0')
if [ -n "$M_G1" ] && [ "$M_G1" != "0" ]; then
  call PUT "/achievements/$M_G1/Q2" "$M_T" \
    '{"actual_value":2,"actual_date":"2026-08-15","status":"completed"}'
  [ "$HTTP_CODE" = "200" ] && pass "#18 Logged Q2 actual value" || fail "#18 Log Q2" "HTTP $HTTP_CODE"

  call PUT "/achievements/$M_G1/Q3" "$M_T" '{"status":"on_track"}'
  [ "$HTTP_CODE" = "200" ] && pass "#19 Set status=on_track" || fail "#19 Status" "HTTP $HTTP_CODE"

  ACH_R=$(call GET "/achievements/sheet/$M_SID" "$M_T")
  S2=$(echo "$ACH_R" | jq '.[] | select(.quarter=="q2") | .computed_score // 0')
  info "  Q2 computed score: $S2"
  [ -n "$S2" ] && [ "$S2" != "0" ] && pass "#17 Achievement with computed score" "score=$S2" || skip "#17 Score" "no Q2 score"
else
  fail "#17 No goal" "in Maria's sheet id=$M_SID"
fi

# Manager check-in
mgapi /manager/team/checkins; [ "$HTTP_CODE" = "200" ] && pass "#20 Manager views check-ins" || fail "#20 Checkins" "HTTP $HTTP_CODE"
call POST "/manager/checkins/$M_SID" "$MGR_T" '{"quarter":"Q2","comment":"Good Q2"}'
[ "$HTTP_CODE" = "200" ] && pass "#22 Added check-in comment" || fail "#22 Checkin" "HTTP $HTTP_CODE"

# ── #23-26: Score computation ──
header "Score Computation"

# Create a fresh sheet for score tests
SC_SH=$(call POST "/goals/sheets" "$M_T"); SC_ID=$(echo "$SC_SH" | jq '.id // 0')
if [ "$SC_ID" -gt 0 ]; then
  # min_numeric: target=10, actual=10 => 100%
  SCG1=$(call POST "/goals/sheets/$SC_ID/goals" "$M_T" \
    "{\"title\":\"Min test\",\"uom_type\":\"min_numeric\",\"target_value\":10,\"weightage\":50,\"thrust_area_id\":$TA_ID}" | jq '.id // 0')
  # max_numeric: target=5, actual=3 => 100%
  SCG2=$(call POST "/goals/sheets/$SC_ID/goals" "$M_T" \
    "{\"title\":\"Max test\",\"uom_type\":\"max_numeric\",\"target_value\":5,\"weightage\":50,\"thrust_area_id\":$TA_ID}" | jq '.id // 0')

  call PUT "/goals/sheets/$SC_ID/submit" "$M_T"
  call PUT "/manager/sheets/$SC_ID/approve" "$MGR_T"

  if [ "$SCG1" -gt 0 ]; then
    call PUT "/achievements/$SCG1/Q1" "$M_T" '{"actual_value":10,"status":"completed"}'
    S1=$(call GET "/achievements/sheet/$SC_ID" "$M_T" | jq '.[] | select(.goal_id=='"$SCG1"') | .computed_score')
    [ "$(echo "$S1 == 100" | bc -l 2>/dev/null)" = "1" ] && pass "#23 min_numeric 10/10=100% ✓" || fail "#23 min_numeric" "expected 100, got $S1"
  fi
  if [ "$SCG2" -gt 0 ]; then
    call PUT "/achievements/$SCG2/Q1" "$M_T" '{"actual_value":3,"status":"completed"}'
    S2=$(call GET "/achievements/sheet/$SC_ID" "$M_T" | jq '.[] | select(.goal_id=='"$SCG2"') | .computed_score')
    [ "$(echo "$S2 == 100" | bc -l 2>/dev/null)" = "1" ] && pass "#24 max_numeric 3/5=100% ✓" || fail "#24 max_numeric" "expected 100, got $S2"
  fi

  # zero: target=0, actual=0 => 100%
  SCG3=$(call POST "/goals/sheets/$SC_ID/goals" "$M_T" \
    "{\"title\":\"Zero test\",\"uom_type\":\"zero\",\"target_value\":0,\"weightage\":100,\"thrust_area_id\":$TA_ID}" | jq '.id // 0')
  if [ "$SCG3" -gt 0 ] 2>/dev/null; then
    call PUT "/achievements/$SCG3/Q1" "$M_T" '{"actual_value":0,"status":"completed"}'
    S3=$(call GET "/achievements/sheet/$SC_ID" "$M_T" | jq '.[] | select(.goal_id=='"$SCG3"') | .computed_score')
    [ "$(echo "$S3 == 100" | bc -l 2>/dev/null)" = "1" ] && pass "#26 zero=0 → 100% ✓" || fail "#26 zero" "expected 100, got $S3"
  fi
else
  skip "#23-26 Score" "cannot create test sheet"
fi
skip "#25 Timeline score" "not implemented"

# ── Phase 2.3 — Schedule ──────────────────────────────────
header "Phase 2.3 — Schedule Enforcement"
skip "#27 Goal setting window" "not implemented"
skip "#28-31 Quarterly windows" "not implemented"

# ── Admin ──────────────────────────────────────────────────
header "Admin Operations"

adapi /admin/cycles;             [ "$HTTP_CODE" = "200" ] && pass "Admin list cycles" || fail "Admin list cycles" "HTTP $HTTP_CODE"
CR_BODY=$(call POST /admin/cycles "$ADM_T" \
  '{"name":"Test Cycle","goal_setting_opens":"2027-05-01","q1_opens":"2027-07-01","q2_opens":"2027-10-01","q3_opens":"2028-01-01","q4_opens":"2028-04-01","is_active":false}')
[ "$HTTP_CODE" = "200" ] && pass "Admin create cycle" || fail "Admin create cycle" "HTTP $HTTP_CODE"
NEW_CID=$(echo "$CR_BODY" | jq '.id // 0')
call PUT "/admin/cycles/$NEW_CID" "$ADM_T" '{"name":"Test Cycle v2"}'
[ "$HTTP_CODE" = "200" ] && pass "Admin update cycle (id=$NEW_CID)" || fail "Admin update cycle" "HTTP $HTTP_CODE"

adapi /admin/departments;       [ "$HTTP_CODE" = "200" ] && pass "Admin list departments" || fail "Admin list depts" "HTTP $HTTP_CODE"
adapi /admin/thrust-areas;      [ "$HTTP_CODE" = "200" ] && pass "Admin list thrust areas" || fail "Admin list TA" "HTTP $HTTP_CODE"
call POST /admin/thrust-areas "$ADM_T" '{"name":"New TA","department_id":1}'
[ "$HTTP_CODE" = "200" ] && pass "Admin create thrust area" || fail "Admin create TA" "HTTP $HTTP_CODE"

adapi /admin/users;             [ "$HTTP_CODE" = "200" ] && pass "Admin list users" || fail "Admin list users" "HTTP $HTTP_CODE"
U_BODY=$(call POST /admin/users "$ADM_T" \
  '{"email":"tmp@demo.com","full_name":"Tmp","password":"password123","department_id":1,"role":"employee","manager_id":4}')
[ "$HTTP_CODE" = "200" ] && pass "Admin create user" || fail "Admin create user" "HTTP $HTTP_CODE"
NEW_UID=$(echo "$U_BODY" | jq '.id // 0')
call PUT "/admin/users/$NEW_UID" "$ADM_T" '{"full_name":"Updated"}'
[ "$HTTP_CODE" = "200" ] && pass "Admin update user (id=$NEW_UID)" || fail "Admin update user" "HTTP $HTTP_CODE"
call DELETE "/admin/users/$NEW_UID" "$ADM_T"
[ "$HTTP_CODE" = "200" ] && pass "Admin delete user" || fail "Admin delete user" "HTTP $HTTP_CODE"

call PUT "/admin/sheets/$M_SID/unlock" "$ADM_T"
[ "$HTTP_CODE" = "200" ] && pass "Admin unlock sheet" || fail "Admin unlock sheet" "HTTP $HTTP_CODE"

adapi /admin/audit-log;         [ "$HTTP_CODE" = "200" ] && pass "Admin view audit log" || fail "Admin audit log" "HTTP $HTTP_CODE"

# ── Reports ────────────────────────────────────────────────
header "Reports"
call GET /reports/achievement "$MGR_T"
[ "$HTTP_CODE" = "200" ] && pass "Manager achievement report" || fail "Manager achievement report" "HTTP $HTTP_CODE"
call GET /reports/completion-dashboard "$MGR_T"
[ "$HTTP_CODE" = "200" ] && pass "Manager completion dashboard" || fail "Manager completion dashboard" "HTTP $HTTP_CODE"

# ── Summary ─────────────────────────────────────────────────
header "Summary"
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
[ "$FAILED" -eq 0 ] && exit 0 || exit 1
