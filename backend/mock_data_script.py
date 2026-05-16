import os
import bcrypt
import mysql.connector
from urllib.parse import urlparse
from datetime import datetime, timedelta
from dotenv import load_dotenv

load_dotenv()

DATABASE_URL = os.getenv("DATABASE_URL")
if not DATABASE_URL:
    raise ValueError("DATABASE_URL not found in .env")

parsed = urlparse(DATABASE_URL)

db_config = {
    "host": parsed.hostname or "localhost",
    "port": parsed.port or 3306,
    "user": parsed.username,
    "password": parsed.password,
    "database": parsed.path.lstrip("/"),
}

conn = mysql.connector.connect(**db_config)
cursor = conn.cursor()


def fetch_one(sql, params=None):
    cursor.execute(sql, params or ())
    return cursor.fetchone()


def fetch_all(sql, params=None):
    cursor.execute(sql, params or ())
    return cursor.fetchall()


def insert(sql, params=None):
    cursor.execute(sql, params or ())
    return cursor.lastrowid


def exists(sql, params=None):
    cursor.execute(sql, params or ())
    return cursor.fetchone() is not None


# ── Read existing data ──────────────────────────────────────
eng = fetch_one("SELECT id FROM departments WHERE short_name = 'ENG'")
engineering_id = eng[0] if eng else 1

admin = fetch_one("SELECT id FROM users WHERE email = 'admin@demo.com'")
admin_id = admin[0] if admin else 3

sarah = fetch_one("SELECT id FROM users WHERE email = 'manager@demo.com'")
sarah_id = sarah[0] if sarah else 4

alex = fetch_one("SELECT id FROM users WHERE email = 'employee@demo.com'")
alex_id = alex[0] if alex else 5

cycle = fetch_one("SELECT id FROM goal_cycles WHERE is_active = 1")
cycle_id = cycle[0] if cycle else 1

thrust_areas = fetch_all("SELECT id, name FROM thrust_areas")
print(f"Found: dept={engineering_id}, admin={admin_id}, sarah={sarah_id}, alex={alex_id}, cycle={cycle_id}")
print(f"Thrust areas: {[(r[0], r[1]) for r in thrust_areas]}")

ta_ids = [r[0] for r in thrust_areas]
ta_product = ta_ids[0] if len(ta_ids) > 0 else None
ta_customer = ta_ids[1] if len(ta_ids) > 1 else None
ta_process = ta_ids[2] if len(ta_ids) > 2 else None
ta_innovation = ta_ids[3] if len(ta_ids) > 3 else None

plain_password = "password123"
hashed_password = bcrypt.hashpw(
    plain_password.encode("utf-8"), bcrypt.gensalt()
).decode("utf-8")

now = datetime.now()

# ═══════════════════════════════════════════════════════════
#  1. ADDITIONAL EMPLOYEES
# ═══════════════════════════════════════════════════════════

employees = []

for name, email in [
    ("Maria Garcia", "maria@demo.com"),
    ("James Wilson", "james@demo.com"),
]:
    row = fetch_one("SELECT id FROM users WHERE email = %s", (email,))
    if row:
        uid = row[0]
        print(f"→ {name} ({email}) already exists (id={uid})")
    else:
        uid = insert(
            "INSERT INTO users (email, password_hash, full_name, department_id, role, manager_id) "
            "VALUES (%s, %s, %s, %s, %s, %s)",
            (email, hashed_password, name, engineering_id, "employee", sarah_id),
        )
        print(f"✓ Created {name} ({email}) id={uid}")
    employees.append((uid, name, email))

# ═══════════════════════════════════════════════════════════
#  2. GOAL SHEETS
# ═══════════════════════════════════════════════════════════

# alex — submitted
alex_sheet = fetch_one("SELECT id, status FROM goal_sheets WHERE user_id = %s AND cycle_id = %s", (alex_id, cycle_id))
if alex_sheet:
    alex_sheet_id = alex_sheet[0]
    if alex_sheet[1] != "submitted":
        cursor.execute("UPDATE goal_sheets SET status='submitted', submitted_at=%s, approved_at=NULL, approved_by=NULL WHERE id=%s", (now, alex_sheet_id))
        print(f"✓ Reset Alex's sheet to submitted (was {alex_sheet[1]})")
    else:
        print(f"→ Alex's sheet already submitted (id={alex_sheet_id})")
else:
    alex_sheet_id = insert(
        "INSERT INTO goal_sheets (user_id, cycle_id, status, submitted_at) VALUES (%s, %s, 'submitted', %s)",
        (alex_id, cycle_id, now),
    )
    print(f"✓ Created Alex's goal sheet (id={alex_sheet_id})")

