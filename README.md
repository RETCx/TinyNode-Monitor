# start_monitor.vbs

A lightweight VBScript to launch all TinyNode Monitor components silently in the background (no console windows).

## Configuration
Before running, open `start_monitor.vbs` in Notepad and update these paths to match your machine:
1. FastAPI Server directory & Anaconda `uvicorn.exe` path.
2. Rust Agent (`tinynode_monitor.exe`) path.
3. Ngrok (`ngrok.exe`) path.

## How to Start
**Double-click** `start_monitor.vbs`. 
*Note: No windows will pop up, but the services will start running immediately.*

## How to Stop
Since it runs in stealth mode, you must close it manually:
1. Open **Task Manager** (`Ctrl + Shift + Esc`).
2. Go to the **Details** tab.
3. Find and **End Task** for:
   - `uvicorn.exe` (or `python.exe`)
   - `tinynode_monitor.exe`
   - `ngrok.exe`