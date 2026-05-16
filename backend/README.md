# AtomQuest — Backend

Goal Setting & Tracking Portal backend (Rust + Axum + MySQL).

---

## Quick Start

### 1. Database setup

Create a MySQL database and set the connection string in `.env`:

```env
DATABASE_URL=mysql://user:password@localhost:3306/atomquest
JWT_SECRET=your-secret-key
ADMIN_SEED_PASSWORD=password123
```

### 2. Run backend

```bash
cargo run
```

On first start, the backend runs migrations and seeds demo data automatically.

### 3. Seed demo data (if tables exist but no data)

```bash
# Users + department + goal cycle
uv run --with bcrypt --with mysql-connector-python --with python-dotenv python users_insert_script.py

# Full mock data (goals, achievements, check-ins)
uv run --with bcrypt --with mysql-connector-python --with python-dotenv python mock_data_script.py
```

---

## Demo Users

All share the password: **`password123`**

| Role | Email | Name | Department |
|---|---|---|---|
| Admin | `admin@demo.com` | David Park | — |
| Manager | `manager@demo.com` | Sarah Chen | Engineering |
| Employee | `employee@demo.com` | Alex Johnson | Engineering |
| Employee | `maria@demo.com` | Maria Garcia | Engineering |
| Employee | `james@demo.com` | James Wilson | Engineering |

---

## Demo Walkthrough Scenarios

### 1. Employee journey — Alex Johnson (`employee@demo.com`)

Alex has a **submitted** goal sheet with 3 goals ready for manager review:

| Goal | UoM | Target | Weightage |
|---|---|---|---|
| Ship 10 product features | min_numeric | 10 | 40% |
| Maintain 95% code coverage | min_percent | 95% | 30% |
| Launch Q1 release on time | timeline | 2026-03-31 | 30% |

**What to demo:**
- Login → see dashboard with "submitted" sheet
- Click into sheet → view goals (read-only since submitted)
- Weightage bar shows 100%

### 2. Manager journey — Sarah Chen (`manager@demo.com`)

Sarah sees her whole team (Alex, Maria, James) with different sheet states.

**Maria Garcia** has a **locked** sheet (fully processed):
- 3 goals with Q1 achievements logged
- Q1 scores visible: 100%, 100%, 97.8%
- Check-in comment she left in Q1

**Alex Johnson** has a **submitted** sheet waiting for review:
- Sarah can Approve (locks the sheet) or Return with a reason

**What to demo:**
- Dashboard shows pending approvals count
- Review Alex's sheet → click Approve or Return
- View Maria's check-in data with actual vs target
- Add a check-in comment for Q1

### 3. Employee journey — James Wilson (`james@demo.com`)

James has a **draft** sheet with incomplete weightage (70% / 100%).

**What to demo:**
- Dashboard shows "draft" status
- Open sheet → weightage bar is yellow (70%)
- Add the 4th goal to reach 100%
- Validation: can't submit until weightage = 100%
- Edit/delete goals

### 4. Admin journey — David Park (`admin@demo.com`)

**What to demo:**
- Dashboard with user/cycle/department counts
- Manage goal cycles (create new, activate)
- Manage departments and thrust areas
- Users table — view all 5 users
- Audit log — see Maria's sheet lock event
- Unlock Maria's locked sheet

---

## Database Scripts

| Script | Purpose |
|---|---|
| `users_insert_script.py` | Seeds 3 core users (admin, manager, employee), department, thrust areas, goal cycle |
| `mock_data_script.py` | Seeds 2 more employees, goal sheets in 3 workflow states, goals, Q1 achievements, check-in comments, audit log |

Both scripts are idempotent — safe to re-run.

---

## Data Flow

```
Employee creates goal sheet → adds goals → submits for approval
                                                    ↓
Manager reviews → approves (locks) or returns with reason
                                                    ↓
Employee logs Q1-Q4 actuals → score computed automatically
                                                    ↓
Manager conducts check-ins → adds comments per quarter
                                                    ↓
Admin manages cycles, departments, thrust areas, audit trail
```

## Score Computation

| UoM Type | Formula |
|---|---|
| `min_numeric` / `min_percent` | `min(actual/target, 1.0) × 100` |
| `max_numeric` / `max_percent` | `min(target/actual, 1.0) × 100` |
| `timeline` | 100% if on/before deadline, else 0% |
| `zero` | 100% if actual = 0, else 0% |

Weighted goal score = `score × (weightage / 100)`. Sheet total = sum of weighted scores.
