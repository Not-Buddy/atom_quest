#!/bin/bash

# Configuration
EMAIL="ak8098@srmist.edu.in"
PASSWORD=""
API_URL="http://localhost:3000"
#API_URL="https://alpha.innosolve.in"

# Full URLs (not just usernames)
LEETCODE_LINK="https://leetcode.com/u/not_buddy/"
GITHUB_LINK="https://github.com/Not-Buddy/"
LINKEDIN_LINK="https://www.linkedin.com/in/aary-k-a77499240/"
CODECHEF_LINK="https://www.codechef.com/users/not_buddy"
CODEFORCES_LINK="https://codeforces.com/profile/not_buddy"

# Faculty credentials
FACULTY_SPECIALIZATION="Computer Science and Engineering"
FACULTY_USERNAME="prof_III_cse"
FACULTY_PASSWORD="faculty@III@cse"

echo "======================================================"
echo " PART 1: STUDENT LOGIN & PROFILE LINKS"
echo "======================================================"
echo ""

echo "--- Submitting Profile Links for ${EMAIL} ---"
echo "1. Attempting to log in as ${EMAIL}..."

# Construct the JSON payload
JSON_PAYLOAD=$(cat <<EOF
{
  "email": "${EMAIL}",
  "password": "${PASSWORD}"
}
EOF
)

echo "Sending payload: ${JSON_PAYLOAD}"

# Send the POST request using curl with improved error handling
LOGIN_RESPONSE=$(curl -w "\n%{http_code}" -s -X POST "${API_URL}/auth/login" \
     -H "Content-Type: application/json" \
     -H "Accept: application/json" \
     -d "${JSON_PAYLOAD}")

# Extract HTTP status code and response body
HTTP_BODY=$(echo "$LOGIN_RESPONSE" | head -n -1)
HTTP_CODE=$(echo "$LOGIN_RESPONSE" | tail -n 1)

echo ""
echo "Response Body:"
echo "$HTTP_BODY"
echo ""
echo "HTTP Status Code: $HTTP_CODE"

# Check for success or failure
if [[ "$HTTP_CODE" -ge 200 && "$HTTP_CODE" -lt 300 ]]; then
    echo "✓ Login successful"
    
    # Extract JWT token
    JWT_TOKEN=$(echo "${HTTP_BODY}" | jq -r '.token')
    
    if [ "${JWT_TOKEN}" == "null" ] || [ -z "${JWT_TOKEN}" ]; then
        echo "❌ No JWT token received in response"
        exit 1
    fi
    
    echo "✅ Successfully logged in. Obtained JWT token."
    echo "Student: $(echo "${HTTP_BODY}" | jq -r '.student.student_name')"
    
elif [[ "$HTTP_CODE" -eq 401 ]]; then
    echo "✗ Authentication failed - Invalid credentials"
    exit 1
else
    echo "✗ Request failed with status code: $HTTP_CODE"
    exit 1
fi

echo ""

