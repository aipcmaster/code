//! AIPCMaster（AI电脑大师）核心引擎 —— 本地存储层
//!
//! 对应《AIPCMaster（AI电脑大师）软件开发工程文档》§5「本地存储」：
//! 诊断原始数据、模型文件、缓存。数据默认本地处理，云端仅存脱敏摘要。
//!
//! 存储内容（对齐 ERD §3 表结构）：
//! - `snapshots`：采集快照（时间序列，本地原始数据）；
//! - `diagnostic_sessions` / `diagnostic_reports` / `issues`：诊断会话与报告；
//! - `audit_logs`：本地审计日志（操作留痕，安全合规）。

mod error;
mod models;

pub use error::{Error, Result};
pub use models::{AuditLog, AuditLogKind, ReportRecord, SnapshotRecord};

use rusqlite::Connection;
use std::path::Path;

/// 本地存储。轻量封装 SQLite。
///
/// 线程安全性：内部用 `Mutex<Connection>`，因为 rusqlite 的 `Connection` 非 `Sync`。
/// 采集线程（写）与 UI/上报线程（读）可并发访问。
#[derive(Clone)]
pub struct Store {
    inner: std::sync::Arc<std::sync::Mutex<Connection>>,
}

impl Store {
    /// 打开数据库，执行迁移。路径不存在时会创建（含父目录）。
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| Error::Io {
                    context: "create store dir",
                    source: e,
                })?;
            }
        }
        let conn = Connection::open(path).map_err(|e| Error::Sqlite {
            context: "open database",
            source: e,
        })?;
        let store = Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    /// 内存数据库（测试用）。
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| Error::Sqlite {
            context: "open in-memory database",
            source: e,
        })?;
        let store = Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.inner.lock().map_err(|_| Error::LockPoisoned)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp_unix_ms INTEGER NOT NULL,
                payload TEXT NOT NULL,
                UNIQUE(timestamp_unix_ms)
            );
            CREATE INDEX IF NOT EXISTS idx_snapshots_time ON snapshots(timestamp_unix_ms);

            CREATE TABLE IF NOT EXISTS diagnostic_sessions (
                id TEXT PRIMARY KEY,
                trigger_type TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at_unix_ms INTEGER NOT NULL,
                ended_at_unix_ms INTEGER
            );

            CREATE TABLE IF NOT EXISTS diagnostic_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL REFERENCES diagnostic_sessions(id),
                health_score INTEGER NOT NULL,
                summary TEXT,
                details_json TEXT,
                generated_at_unix_ms INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES diagnostic_sessions(id)
            );

            CREATE TABLE IF NOT EXISTS audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                action TEXT NOT NULL,
                target TEXT,
                detail TEXT,
                created_at_unix_ms INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_audit_kind ON audit_logs(kind);
            "#,
        )
        .map_err(|e| Error::Sqlite {
            context: "migrate",
            source: e,
        })
    }

    /// 写入一条采集快照（本地原始数据，断网可继续）。
    pub fn insert_snapshot(&self, snap: &aipcmaster_collect::SystemSnapshot) -> Result<()> {
        let payload = serde_json::to_string(snap).map_err(Error::Json)?;
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO snapshots(timestamp_unix_ms, payload) VALUES(?1, ?2)",
            rusqlite::params![snap.timestamp_unix_ms as i64, payload],
        )
        .map_err(|e| Error::Sqlite {
            context: "insert_snapshot",
            source: e,
        })?;
        Ok(())
    }

    /// 读取最近 N 条快照（时间升序）。
    pub fn recent_snapshots(&self, limit: usize) -> Result<Vec<SnapshotRecord>> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT timestamp_unix_ms, payload FROM snapshots \
                 ORDER BY timestamp_unix_ms DESC LIMIT ?1",
            )
            .map_err(|e| Error::Sqlite {
                context: "recent_snapshots prepare",
                source: e,
            })?;
        let rows = stmt
            .query_map(rusqlite::params![limit as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| Error::Sqlite {
                context: "recent_snapshots query",
                source: e,
            })?;
        let mut out = Vec::new();
        for r in rows {
            let (ts, payload) = r.map_err(|e| Error::Sqlite {
                context: "recent_snapshots row",
                source: e,
            })?;
            let snap: aipcmaster_collect::SystemSnapshot =
                serde_json::from_str(&payload).map_err(Error::Json)?;
            out.push(SnapshotRecord {
                timestamp_unix_ms: ts as u64,
                snapshot: snap,
            });
        }
        out.reverse(); // 升序
        Ok(out)
    }

    /// 最近一条快照。
    pub fn latest_snapshot(&self) -> Result<Option<SnapshotRecord>> {
        Ok(self.recent_snapshots(1)?.into_iter().next())
    }

    /// 开始一次诊断会话。
    pub fn start_session(
        &self,
        session_id: &str,
        trigger_type: &str,
        started_at_unix_ms: u64,
    ) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO diagnostic_sessions(id, trigger_type, status, started_at_unix_ms) \
             VALUES(?1, ?2, 'running', ?3)",
            rusqlite::params![session_id, trigger_type, started_at_unix_ms as i64],
        )
        .map_err(|e| Error::Sqlite {
            context: "start_session",
            source: e,
        })
        .map(|_| ())
    }

    /// 结束诊断会话并写入报告摘要。
    pub fn finish_session(
        &self,
        session_id: &str,
        health_score: u8,
        issues: &[aipcmaster_diagnose::Issue],
        summary: &str,
        ended_at_unix_ms: u64,
    ) -> Result<()> {
        let details = serde_json::to_string(issues).map_err(Error::Json)?;
        let conn = self.conn()?;
        conn.execute(
            "UPDATE diagnostic_sessions SET status='completed', ended_at_unix_ms=?1 WHERE id=?2",
            rusqlite::params![ended_at_unix_ms as i64, session_id],
        )
        .map_err(|e| Error::Sqlite {
            context: "finish_session update",
            source: e,
        })?;
        conn.execute(
            "INSERT INTO diagnostic_reports(session_id, health_score, summary, details_json, generated_at_unix_ms) \
             VALUES(?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                session_id,
                health_score as i64,
                summary,
                details,
                ended_at_unix_ms as i64
            ],
        )
        .map_err(|e| Error::Sqlite {
            context: "finish_session insert report",
            source: e,
        })?;
        Ok(())
    }

    /// 查询最近诊断报告。
    pub fn recent_reports(&self, limit: usize) -> Result<Vec<ReportRecord>> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT session_id, health_score, summary, details_json, generated_at_unix_ms \
                 FROM diagnostic_reports ORDER BY generated_at_unix_ms DESC LIMIT ?1",
            )
            .map_err(|e| Error::Sqlite {
                context: "recent_reports prepare",
                source: e,
            })?;
        let rows = stmt
            .query_map(rusqlite::params![limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| Error::Sqlite {
                context: "recent_reports query",
                source: e,
            })?;
        let mut out = Vec::new();
        for r in rows {
            let (session_id, score, summary, details, ts) = r.map_err(|e| Error::Sqlite {
                context: "recent_reports row",
                source: e,
            })?;
            let issues: Vec<aipcmaster_diagnose::Issue> =
                serde_json::from_str(&details).map_err(Error::Json)?;
            out.push(ReportRecord {
                session_id,
                health_score: score as u8,
                summary,
                issues,
                generated_at_unix_ms: ts as u64,
            });
        }
        out.reverse();
        Ok(out)
    }

    /// 写入审计日志（本地留痕；云端同步由上报层完成）。
    pub fn insert_audit(&self, log: &AuditLog) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO audit_logs(kind, action, target, detail, created_at_unix_ms) \
             VALUES(?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                log.kind.as_str(),
                log.action,
                log.target.as_deref(),
                log.detail,
                log.created_at_unix_ms as i64
            ],
        )
        .map_err(|e| Error::Sqlite {
            context: "insert_audit",
            source: e,
        })?;
        Ok(())
    }

    /// 查询审计日志（最近 N 条）。
    pub fn audit_logs(&self, limit: usize) -> Result<Vec<AuditLog>> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT kind, action, target, detail, created_at_unix_ms \
                 FROM audit_logs ORDER BY created_at_unix_ms DESC LIMIT ?1",
            )
            .map_err(|e| Error::Sqlite {
                context: "audit_logs prepare",
                source: e,
            })?;
        let rows = stmt
            .query_map(rusqlite::params![limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| Error::Sqlite {
                context: "audit_logs query",
                source: e,
            })?;
        let mut out = Vec::new();
        for r in rows {
            let (kind, action, target, detail, ts) = r.map_err(|e| Error::Sqlite {
                context: "audit_logs row",
                source: e,
            })?;
            let kind = match kind.as_str() {
                "optimize" => AuditLogKind::Optimize,
                "diagnose" => AuditLogKind::Diagnose,
                "setting" => AuditLogKind::Setting,
                _ => AuditLogKind::Other,
            };
            out.push(AuditLog {
                kind,
                action,
                target,
                detail,
                created_at_unix_ms: ts as u64,
            });
        }
        out.reverse();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipcmaster_collect::SystemSnapshot;

    #[test]
    fn migrations_run() {
        let store = Store::open_in_memory().unwrap();
        // 迁移幂等
        store.migrate().unwrap();
    }

    #[test]
    fn snapshot_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        let snap = SystemSnapshot {
            timestamp_unix_ms: 111,
            cpu: Some(aipcmaster_collect::CpuMetrics {
                usage_percent: 42.0,
                per_core_usage_percent: vec![],
                load_avg_1: 0.0,
                load_avg_5: 0.0,
                load_avg_15: 0.0,
                core_count: 8,
                temperature_c: None,
            }),
            ..Default::default()
        };

        store.insert_snapshot(&snap).unwrap();
        let got = store.latest_snapshot().unwrap().unwrap();
        assert_eq!(got.timestamp_unix_ms, 111);
        assert_eq!(got.snapshot.cpu.unwrap().usage_percent, 42.0);
    }

    #[test]
    fn diagnostic_session_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        let ss = SystemSnapshot {
            timestamp_unix_ms: 222,
            ..Default::default()
        };
        store.insert_snapshot(&ss).unwrap();

        let session = "sess-1";
        store.start_session(session, "manual", 222).unwrap();
        store
            .finish_session(session, 85, &[], "一切正常", 333)
            .unwrap();

        let reports = store.recent_reports(10).unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].health_score, 85);
        assert_eq!(reports[0].session_id, session);
    }

    #[test]
    fn audit_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        store
            .insert_audit(&AuditLog {
                kind: AuditLogKind::Optimize,
                action: "clean_memory".into(),
                target: Some("mem".into()),
                detail: "released 256MB".into(),
                created_at_unix_ms: 1000,
            })
            .unwrap();
        store
            .insert_audit(&AuditLog {
                kind: AuditLogKind::Diagnose,
                action: "run_diagnose".into(),
                target: None,
                detail: "manual".into(),
                created_at_unix_ms: 2000,
            })
            .unwrap();

        let logs = store.audit_logs(10).unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].action, "clean_memory"); // 升序：最旧在前
        assert_eq!(logs[1].kind, AuditLogKind::Diagnose);
    }
}
