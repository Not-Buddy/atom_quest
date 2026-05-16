#!/usr/bin/env bash
# ==============================================================
# Test: Admin (Cycles, Departments, Thrust Areas, Users, Unlock, Audit)
# ==============================================================
source "$(dirname "$0")/_common.sh"
login_all

header "Admin — Cycles"
call GET /admin/cycles "$ADM_T"; [ "$HTTP_CODE" = "200" ] && pass "List cycles" || fail "List cycles" "HTTP $HTTP_CODE"

call POST /admin/cycles "$ADM_T" \
  '{"name":"ShTst Cycle","goal_setting_opens":"2027-05-01","q1_opens":"2027-07-01","q2_opens":"2027-10-01","q3_opens":"2028-01-01","q4_opens":"2028-04-01","is_active":false}'
[ "$HTTP_CODE" = "200" ] && pass "Create cycle" || fail "Create cycle" "HTTP $HTTP_CODE"
NEW_CID=$(get_body | jq '.id // 0')

call PUT "/admin/cycles/$NEW_CID" "$ADM_T" '{"name":"ShTst Cycle v2"}'
[ "$HTTP_CODE" = "200" ] && pass "Update cycle" || fail "Update cycle" "HTTP $HTTP_CODE"

header "Admin — Departments"
call GET /admin/departments "$ADM_T";  [ "$HTTP_CODE" = "200" ] && pass "List departments" || fail "List depts" "HTTP $HTTP_CODE"

header "Admin — Thrust Areas"
call GET /admin/thrust-areas "$ADM_T"; [ "$HTTP_CODE" = "200" ] && pass "List thrust areas" || fail "List TA" "HTTP $HTTP_CODE"
call POST /admin/thrust-areas "$ADM_T" '{"name":"Sh TA","department_id":1}'
[ "$HTTP_CODE" = "200" ] && pass "Create thrust area" || fail "Create TA" "HTTP $HTTP_CODE"

header "Admin — Users"
call GET /admin/users "$ADM_T";        [ "$HTTP_CODE" = "200" ] && pass "List users" || fail "List users" "HTTP $HTTP_CODE"

call POST /admin/users "$ADM_T" \
  '{"email":"shuid@demo.com","full_name":"ShUid","password":"password123","department_id":1,"role":"employee","manager_id":4}'
[ "$HTTP_CODE" = "200" ] && pass "Create user" || fail "Create user" "HTTP $HTTP_CODE"
NEW_UID=$(get_body | jq '.id // 0')

call PUT "/admin/users/$NEW_UID" "$ADM_T" '{"full_name":"ShUid Updated"}'
[ "$HTTP_CODE" = "200" ] && pass "Update user" || fail "Update user" "HTTP $HTTP_CODE"

call DELETE "/admin/users/$NEW_UID" "$ADM_T"
[ "$HTTP_CODE" = "200" ] && pass "Delete user" || fail "Delete user" "HTTP $HTTP_CODE"

header "Admin — Sheet Unlock"
# Maria's sheet should be locked after mock_data_script reset
call PUT "/admin/sheets/$M_SID/unlock" "$ADM_T"
[ "$HTTP_CODE" = "200" ] && pass "Unlock sheet" || fail "Unlock sheet" "HTTP $HTTP_CODE"

header "Admin — Audit Log"
call GET /admin/audit-log "$ADM_T"
[ "$HTTP_CODE" = "200" ] && pass "View audit log" || fail "Audit log" "HTTP $HTTP_CODE"
AUDIT_COUNT=$(get_body | jq 'length')
info "  Audit entries: $AUDIT_COUNT"
[ "$AUDIT_COUNT" -gt 0 ] && pass "Audit log has entries" || skip "Audit entries" "empty"

summary "Admin"
exit $((FAILED > 0 ? 1 : 0))
