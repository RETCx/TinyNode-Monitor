Set WshShell = CreateObject("WScript.Shell")

' ==========================================
' TinyNode Monitor - Stealth Startup Script
' ==========================================
' UPDATE THE PATHS BELOW TO MATCH YOUR SYSTEM.
' DO NOT remove the extra quotes (""..."").

' 1. Run FastAPI Server
' Replace <SERVER_DIR> with the full path to the 'server' folder.
' Replace <UVICORN_PATH> with the full path to uvicorn.exe in your environment.
WshShell.Run "cmd.exe /c ""cd /d <SERVER_DIR> && <UVICORN_PATH> main:app --host 0.0.0.0 --port 8000""", 0, False

' 2. Run Rust Agent
' Replace <AGENT_PATH> with the full path to tinynode_monitor.exe
WshShell.Run """<AGENT_PATH>""", 0, False

' 3. Run Ngrok
' Replace <NGROK_PATH> with the full path to ngrok.exe
WshShell.Run """<NGROK_PATH>"" http 8000", 0, False