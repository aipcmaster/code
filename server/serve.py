#!/usr/bin/env python3
"""启动 server 于独立会话（本地开发模式），立即返回。

安全说明：本地开发显式设置 AIPCMASTER_INSECURE_DEV=1（允许默认 JWT 密钥）与
AIPCMASTER_ALLOW_MOCK_CHECKOUT=1（启用模拟支付）。生产部署切勿设置这两个变量，
并必须提供强随机 AIPCMASTER_JWT_SECRET（openssl rand -hex 32）。
"""
import os
import subprocess
import sys
import time

LOG = "/tmp/opencode/server3.log"
DB = "aipcmaster.db"

# 清理旧进程（精确匹配，避免误杀同名命令行）
subprocess.run(["pkill", "-f", "target/debug/aipcmaster-server"], capture_output=True)
time.sleep(0.5)
for suf in ("", "-wal", "-shm"):
    try:
        os.remove(DB + suf)
    except FileNotFoundError:
        pass

env = dict(os.environ)
env["AIPCMASTER_INSECURE_DEV"] = "1"
env["AIPCMASTER_ALLOW_MOCK_CHECKOUT"] = "1"

logf = open(LOG, "wb")
proc = subprocess.Popen(
    ["target/debug/aipcmaster-server"],
    stdout=logf,
    stderr=logf,
    stdin=subprocess.DEVNULL,
    start_new_session=True,  # setsid
    cwd=os.path.dirname(os.path.abspath(__file__)) or ".",
    env=env,
)
print(f"server pid={proc.pid} -> {LOG}")
sys.exit(0)