# maria — approved/locked
maria_id = employees[0][0]
maria_sheet = fetch_one("SELECT id, status FROM goal_sheets WHERE user_id = %s AND cycle_id = %s", (maria_id, cycle_id))
if maria_sheet:
    maria_sheet_id = maria_sheet[0]
    if maria_sheet[1] != "locked":
        cursor.execute("UPDATE goal_sheets SET status='locked', submitted_at=%s, approved_at=%s, approved_by=%s WHERE id=%s", (now - timedelta(days=7), now - timedelta(days=5), sarah_id, maria_sheet_id))
        print(f"✓ Reset Maria's sheet to locked (was {maria_sheet[1]})")
    else:
        print(f"→ Maria's sheet already locked (id={maria_sheet_id})")
else:
    maria_sheet_id = insert(
        "INSERT INTO goal_sheets (user_id, cycle_id, status, submitted_at, approved_at, approved_by) "
        "VALUES (%s, %s, 'locked', %s, %s, %s)",
        (maria_id, cycle_id, now - timedelta(days=7), now - timedelta(days=5), sarah_id),
    )
    print(f"✓ Created Maria's goal sheet (id={maria_sheet_id})")

# james — draft
james_id = employees[1][0]
james_sheet = fetch_one("SELECT id, status FROM goal_sheets WHERE user_id = %s AND cycle_id = %s", (james_id, cycle_id))
if james_sheet:
    james_sheet_id = james_sheet[0]
    if james_sheet[1] != "draft":
        cursor.execute("UPDATE goal_sheets SET status='draft', submitted_at=NULL, approved_at=NULL, approved_by=NULL WHERE id=%s", (james_sheet_id,))
        print(f"✓ Reset James's sheet to draft (was {james_sheet[1]})")
    else:
        print(f"→ James's sheet already draft (id={james_sheet_id})")
else:
    james_sheet_id = insert(
        "INSERT INTO goal_sheets (user_id, cycle_id, status) VALUES (%s, %s, 'draft')",
        (james_id, cycle_id),
    )
    print(f"✓ Created James's goal sheet (id={james_sheet_id})")

# ═══════════════════════════════════════════════════════════
#  3. GOALS
# ═══════════════════════════════════════════════════════════

def add_goal(sheet_id, title, uom_type, target_value, weightage, thrust_area_id=None, description=None, target_date=None):
    cursor.execute("SELECT id FROM goals WHERE sheet_id = %s AND title = %s", (sheet_id, title))
    row = cursor.fetchone()
    if row:
        return row[0]
    return insert(
        """INSERT INTO goals (sheet_id, thrust_area_id, title, description, uom_type, target_value, target_date, weightage, is_shared, sort_order)
           VALUES (%s, %s, %s, %s, %s, %s, %s, %s, 0, 0)""",
        (sheet_id, thrust_area_id, title, description, uom_type, target_value, target_date, weightage),
    )

# ── Alex (submitted) ──
alex_goals = [
    ("Ship 10 product features this quarter", "min_numeric", 10.0, 40, ta_product, "Deliver 10 customer-facing features across the platform"),
    ("Maintain 95% code coverage", "min_percent", 95.0, 30, ta_process, "Ensure all new code meets 95% test coverage threshold"),
    ("Launch Q1 product release on time", "timeline", 0.0, 30, ta_innovation, "Complete and ship Q1 release by March 31", "2026-03-31"),
]
alex_goal_ids = []
for title, uom, target, wt, ta, *rest in alex_goals:
    desc = rest[0] if rest else None
    dt = rest[1] if len(rest) > 1 else None
    gid = add_goal(alex_sheet_id, title, uom, target, wt, ta, desc, dt)
    if gid:
        alex_goal_ids.append(gid)
        print(f"  → Alex: '{title}' (id={gid})")

# ── Maria (locked, with achievements) ──
maria_goals = [
    ("Keep critical bugs under 5", "max_numeric", 5.0, 35, ta_product, "Ensure fewer than 5 P1/P2 bugs are reported per quarter"),
    ("Zero P0 security incidents", "zero", 0.0, 35, ta_process, "No critical security incidents throughout the quarter"),
    ("Achieve 90% on-time delivery", "min_percent", 90.0, 30, ta_customer, "Deliver all sprint commitments on schedule"),
]
maria_goal_ids = []
for title, uom, target, wt, ta, desc in maria_goals:
    gid = add_goal(maria_sheet_id, title, uom, target, wt, ta, desc)
    if gid:
        maria_goal_ids.append(gid)
        print(f"  → Maria: '{title}' (id={gid})")