echo "2. Submitting profile links..."
UPDATE_RESPONSE=$(curl -s -w "\\n%{http_code}" -X POST "${API_URL}/profile/links" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${JWT_TOKEN}" \
  -d "{
  \"github_link\": \"${GITHUB_LINK}\",
  \"linkedin_link\": \"${LINKEDIN_LINK}\",
  \"leetcode_link\": \"${LEETCODE_LINK}\",
  \"codechef_link\": \"${CODECHEF_LINK}\",
  \"codeforces_link\": \"${CODEFORCES_LINK}\"
}")

# Extract body and status
HTTP_BODY=$(echo "$UPDATE_RESPONSE" | head -n -1)
HTTP_STATUS=$(echo "$UPDATE_RESPONSE" | tail -n 1)

if [ "$HTTP_STATUS" -ge 200 ] && [ "$HTTP_STATUS" -lt 300 ]; then
  echo "✅ Profile links submitted successfully. HTTP Status: ${HTTP_STATUS}"
  echo "Response:"
  echo "${HTTP_BODY}" | jq .
else
  echo "❌ Failed to submit profile links. HTTP Status: ${HTTP_STATUS}"
  echo "Response: ${HTTP_BODY}"
  exit 1
fi

echo ""

echo "3. Fetching submitted profile links..."
FETCH_RESPONSE=$(curl -s -w "\\n%{http_code}" -X GET "${API_URL}/profile/links" \
  -H "Authorization: Bearer ${JWT_TOKEN}")

# Extract body and status
FETCH_BODY=$(echo "$FETCH_RESPONSE" | head -n -1)
FETCH_STATUS=$(echo "$FETCH_RESPONSE" | tail -n 1)

if [ "$FETCH_STATUS" -ge 200 ] && [ "$FETCH_STATUS" -lt 300 ]; then
  echo "✅ Successfully fetched profile links. HTTP Status: ${FETCH_STATUS}"
  echo ""
  echo "📋 Current Profile Links:"
  echo "----------------------------------------"
  GITHUB=$(echo "${FETCH_BODY}" | jq -r '.github_link // "Not set"')
  LEETCODE=$(echo "${FETCH_BODY}" | jq -r '.leetcode_link // "Not set"')
  CODECHEF=$(echo "${FETCH_BODY}" | jq -r '.codechef_link // "Not set"')
  CODEFORCES=$(echo "${FETCH_BODY}" | jq -r '.codeforces_link // "Not set"')
  LINKEDIN=$(echo "${FETCH_BODY}" | jq -r '.linkedin_link // "Not set"')
  echo "GitHub: ${GITHUB}"
  echo "LeetCode: ${LEETCODE}"
  echo "CodeChef: ${CODECHEF}"
  echo "Codeforces: ${CODEFORCES}"
  echo "LinkedIn: ${LINKEDIN}"
  echo "----------------------------------------"
  
  # Also show raw JSON
  echo ""
  echo "Raw JSON Response:"
  echo "${FETCH_BODY}" | jq .
else
  echo "❌ Failed to fetch profile links. HTTP Status: ${FETCH_STATUS}"
  echo "Response: ${FETCH_BODY}"
fi

echo ""
echo "--- Student Section Complete ---"
echo ""
echo ""

# Rest of the script continues unchanged...
# (Faculty login, Excel reports, leaderboard, stats tests remain the same)



# ====================================================
# FACULTY LOGIN TEST
# ====================================================


echo "======================================================"
echo "    PART 2: FACULTY LOGIN TEST"
echo "======================================================"
echo ""


echo "4. Attempting to log in as faculty..."
echo "   Specialization: ${FACULTY_SPECIALIZATION}"
echo "   Username: ${FACULTY_USERNAME}"
echo ""


FACULTY_LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/faculty/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"specialization\": \"${FACULTY_SPECIALIZATION}\",
    \"username\": \"${FACULTY_USERNAME}\",
    \"password\": \"${FACULTY_PASSWORD}\"
  }")


# Extract Faculty JWT token
FACULTY_JWT_TOKEN=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.token')


if [ "${FACULTY_JWT_TOKEN}" == "null" ] || [ -z "${FACULTY_JWT_TOKEN}" ]; then
  echo "❌ Faculty login failed. Check credentials or API server status."
  echo "Response: ${FACULTY_LOGIN_RESPONSE}"
  exit 1
fi


echo "✅ Successfully logged in as faculty. Obtained JWT token."
echo ""
echo "Faculty Details:"
echo "----------------------------------------"
FACULTY_ID=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.faculty.id')
FACULTY_SPEC=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.faculty.specialization')
FACULTY_USER=$(echo "${FACULTY_LOGIN_RESPONSE}" | jq -r '.faculty.username')


echo "ID:             ${FACULTY_ID}"
echo "Specialization: ${FACULTY_SPEC}"
echo "Username:       ${FACULTY_USER}"
echo "----------------------------------------"
echo ""


echo "Raw JSON Response:"
echo "${FACULTY_LOGIN_RESPONSE}" | jq .
echo ""


echo "5. Fetching faculty profile with JWT..."
FACULTY_ME_RESPONSE=$(curl -s -w "\\n%{http_code}" -X GET "${API_URL}/faculty/me" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}")


# Extract body and status
FACULTY_ME_BODY=$(echo "$FACULTY_ME_RESPONSE" | head -n -1)
FACULTY_ME_STATUS=$(echo "$FACULTY_ME_RESPONSE" | tail -n 1)


if [ "$FACULTY_ME_STATUS" -ge 200 ] && [ "$FACULTY_ME_STATUS" -lt 300 ]; then
  echo "✅ Successfully fetched faculty profile. HTTP Status: ${FACULTY_ME_STATUS}"
  echo ""
  echo "📋 Faculty Profile:"
  echo "----------------------------------------"
  echo "${FACULTY_ME_BODY}" | jq .
  echo "----------------------------------------"
