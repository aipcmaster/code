#!/usr/bin/env python3
"""启动 server 于独立会话，立即返回。"""
import os
import signal
import subprocess
import sys
import time

LOG = "/tmp/opencode/server3.log"
DB = "aipcmaster.db"

# 清理旧进程
subprocess.run(["pkill", "-f", "aipcmaster-server"], capture_output=True)
time.sleep(0.5)
for suf in ("", "-wal", "-shm"):
    try:
        os.remove(DB + suf)
    except FileNotFoundError:
        pass

logf = open(LOG, "wb")
proc = subprocess.Popen(
    ["target/debug/aipcmaster-server"],
    stdout=logf,
    stderr=logf,
    stdin=subprocess.DEVNULL,
    start_new_session=True,  # setsid
    cwd=os.path.dirname(os.path.abspath(__file__)) or ".",
)
print(f"server pid={proc.pid} -> {LOG}")
sys.exit(0)