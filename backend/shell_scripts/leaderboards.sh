#API_URL="https://alpha.innosolve.in"
API_URL="http://localhost:3000"

# ====================================================
# PART 5: PUBLIC LEADERBOARD API TEST
# ====================================================

echo "======================================================"
echo "    PART 5: PUBLIC LEADERBOARD API TEST"
echo "======================================================"
echo ""


echo "9. Testing leaderboard endpoint (Public - No Auth Required)"
echo ""


# Test all academic years
for YEAR in "I" "II" "III"; do
    echo "----------------------------------------"
    echo "Testing Academic Year: ${YEAR}"
    echo "----------------------------------------"
    
    LEADERBOARD_RESPONSE=$(curl -s -w "\\n%{http_code}" -X GET "${API_URL}/leaderboard?academic_year=${YEAR}")
    
    # Extract body and status
    LEADERBOARD_BODY=$(echo "$LEADERBOARD_RESPONSE" | head -n -1)
    LEADERBOARD_STATUS=$(echo "$LEADERBOARD_RESPONSE" | tail -n 1)
    
    if [ "$LEADERBOARD_STATUS" -ge 200 ] && [ "$LEADERBOARD_STATUS" -lt 300 ]; then
        echo "✅ Leaderboard fetched successfully. HTTP Status: ${LEADERBOARD_STATUS}"
        echo ""
        
        # Parse response
        TOTAL_STUDENTS=$(echo "${LEADERBOARD_BODY}" | jq -r '.total_students // 0')
        ACADEMIC_YEAR=$(echo "${LEADERBOARD_BODY}" | jq -r '.academic_year // "N/A"')
        
        echo "📊 Leaderboard Summary:"
        echo "  Academic Year: ${ACADEMIC_YEAR}"
        echo "  Total Students: ${TOTAL_STUDENTS}"
        echo ""
        
        if [ "$TOTAL_STUDENTS" -gt 0 ]; then
            echo "🏆 Top 5 Students:"
            echo "----------------------------------------"
            
            # Display top 5 students
            for i in 0 1 2 3 4; do
                RANK=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].rank // null")
                
                if [ "$RANK" != "null" ]; then
                    # UPDATED FIELDS HERE
                    RA_NUM=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].registration_number")
                    NAME=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].full_name")
                    SPEC=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].specialization // \"N/A\"")
                    
                    # New metric field name
                    QUESTIONS=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].total_solved_last_30_days")
                    
                    # API now returns username, so we format it for display
                    LEETCODE_USER=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].leetcode_username // \"null\"")
                    if [ "$LEETCODE_USER" != "null" ] && [ "$LEETCODE_USER" != "" ]; then
                        LEETCODE_DISPLAY="https://leetcode.com/u/$LEETCODE_USER"
                    else
                        LEETCODE_DISPLAY="Not linked"
                    fi
                    
                    # Truncate specialization if too long
                    if [ ${#SPEC} -gt 40 ]; then
                        SPEC="${SPEC:0:37}..."
                    fi
                    
                    echo "Rank ${RANK}: ${NAME}"
                    echo "  RA Number: ${RA_NUM}"
                    echo "  Specialization: ${SPEC}"
                    echo "  Questions (30 days): ${QUESTIONS}"
                    echo "  LeetCode: ${LEETCODE_DISPLAY}"
                    echo ""
                fi
            done
            
            echo "Raw JSON Response (formatted):"
            echo "${LEADERBOARD_BODY}" | jq '.'
        else
            echo "ℹ️  No students found for this academic year"
        fi
    else
        echo "❌ Failed to fetch leaderboard. HTTP Status: ${LEADERBOARD_STATUS}"
        echo "Response: ${LEADERBOARD_BODY}"
    fi
    
    echo ""
done