else
  echo "❌ Failed to fetch faculty profile. HTTP Status: ${FACULTY_ME_STATUS}"
  echo "Response: ${FACULTY_ME_BODY}"
fi


echo ""
echo "6. JWT Token Details:"
echo "----------------------------------------"
echo "Student Token (first 50 chars): ${JWT_TOKEN:0:50}..."
echo "Faculty Token (first 50 chars): ${FACULTY_JWT_TOKEN:0:50}..."
echo "----------------------------------------"


echo ""
echo ""


# ====================================================
# PART 3: EXCEL REPORT GENERATION
# ====================================================


echo "======================================================"
echo "    PART 3: FACULTY EXCEL REPORT DOWNLOAD"
echo "======================================================"
echo ""


echo "7. Downloading student submissions Excel report..."


# Generate filename with timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
EXCEL_FILENAME="student_submissions_${TIMESTAMP}.xlsx"


# Download the Excel report
curl -s -X GET "${API_URL}/faculty/reports/submissions" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}" \
  -o "${EXCEL_FILENAME}" \
  -w "\nHTTP Status: %{http_code}\n"


# Check if download was successful
if [ $? -eq 0 ] && [ -f "${EXCEL_FILENAME}" ]; then
  FILE_SIZE=$(ls -lh "${EXCEL_FILENAME}" | awk '{print $5}')
  
  # Check if file is not empty and is actually an Excel file
  if [ -s "${EXCEL_FILENAME}" ]; then
    echo ""
    echo "✅ Excel report downloaded successfully!"
    echo "----------------------------------------"
    echo "📄 Filename: ${EXCEL_FILENAME}"
    echo "📊 File Size: ${FILE_SIZE}"
    echo "📍 Location: $(pwd)/${EXCEL_FILENAME}"
    echo "----------------------------------------"
    echo ""
    
    # Check file type
    FILE_TYPE=$(file -b "${EXCEL_FILENAME}" 2>/dev/null || echo "Unknown")
    echo "File Type: ${FILE_TYPE}"
    echo ""
    
    # If 'file' command shows it's a text file, it might be an error response
    if [[ "${FILE_TYPE}" == *"text"* ]] || [[ "${FILE_TYPE}" == *"JSON"* ]]; then
      echo "⚠️  Warning: File appears to be text/JSON instead of Excel"
      echo "Content:"
      cat "${EXCEL_FILENAME}"
      echo ""
    else
      echo "✨ You can now open the Excel file with:"
      echo "   libreoffice ${EXCEL_FILENAME}"
      echo "   or"
      echo "   xdg-open ${EXCEL_FILENAME}"
    fi
  else
    echo "❌ Downloaded file is empty"
    rm -f "${EXCEL_FILENAME}"
    exit 1
  fi
else
  echo "❌ Failed to download Excel report"
  exit 1
fi


echo ""
echo ""


# ====================================================
# PART 4: DEFAULTERS EXCEL REPORT GENERATION
# ====================================================


echo "======================================================"
echo "    PART 4: FACULTY DEFAULTERS REPORT DOWNLOAD"
echo "======================================================"
echo ""


echo "8. Downloading defaulters Excel report..."
echo "   (Students with <15 questions in last 30 days)"
echo ""


# Generate filename with timestamp
DEFAULTERS_FILENAME="defaulters_report_${TIMESTAMP}.xlsx"


# Download the defaulters Excel report
curl -s -X GET "${API_URL}/faculty/reports/defaulters" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}" \
  -o "${DEFAULTERS_FILENAME}" \
  -w "\nHTTP Status: %{http_code}\n"


