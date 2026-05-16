import os
import bcrypt
import mysql.connector
from urllib.parse import urlparse
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


def ensure_department(cursor, name: str, short_name: str) -> int:
    cursor.execute("SELECT id FROM departments WHERE short_name = %s LIMIT 1", (short_name,))
    row = cursor.fetchone()
    if row:
        return row[0]
    cursor.execute(
        "INSERT INTO departments (name, short_name) VALUES (%s, %s)",
        (name, short_name),
    )
    return cursor.lastrowid


def ensure_thrust_areas(cursor, department_id: int, created_by: int):
    areas = [
        "Product Development",
        "Customer Success",
        "Process Improvement",
        "Innovation & Research",
    ]
    for area in areas:
        cursor.execute(
            "SELECT id FROM thrust_areas WHERE name = %s LIMIT 1", (area,)
        )
        if not cursor.fetchone():
            cursor.execute(
                "INSERT INTO thrust_areas (name, department_id, created_by) VALUES (%s, %s, %s)",
                (area, department_id, created_by),
            )


def ensure_goal_cycle(cursor, created_by: int):
    cursor.execute("SELECT id FROM goal_cycles LIMIT 1")
    if cursor.fetchone():
        return
    cursor.execute(
        """INSERT INTO goal_cycles (name, goal_setting_opens, q1_opens, q2_opens, q3_opens, q4_opens, is_active, created_by)
           VALUES (%s, NOW(), NOW(), NOW(), NOW(), NOW(), 1, %s)""",
        ("FY 2026-27", created_by),
    )


plain_password = "password123"
hashed_password = bcrypt.hashpw(
    plain_password.encode("utf-8"),
    bcrypt.gensalt(),
).decode("utf-8")

conn = mysql.connector.connect(**db_config)
cursor = conn.cursor()

try:
    # 1. Ensure Engineering department
    engineering_id = ensure_department(cursor, "Engineering", "ENG")
    print(f"✓ Engineering department (id={engineering_id})")

    # 2. Insert Admin
    cursor.execute(
        "SELECT id FROM users WHERE email = %s LIMIT 1", ("admin@demo.com",)
    )
    admin_row = cursor.fetchone()
    if admin_row:
        admin_id = admin_row[0]
        print(f"→ Admin already exists (id={admin_id})")
    else:
        cursor.execute(
            """INSERT INTO users (email, password_hash, full_name, department_id, role, manager_id)
               VALUES (%s, %s, %s, %s, %s, %s)""",
            ("admin@demo.com", hashed_password, "David Park", None, "admin", None),
        )
        admin_id = cursor.lastrowid
        print(f"✓ Admin created (id={admin_id})")

    # 3. Insert Manager
    cursor.execute(
        "SELECT id FROM users WHERE email = %s LIMIT 1", ("manager@demo.com",)
    )
    mgr_row = cursor.fetchone()
    if mgr_row:
        manager_id = mgr_row[0]
        print(f"→ Manager already exists (id={manager_id})")
    else:
        cursor.execute(
            """INSERT INTO users (email, password_hash, full_name, department_id, role, manager_id)
               VALUES (%s, %s, %s, %s, %s, %s)""",
            ("manager@demo.com", hashed_password, "Sarah Chen", engineering_id, "manager", None),
        )
        manager_id = cursor.lastrowid
        print(f"✓ Manager created (id={manager_id})")

    # 4. Insert Employee (reports to Manager)
    cursor.execute(
        "SELECT id FROM users WHERE email = %s LIMIT 1", ("employee@demo.com",)
    )
    emp_row = cursor.fetchone()
    if emp_row:
        print(f"→ Employee already exists (id={emp_row[0]})")
    else:
        cursor.execute(
            """INSERT INTO users (email, password_hash, full_name, department_id, role, manager_id)
               VALUES (%s, %s, %s, %s, %s, %s)""",
            ("employee@demo.com", hashed_password, "Alex Johnson", engineering_id, "employee", manager_id),
        )
        print(f"✓ Employee created (id={cursor.lastrowid})")

    # 5. Seed thrust areas
    ensure_thrust_areas(cursor, engineering_id, admin_id)
    print("✓ Thrust areas seeded")

    # 6. Seed active goal cycle
    ensure_goal_cycle(cursor, admin_id)
    print("✓ Goal cycle seeded")

    conn.commit()
    print("\n✅ Demo data inserted successfully!")
    print("   ├─ admin@demo.com / password123  (Admin)")
    print("   ├─ manager@demo.com / password123 (Manager)")
    print("   └─ employee@demo.com / password123 (Employee)")

except mysql.connector.Error as err:
    conn.rollback()
    print(f"Database error: {err}")

except ValueError as err:
    conn.rollback()
    print(f"Validation error: {err}")

finally:
    cursor.close()
    conn.close()