# ── James (draft, incomplete weightage) ──
james_goals = [
    ("Complete 20 peer code reviews", "min_numeric", 20.0, 25, ta_process, "Review at least 20 PRs from team members"),
    ("Maintain 80% test pass rate", "min_percent", 80.0, 25, ta_process, "Keep CI test pass rate above 80%"),
    ("Keep average response time under 2 hours", "max_numeric", 2.0, 20, ta_customer, "Maintain sub-2 hour average response to production issues"),
]
james_goal_ids = []
for title, uom, target, wt, ta, desc in james_goals:
    gid = add_goal(james_sheet_id, title, uom, target, wt, ta, desc)
    if gid:
        james_goal_ids.append(gid)
        print(f"  → James: '{title}' (id={gid})")

# ═══════════════════════════════════════════════════════════
#  4. ACHIEVEMENTS (Maria's locked sheet — Q1 actuals)
# ═══════════════════════════════════════════════════════════

def compute_score(uom_type, target, actual):
    if uom_type == "min_numeric" or uom_type == "min_percent":
        return min(actual / target, 1.0) * 100.0
    elif uom_type == "max_numeric" or uom_type == "max_percent":
        return min(target / actual, 1.0) * 100.0
    elif uom_type == "timeline":
        return 100.0 if actual <= target else 0.0
    elif uom_type == "zero":
        return 100.0 if actual == 0.0 else 0.0
    return 0.0

maria_q1_data = [
    (maria_goal_ids[0], "max_numeric", 5.0, 3.0, "completed"),   # 3 bugs — 100%
    (maria_goal_ids[1], "zero", 0.0, 0.0, "completed"),           # 0 incidents — 100%
    (maria_goal_ids[2], "min_percent", 90.0, 88.0, "on_track"),   # 88% — 97.8%
]

for gid, uom, target, actual, status in maria_q1_data:
    if gid is None:
        continue
    score = compute_score(uom, target, actual)
    if not exists("SELECT id FROM achievements WHERE goal_id = %s AND quarter = 'q1'", (gid,)):
        insert(
            "INSERT INTO achievements (goal_id, quarter, actual_value, actual_date, status, computed_score) "
            "VALUES (%s, 'q1', %s, %s, %s, %s)",
            (gid, actual, now - timedelta(days=30), status, score),
        )
        print(f"  ✓ Q1 achievement for goal {gid}: actual={actual}, score={score:.1f}%")

# ═══════════════════════════════════════════════════════════
#  5. MANAGER CHECK-IN (Maria's sheet, Q1)
# ═══════════════════════════════════════════════════════════

if not exists(
    "SELECT id FROM checkin_comments WHERE goal_sheet_id = %s AND quarter = 'q1' AND manager_id = %s",
    (maria_sheet_id, sarah_id),
):
    insert(
        "INSERT INTO checkin_comments (goal_sheet_id, quarter, manager_id, comment) "
        "VALUES (%s, 'q1', %s, %s)",
        (maria_sheet_id, sarah_id, "Great progress in Q1. Security metrics are on track. Let's push the on-time delivery above 90% in Q2."),
    )
    print("✓ Check-in comment added for Maria's Q1")

# ═══════════════════════════════════════════════════════════
#  6. AUDIT LOG ENTRY (sheet lock)
# ═══════════════════════════════════════════════════════════

if not exists(
    "SELECT id FROM audit_log WHERE table_name = 'goal_sheets' AND record_id = %s",
    (maria_sheet_id,),
):
    insert(
        "INSERT INTO audit_log (table_name, record_id, field_name, old_value, new_value, changed_by) "
        "VALUES (%s, %s, %s, %s, %s, %s)",
        ("goal_sheets", maria_sheet_id, "status", "submitted", "locked", sarah_id),
    )
    print("✓ Audit log entry added")

conn.commit()
print("\n" + "═" * 50)
print("✅ Mock data seeded successfully!")
print("═" * 50)
print()
print("All 3 employees now have goal sheets in different states:")
print("  ├─ Alex Johnson   — submitted  (3 goals, ready for review)")
print("  ├─ Maria Garcia   — locked     (3 goals, Q1 achievements, check-in)")
print("  └─ James Wilson   — draft      (3 goals, incomplete weightage)")
print()
print("Demo credentials (all password: password123):")
print("  Admin:    admin@demo.com    — David Park")
print("  Manager:  manager@demo.com  — Sarah Chen")
print("  Employee: employee@demo.com — Alex Johnson")
print("  Employee: maria@demo.com    — Maria Garcia")
print("  Employee: james@demo.com    — James Wilson")

cursor.close()
conn.close()