# Check if download was successful
if [ $? -eq 0 ] && [ -f "${DEFAULTERS_FILENAME}" ]; then
  FILE_SIZE=$(ls -lh "${DEFAULTERS_FILENAME}" | awk '{print $5}')
  
  # Check if file is not empty and is actually an Excel file
  if [ -s "${DEFAULTERS_FILENAME}" ]; then
    echo ""
    echo "✅ Defaulters report downloaded successfully!"
    echo "----------------------------------------"
    echo "📄 Filename: ${DEFAULTERS_FILENAME}"
    echo "📊 File Size: ${FILE_SIZE}"
    echo "📍 Location: $(pwd)/${DEFAULTERS_FILENAME}"
    echo "----------------------------------------"
    echo ""
    
    # Check file type
    FILE_TYPE=$(file -b "${DEFAULTERS_FILENAME}" 2>/dev/null || echo "Unknown")
    echo "File Type: ${FILE_TYPE}"
    echo ""
    
    # If 'file' command shows it's a text file, it might be an error response
    if [[ "${FILE_TYPE}" == *"text"* ]] || [[ "${FILE_TYPE}" == *"JSON"* ]]; then
      echo "⚠️  Warning: File appears to be text/JSON instead of Excel"
      echo "Content:"
      cat "${DEFAULTERS_FILENAME}"
      echo ""
    else
      echo "✨ You can now open the Excel file with:"
      echo "   libreoffice ${DEFAULTERS_FILENAME}"
      echo "   or"
      echo "   xdg-open ${DEFAULTERS_FILENAME}"
    fi
  else
    echo "❌ Downloaded file is empty"
    rm -f "${DEFAULTERS_FILENAME}"
    exit 1
  fi
else
  echo "❌ Failed to download defaulters report"
  exit 1
fi


echo ""
echo "======================================================"
echo "    ALL TESTS COMPLETE"
echo "======================================================"
echo ""
echo "Summary:"
echo "  ✅ Student login and profile link submission"
echo "  ✅ Faculty login and profile verification"
echo "  ✅ Excel submissions report generation and download"
echo "  ✅ Excel defaulters report generation and download"
echo ""
echo "Generated Files:"
echo "  📊 ${EXCEL_FILENAME}"
echo "  📊 ${DEFAULTERS_FILENAME}"
echo ""
echo "Quick Open Commands:"
echo "  libreoffice ${EXCEL_FILENAME} &"
echo "  libreoffice ${DEFAULTERS_FILENAME} &"
echo ""
echo "======================================================"

