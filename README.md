
### Simple Architecture of this project
```mermaid
%%{init: {
  'theme': 'base',
  'themeVariables': {
    'primaryColor': '#000000',
    'primaryTextColor': '#ffffff',
    'primaryBorderColor': '#ffffff',
    'lineColor': '#ffffff',
    'secondaryColor': '#333333',
    'tertiaryColor': '#000000',
    'background': '#000000'
  }
}}%%
flowchart TB
    %% --- STYLE DEFINITIONS ---
    %% Frontend: Pure Black with Thick White Border
    classDef frontend fill:#000000,stroke:#ffffff,stroke-width:2px,color:#ffffff;

    %% Backend: Charcoal Grey with Silver Border
    classDef backend fill:#262626,stroke:#a3a3a3,stroke-width:2px,color:#ffffff;

    %% Database: Black with Double White Border
    classDef database fill:#000000,stroke:#ffffff,stroke-width:4px,color:#ffffff;

    %% External: Dark Grey with Dashed Border (to indicate external/boundary)
    classDef external fill:#171717,stroke:#666666,stroke-width:2px,stroke-dasharray: 5 5,color:#d4d4d4;

    %% Storage: Dim Grey
    classDef storage fill:#1c1917,stroke:#525252,stroke-width:2px,color:#a8a29e;

    %% --- FRONTEND ---
    subgraph Client ["Frontend (React + TypeScript)"]
        direction TB
        F_App["App.tsx Entry"]
        
        subgraph F_Core ["Core & State"]
            direction LR
            F_Ctx["Contexts & Providers<br/>(Auth, Theme, Query)"]
            F_Hooks["Custom Hooks"]
        end

        subgraph F_Pages ["Routing & Pages"]
            direction TB
            P_Public["<b>Public Routes</b><br/>Landing, Leaderboard, Login"]
            P_Secure["<b>Protected Routes</b><br/>Student & Faculty Dashboards"]
        end

        subgraph F_UI ["UI Library"]
            F_Comps["Components<br/>Header, Footer, Tables"]
        end

        %% Frontend Internal Wiring
        F_App --> F_Core
        F_Core --> F_Pages
        F_Pages --> F_UI
    end

    %% --- BACKEND ---
    subgraph Server ["Backend (Rust + Axum)"]
        direction TB
        
        B_Main["Main & Config"]
        
        subgraph B_API ["API Layer"]
            B_Router["<b>Axum Router</b><br/>Auth, Leaderboard, Reports, Stats"]
            B_Middle["Middleware (CORS/Auth)"]
        end

        subgraph B_Services ["Services & Workers"]
            B_Bg["Background Worker<br/>(LeetCode Sync)"]
            B_TUI["TUI Interface"]
        end

        B_Data["<b>Data Access Layer</b><br/>Database Wrapper + SQLx Pool"]

        %% Backend Internal Wiring
        B_Main --> B_Middle --> B_Router
        B_Main --> B_Bg
        B_Main --> B_TUI
        B_Router & B_Bg --> B_Data
    end

    %% --- INFRASTRUCTURE ---
    subgraph Infra ["Infrastructure & External"]
        direction LR
        DB[("MySQL Database")]
        
        subgraph Ext_API ["External APIs"]
            API_LC["LeetCode API"]
            API_Auth["Auth Services"]
        end

        subgraph FileSys ["File System"]
            FS_Logs["Logs, Backups,<br/>Reports, Data"]
        end
    end

    %% --- CROSS-LAYER CONNECTIONS ---
    
    %% 1. Client to Server (The Main Bridge)
    F_Core <==> |"HTTP/REST (JSON)"| B_Middle

    %% 2. Server to DB
    B_Data <==> |"SQL / TCP"| DB

    %% 3. Server to External
    B_Bg -.-> |"Fetch Stats"| API_LC
    B_Router -.-> |"Verify"| API_Auth

    %% 4. Server to Disk
    B_TUI --> |"Read/Write"| FS_Logs

    %% --- CLASS ASSIGNMENT ---
    class F_App,F_Core,F_Ctx,F_Hooks,F_Pages,P_Public,P_Secure,F_UI,F_Comps frontend;
    class B_Main,B_API,B_Router,B_Middle,B_Services,B_Bg,B_TUI,B_Data backend;
    class DB database;
    class Ext_API,API_LC,API_Auth external;
    class FileSys,FS_Logs storage;
```mermaid