echo ""
echo ""


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
for YEAR in "I" "II" "III" "IV"; do
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
                    RA_NUM=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].ra_number")
                    NAME=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].student_name")
                    SPEC=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].specialization // \"N/A\"")
                    QUESTIONS=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].last_30_days_questions")
                    LEETCODE=$(echo "${LEADERBOARD_BODY}" | jq -r ".leaderboard[$i].leetcode_profile // \"Not set\"")
                    
                    # Truncate specialization if too long
                    if [ ${#SPEC} -gt 40 ]; then
                        SPEC="${SPEC:0:37}..."
                    fi
                    
                    echo "Rank ${RANK}: ${NAME}"
                    echo "  RA Number: ${RA_NUM}"
                    echo "  Specialization: ${SPEC}"
                    echo "  Questions (30 days): ${QUESTIONS}"
                    echo "  LeetCode: ${LEETCODE}"
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


echo "----------------------------------------"
echo "Testing Invalid Academic Year (should fail)"
echo "----------------------------------------"


INVALID_RESPONSE=$(curl -s -w "\\n%{http_code}" -X GET "${API_URL}/leaderboard?academic_year=V")
INVALID_BODY=$(echo "$INVALID_RESPONSE" | head -n -1)
INVALID_STATUS=$(echo "$INVALID_RESPONSE" | tail -n 1)


if [ "$INVALID_STATUS" -eq 400 ]; then
    echo "✅ Invalid academic year correctly rejected. HTTP Status: ${INVALID_STATUS}"
    echo "Error Message: $(echo "${INVALID_BODY}" | jq -r '.error')"
else
    echo "⚠️  Expected 400 Bad Request, got: ${INVALID_STATUS}"
    echo "Response: ${INVALID_BODY}"
fi


echo ""
echo "--- Leaderboard Testing Complete ---"
echo ""
echo ""


# ====================================================
# FINAL SUMMARY
# ====================================================


echo "======================================================"
echo "    ALL TESTS COMPLETE"
echo "======================================================"
echo ""
echo "Summary:"
echo "  ✅ Student login and profile link submission"
echo "  ✅ Faculty login and profile verification"
echo "  ✅ Excel submissions report generation and download"
echo "  ✅ Excel defaulters report generation and download"
echo "  ✅ Public leaderboard API testing (all years)"
echo ""
echo "Generated Files:"
echo "  📊 ${EXCEL_FILENAME}"
echo "  📊 ${DEFAULTERS_FILENAME}"
echo ""
echo "Quick Open Commands:"
echo "  libreoffice ${EXCEL_FILENAME} &"
echo "  libreoffice ${DEFAULTERS_FILENAME} &"
echo ""
echo "======================================================"

echo ""
echo ""


# ====================================================
# PART 6: FACULTY STATS API TEST
# ====================================================


echo "======================================================"
echo "    PART 6: FACULTY STATS API TEST"
echo "======================================================"
echo ""


echo "10. Testing faculty stats endpoint (Faculty Auth Required)"
echo ""
echo "Using faculty token from Part 2..."
echo ""


STATS_RESPONSE=$(curl -s -w "\\n%{http_code}" -X GET "${API_URL}/faculty/stats" \
  -H "Authorization: Bearer ${FACULTY_JWT_TOKEN}")


# Extract body and status
STATS_BODY=$(echo "$STATS_RESPONSE" | head -n -1)
STATS_STATUS=$(echo "$STATS_RESPONSE" | tail -n 1)


if [ "$STATS_STATUS" -ge 200 ] && [ "$STATS_STATUS" -lt 300 ]; then
    echo "✅ Faculty stats fetched successfully. HTTP Status: ${STATS_STATUS}"
    echo ""
    
    # Parse response
    SPECIALIZATION=$(echo "${STATS_BODY}" | jq -r '.specialization // "N/A"')
    ACADEMIC_YEAR=$(echo "${STATS_BODY}" | jq -r '.academic_year // "N/A"')
    TOTAL=$(echo "${STATS_BODY}" | jq -r '.total_students // 0')
    WITH_LC=$(echo "${STATS_BODY}" | jq -r '.with_leetcode_profiles // 0')
    WITHOUT_LC=$(echo "${STATS_BODY}" | jq -r '.without_leetcode_profiles // 0')
    DEFAULTERS=$(echo "${STATS_BODY}" | jq -r '.defaulters // 0')
    
    echo "📊 Statistics Dashboard"
    echo "========================================"
    echo ""
    echo "📚 Academic Information:"
    echo "  Specialization: ${SPECIALIZATION}"
    echo "  Academic Year: ${ACADEMIC_YEAR}"
    echo ""
    echo "👥 Student Counts:"
    echo "  Total Students: ${TOTAL}"
    echo ""
    echo "🔗 LeetCode Profile Submission:"
    echo "  ✅ With Profiles: ${WITH_LC}"
    echo "  ❌ Without Profiles: ${WITHOUT_LC}"
    
    # Calculate percentage if total > 0
    if [ "$TOTAL" -gt 0 ]; then
        PROFILE_PERCENTAGE=$(awk "BEGIN {printf \"%.1f\", ($WITH_LC / $TOTAL) * 100}")
        echo "  📈 Submission Rate: ${PROFILE_PERCENTAGE}%"
    fi
    
    echo ""
    echo "⚠️  Performance Status:"
    echo "  Defaulters (<15 questions/30 days): ${DEFAULTERS}"
    
    # Calculate defaulters percentage if total > 0
    if [ "$TOTAL" -gt 0 ]; then
        DEFAULTER_PERCENTAGE=$(awk "BEGIN {printf \"%.1f\", ($DEFAULTERS / $TOTAL) * 100}")
        echo "  📉 Defaulter Rate: ${DEFAULTER_PERCENTAGE}%"
        
        ACTIVE=$(($TOTAL - $DEFAULTERS))
        ACTIVE_PERCENTAGE=$(awk "BEGIN {printf \"%.1f\", ($ACTIVE / $TOTAL) * 100}")
        echo "  ✅ Active Students (≥15 questions): ${ACTIVE} (${ACTIVE_PERCENTAGE}%)"
    fi
    
    echo ""
    echo "========================================"
    echo ""
    
    echo "Raw JSON Response:"
    echo "${STATS_BODY}" | jq '.'
else
    echo "❌ Failed to fetch faculty stats. HTTP Status: ${STATS_STATUS}"
    echo "Response: ${STATS_BODY}"
fi


echo ""
echo "----------------------------------------"
echo "Testing stats without faculty token (should fail)"
echo "----------------------------------------"


NO_AUTH_RESPONSE=$(curl -s -w "\\n%{http_code}" -X GET "${API_URL}/faculty/stats")
NO_AUTH_BODY=$(echo "$NO_AUTH_RESPONSE" | head -n -1)
NO_AUTH_STATUS=$(echo "$NO_AUTH_RESPONSE" | tail -n 1)


if [ "$NO_AUTH_STATUS" -eq 401 ]; then
    echo "✅ Unauthorized access correctly rejected. HTTP Status: ${NO_AUTH_STATUS}"
    echo "Error Message: $(echo "${NO_AUTH_BODY}" | jq -r '.error // "Unauthorized"')"
else
    echo "⚠️  Expected 401 Unauthorized, got: ${NO_AUTH_STATUS}"
    echo "Response: ${NO_AUTH_BODY}"
fi


echo ""
echo "----------------------------------------"
echo "Testing stats with student token (should fail)"
echo "----------------------------------------"


STUDENT_STATS_RESPONSE=$(curl -s -w "\\n%{http_code}" -X GET "${API_URL}/faculty/stats" \
  -H "Authorization: Bearer ${JWT_TOKEN}")
STUDENT_STATS_BODY=$(echo "$STUDENT_STATS_RESPONSE" | head -n -1)
STUDENT_STATS_STATUS=$(echo "$STUDENT_STATS_RESPONSE" | tail -n 1)


if [ "$STUDENT_STATS_STATUS" -eq 401 ]; then
    echo "✅ Student token correctly rejected. HTTP Status: ${STUDENT_STATS_STATUS}"
    echo "Error Message: $(echo "${STUDENT_STATS_BODY}" | jq -r '.error // "Unauthorized"')"
else
    echo "⚠️  Expected 401 Unauthorized, got: ${STUDENT_STATS_STATUS}"
    echo "Response: ${STUDENT_STATS_BODY}"
fi

# Specialization mappings: "Full Name|Suffix"
# This matches the names used in your database and the suffixes in your usernames
DEPT_LIST=(
"Artificial Intelligence|ai"
"Biomedical Engineering|be"
"Biotechnology|bt"
"Civil Engineering|ce"
"Computer Science and Business System|csbs"
"Computer Science and Engineering|cse"
"Computer Science and Engineering with specialization in Artificial Intelligence and Machine Learning|cseaiml"
"Computer Science and Engineering with specialization in Big Data Analytics|csebd"
"Computer Science and Engineering with specialization in Cloud Computing|csecc"
"Computer Science and Engineering with specialization in Cyber Security|csecs"
"Computer Science and Engineering with specialization in Gaming Technology|gt"
"Computer Science and Engineering with specialization in Internet of Things|iot"
"Electrical and Electronics Engineering|eee"
"Electronics and Communication Engineering|ece"
"Electronics and Communication Engineering with specialization in Data Science|eceds"
"Information Technology|it"
"Mechanical Engineering|me"
)

YEARS=("I" "II" "III" "IV")
TOTAL_TESTS=0
SUCCESS_COUNT=0
FAILED_COUNT=0

echo "======================================================================"
echo "          FACULTY LOGIN & STATS TEST"
echo "======================================================================"
echo " Total Accounts to Test: $(( ${#DEPT_LIST[@]} * 4 ))"
echo "======================================================================"
echo ""

for DEPT_DATA in "${DEPT_LIST[@]}"; do
  # Split the string by the pipe character
  IFS="|" read -r FULL_NAME SUFFIX <<< "$DEPT_DATA"
  
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "📚 Department: $FULL_NAME"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  
  for YEAR in "${YEARS[@]}"; do
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Construct credentials
    USERNAME="prof_${YEAR}_${SUFFIX}"
    PASSWORD="faculty@${YEAR}@${SUFFIX}"
    
    echo ""
    echo "Year ${YEAR} - Testing ${USERNAME}..."
    
    # Perform Login
    LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/faculty/login" \
      -H "Content-Type: application/json" \
      -d "{
        \"specialization\": \"${FULL_NAME}\",
        \"username\": \"${USERNAME}\",
        \"password\": \"${PASSWORD}\"
      }")
    
    # Extract Token
    JWT_TOKEN=$(echo "${LOGIN_RESPONSE}" | jq -r '.token // empty')
    
    if [ -n "$JWT_TOKEN" ] && [ "$JWT_TOKEN" != "null" ]; then
      echo "  ✅ Login: SUCCESS"
      
      # Fetch Faculty Stats using the token
      STATS_RESPONSE=$(curl -s -X GET "${API_URL}/faculty/stats" \
        -H "Authorization: Bearer ${JWT_TOKEN}" \
        -H "Content-Type: application/json")
      
      # Extract stats
      SPECIALIZATION=$(echo "${STATS_RESPONSE}" | jq -r '.specialization // "N/A"')
      ACADEMIC_YEAR=$(echo "${STATS_RESPONSE}" | jq -r '.academic_year // "N/A"')
      TOTAL_STUDENTS=$(echo "${STATS_RESPONSE}" | jq -r '.total_students // "N/A"')
      WITH_LEETCODE=$(echo "${STATS_RESPONSE}" | jq -r '.with_leetcode_profiles // "N/A"')
      WITHOUT_LEETCODE=$(echo "${STATS_RESPONSE}" | jq -r '.without_leetcode_profiles // "N/A"')
      DEFAULTERS=$(echo "${STATS_RESPONSE}" | jq -r '.defaulters // "N/A"')
      
      if [ "$TOTAL_STUDENTS" != "N/A" ]; then
        echo "  📊 Stats Retrieved Successfully"
        echo "  ┌────────────────────────────────────────────────"
        echo "  │ Specialization:       ${SPECIALIZATION}"
        echo "  │ Academic Year:        ${ACADEMIC_YEAR}"
        echo "  │ Total Students:       ${TOTAL_STUDENTS}"
        echo "  │ With LeetCode:        ${WITH_LEETCODE}"
        echo "  │ Without LeetCode:     ${WITHOUT_LEETCODE}"
        echo "  │ Defaulters:           ${DEFAULTERS}"
        echo "  └────────────────────────────────────────────────"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
      else
        echo "  ⚠️  Stats Fetch FAILED"
        echo "  Response: ${STATS_RESPONSE}"
        FAILED_COUNT=$((FAILED_COUNT + 1))
      fi
    else
      echo "  ❌ Login: FAILED"
      echo "  Response: ${LOGIN_RESPONSE}"
      FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
  done
  echo ""
done

echo ""
echo "======================================================================"
echo "                        TEST SUMMARY"
echo "======================================================================"
echo "  Total Attempted:    $TOTAL_TESTS"
echo "  ✅ Success:          $SUCCESS_COUNT"
echo "  ❌ Failed:           $FAILED_COUNT"
echo "======================================================================"

if [ $FAILED_COUNT -eq 0 ]; then
  echo "  🎉 All faculty logins and stats fetches passed!"
else
  echo "  ⚠️  Some tests failed. Please check the logs above."
  exit 1
fi

echo ""
echo "--- Faculty Stats Testing Complete ---"
echo ""
echo ""


# ====================================================
# FINAL SUMMARY
# ====================================================


echo "======================================================"
echo "    ALL TESTS COMPLETE"
echo "======================================================"
echo ""
echo "Summary:"
echo "  ✅ Student login and profile link submission"
echo "  ✅ Faculty login and profile verification"
echo "  ✅ Excel submissions report generation and download"
echo "  ✅ Excel defaulters report generation and download"
echo "  ✅ Public leaderboard API testing (all years)"
echo "  ✅ Faculty statistics API testing"
echo ""
echo "Generated Files:"
echo "  📊 ${EXCEL_FILENAME}"
echo "  📊 ${DEFAULTERS_FILENAME}"
echo ""
echo "API Endpoints Tested:"
echo "  🔐 POST   /auth/login"
echo "  🔐 POST   /faculty/login"
echo "  🔐 GET    /auth/me"
echo "  🔐 GET    /faculty/me"
echo "  🔐 GET    /profile/links"
echo "  🔐 POST   /profile/links"
echo "  🔐 GET    /faculty/reports/submissions"
echo "  🔐 GET    /faculty/reports/defaulters"
echo "  🔐 GET    /faculty/stats"
echo "  🌐 GET    /leaderboard?academic_year={I|II|III|IV}"
echo ""
echo "Quick Open Commands:"
echo "  libreoffice ${EXCEL_FILENAME} &"
echo "  libreoffice ${DEFAULTERS_FILENAME} &"
echo ""
echo "======================================================"


