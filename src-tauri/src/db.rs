use rusqlite::{Connection, DatabaseName};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Db {
    pub conn: std::sync::Arc<Mutex<Connection>>,
    path: PathBuf,
}

// Penanda file terenkripsi. File lama (sqlite polos) tetap bisa dimigrasi otomatis.
const MAGIC: &[u8; 8] = b"TMDB\x00\x01\x00\x00";
const KEY: &[u8] = b"TaskMini-pointer-key";

// ponytail: obfuscate XOR ber-stream, bukan kriptografi sejati. Kalau butuh AEAD asli,
// ganti ke SQLCipher (rusqlite feature "bundled-sqlcipher") + Password.
fn stream() -> u32 {
    let mut s: u32 = 0x9E37_79B9;
    for &k in KEY {
        s = (s ^ k as u32).wrapping_mul(0x0100_0193);
    }
    s
}

fn xor(data: &[u8]) -> Vec<u8> {
    let mut s = stream();
    data.iter()
        .map(|&b| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            b ^ (s as u8)
        })
        .collect()
}

fn encrypt(data: &[u8]) -> Vec<u8> {
    let body = xor(data);
    let mut out = Vec::with_capacity(MAGIC.len() + body.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&body);
    out
}

fn decrypt(raw: &[u8]) -> Option<Vec<u8>> {
    if raw.len() < MAGIC.len() || &raw[..MAGIC.len()] != MAGIC {
        return None;
    }
    Some(xor(&raw[MAGIC.len()..]))
}

// Muat byte sqlite (polos/terdekripsi) ke koneksi in-memory.
fn deserialize_bytes(conn: &mut Connection, bytes: Vec<u8>) -> bool {
    let n = bytes.len();
    if n == 0 {
        return false;
    }
    let ptr = unsafe { rusqlite::ffi::sqlite3_malloc64(n as u64) } as *mut u8;
    if ptr.is_null() {
        return false;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, n) };
    let owned = unsafe {
        rusqlite::serialize::OwnedData::from_raw_nonnull(std::ptr::NonNull::new_unchecked(ptr), n)
    };
    conn.deserialize(DatabaseName::Main, owned, false).is_ok()
}

// Simpan snapshot DB terenkripsi (tulis-tmp lalu rename, hindari file rusak saat crash).
fn save(c: &Connection, path: &Path) {
    let data = match c.serialize(DatabaseName::Main) {
        Ok(d) => d,
        Err(_) => return,
    };
    let enc = encrypt(&data);
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, &enc).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

#[derive(Serialize)]
pub struct Task {
    pub id: i64,
    pub text: String,
    pub done: bool,
    pub deleted: bool,
    pub pos: i64,
    pub category_id: i64,
    pub project_id: i64,
}

#[derive(Serialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub pos: i64,
}

#[derive(Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    pin_hash: Option<String>,
}

const CONFIG_FILE: &str = "data.cfg";
const OLD_CONFIG_FILE: &str = "config.json";

fn config_path() -> PathBuf {
    db_dir().join(CONFIG_FILE)
}

fn load_config() -> AppConfig {
    let p = config_path();
    let old_p = db_dir().join(OLD_CONFIG_FILE);

    if let Ok(raw) = std::fs::read(&p) {
        let bytes = decrypt(&raw).unwrap_or(raw);
        if let Ok(s) = String::from_utf8(bytes) {
            if let Ok(c) = serde_json::from_str(&s) {
                return c;
            }
        }
    } else if let Ok(raw) = std::fs::read_to_string(&old_p) {
        if let Ok(c) = serde_json::from_str(&raw) {
            return c;
        }
    }
    AppConfig { pin_hash: None }
}

fn save_config(c: &AppConfig) {
    let p = config_path();
    if let Ok(s) = serde_json::to_string(c) {
        let enc = encrypt(s.as_bytes());
        let _ = std::fs::write(&p, enc);
    }
    let old_p = db_dir().join(OLD_CONFIG_FILE);
    if old_p != p && old_p.exists() {
        let _ = std::fs::remove_file(old_p);
    }
}

fn hash_pin(pin: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(pin.as_bytes());
    h.update(b"|TaskMini-pin-v1");
    format!("{:x}", h.finalize())
}

#[tauri::command]
pub fn get_lock_status() -> R<serde_json::Value> {
    let cfg = load_config();
    Ok(serde_json::json!({ "has_pin": cfg.pin_hash.is_some() }))
}

#[tauri::command]
pub fn set_pin(pin: String) -> R<()> {
    let p = pin.trim().to_string();
    if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN harus berupa angka".into());
    }
    if p.len() < 4 || p.len() > 6 {
        return Err("PIN harus 4-6 digit".into());
    }
    let mut cfg = load_config();
    cfg.pin_hash = Some(hash_pin(&p));
    save_config(&cfg);
    Ok(())
}

#[tauri::command]
pub fn verify_pin(pin: String) -> R<bool> {
    let cfg = load_config();
    match cfg.pin_hash {
        None => Ok(true),
        Some(h) => Ok(hash_pin(&pin) == h),
    }
}

#[tauri::command]
pub fn clear_pin() -> R<()> {
    let mut cfg = load_config();
    cfg.pin_hash = None;
    save_config(&cfg);
    Ok(())
}

const PALETTE: [&str; 8] = [
    "red", "orange", "amber", "green", "cyan", "blue", "violet", "pink",
];

fn default_color(id: i64) -> &'static str {
    PALETTE[(id.rem_euclid(8)) as usize]
}

fn is_tone(s: &str) -> bool {
    PALETTE.contains(&s)
}

type R<T> = Result<T, String>;

fn now() -> String {
    // ponytail: waktu UTC, cukup untuk urutan. Ganti ke chrono/local kalau perlu tampil ke user.
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    t.to_string()
}

fn db_dir() -> PathBuf {
    // ponytail: DB di sebelah exe (portable). Pindah ke app_data_dir kalau perlu multi-user.
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

const DB_FILE: &str = "data.bin";
const OLD_DB_FILE: &str = "tasks.db";

impl Db {
    pub fn new() -> Self {
        let db_dir = db_dir();
        let path = db_dir.join(DB_FILE);
        let old_path = db_dir.join(OLD_DB_FILE);
        let mut conn = Connection::open_in_memory().expect("buka DB in-memory");

        // muat: coba data.bin dulu; kalau belum ada, muat dari tasks.db lama
        let mut migrated = false;
        if let Ok(raw) = std::fs::read(&path) {
            let bytes = decrypt(&raw).unwrap_or(raw);
            deserialize_bytes(&mut conn, bytes);
        } else if let Ok(raw) = std::fs::read(&old_path) {
            let bytes = decrypt(&raw).unwrap_or(raw);
            deserialize_bytes(&mut conn, bytes);
            migrated = true;
        }

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                done INTEGER NOT NULL DEFAULT 0,
                deleted INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                done_at TEXT,
                deleted_at TEXT,
                pos INTEGER NOT NULL DEFAULT 0
            );",
        )
        .expect("buat tabel tasks");

        // migrasi dari skema lama tanpa kolom pos
        if conn
            .execute("ALTER TABLE tasks ADD COLUMN pos INTEGER NOT NULL DEFAULT 0", [])
            .is_ok()
        {
            conn.execute("UPDATE tasks SET pos = id", []).ok();
        }

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL COLLATE NOCASE UNIQUE,
                description TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO projects(name)
            SELECT 'Project Saya'
            WHERE NOT EXISTS (SELECT 1 FROM projects);",
        )
        .expect("buat tabel projects");
        conn.execute(
            "ALTER TABLE projects ADD COLUMN description TEXT NOT NULL DEFAULT ''",
            [],
        )
        .ok();
        let default_project: i64 = conn
            .query_row("SELECT id FROM projects ORDER BY id LIMIT 1", [], |r| r.get(0))
            .expect("project awal");

        let categories_exist: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='categories')",
                [],
                |r| r.get(0),
            )
            .expect("periksa tabel categories");
        if !categories_exist {
            conn.execute_batch(
                "CREATE TABLE categories(
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    color TEXT,
                    project_id INTEGER NOT NULL,
                    pos INTEGER NOT NULL DEFAULT 0,
                    UNIQUE(project_id, name)
                );",
            )
            .expect("buat tabel categories");
        } else {
            let has_color: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM pragma_table_info('categories') WHERE name='color')",
                    [],
                    |r| r.get(0),
                )
                .expect("periksa kolom color");
            if !has_color {
                conn.execute("ALTER TABLE categories ADD COLUMN color TEXT", [])
                    .expect("tambah kolom warna kategori");
            }
            let has_project_id: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM pragma_table_info('categories') WHERE name='project_id')",
                    [],
                    |r| r.get(0),
                )
                .expect("periksa kolom categories");
            if !has_project_id {
                conn.execute_batch(&format!(
                    "ALTER TABLE categories RENAME TO categories_legacy;
                    CREATE TABLE categories(
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        color TEXT,
                        project_id INTEGER NOT NULL,
                        pos INTEGER NOT NULL DEFAULT 0,
                        UNIQUE(project_id, name)
                    );
                    INSERT INTO categories(id, name, color, project_id, pos)
                    SELECT id, name, color, {default_project}, id * 10 FROM categories_legacy;
                    DROP TABLE categories_legacy;"
                ))
                .expect("migrasi categories ke project");
            }
            let has_pos: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM pragma_table_info('categories') WHERE name='pos')",
                    [],
                    |r| r.get(0),
                )
                .expect("periksa kolom pos categories");
            if !has_pos {
                conn.execute("ALTER TABLE categories ADD COLUMN pos INTEGER NOT NULL DEFAULT 0", [])
                    .expect("tambah kolom pos");
                conn.execute("UPDATE categories SET pos = id * 10", []).ok();
            }
        }
        conn.execute_batch(
            "CREATE UNIQUE INDEX IF NOT EXISTS categories_project_name
             ON categories(project_id, name COLLATE NOCASE);",
        )
        .expect("buat indeks kategori");

        conn.execute(
            "ALTER TABLE tasks ADD COLUMN category_id INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .ok();
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN project_id INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .ok();
        conn.execute("UPDATE tasks SET project_id=? WHERE project_id=0", [default_project])
            .ok();
        let umum: i64 = match conn.query_row(
            "SELECT id FROM categories WHERE project_id=? ORDER BY id LIMIT 1",
            [default_project],
            |r| r.get(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                conn.execute(
                    "INSERT INTO categories(name, project_id) VALUES('Umum', ?)",
                    [default_project],
                )
                .expect("kategori awal");
                conn.last_insert_rowid()
            }
        };
        conn.execute(
            "UPDATE tasks SET category_id=? WHERE category_id=0",
            [umum],
        )
        .ok();

        // konversi hex lama (kalau ada) ke nama warna tailwind
        conn.execute(
            "UPDATE categories SET color = CASE color
                WHEN '#ef4444' THEN 'red' WHEN '#f97316' THEN 'orange'
                WHEN '#eab308' THEN 'amber' WHEN '#22c55e' THEN 'green'
                WHEN '#06b6d4' THEN 'cyan' WHEN '#3b82f6' THEN 'blue'
                WHEN '#8b5cf6' THEN 'violet' WHEN '#ec4899' THEN 'pink'
                ELSE color END
             WHERE color LIKE '#%'",
            [],
        )
        .ok();

        // isi warna hanya untuk baris yang belum punya (record lain tak diubah)
        conn.execute(
            "UPDATE categories SET color = CASE id % 8
                WHEN 0 THEN 'red' WHEN 1 THEN 'orange' WHEN 2 THEN 'amber' WHEN 3 THEN 'green'
                WHEN 4 THEN 'cyan' WHEN 5 THEN 'blue' WHEN 6 THEN 'violet' ELSE 'pink' END
             WHERE color IS NULL OR color = ''",
            [],
        )
        .ok();

        save(&conn, &path);

        // Jika baru bermigrasi dari tasks.db lama, bersihkan file tasks.db lama dan residunya
        if migrated && old_path.exists() {
            let _ = std::fs::remove_file(&old_path);
            let _ = std::fs::remove_file(db_dir.join("tasks.db-journal"));
            let _ = std::fs::remove_file(db_dir.join("tasks.db-wal"));
            let _ = std::fs::remove_file(db_dir.join("tasks.db-shm"));
            let _ = std::fs::remove_file(db_dir.join("tasks.db.tmp"));
        }

        Db {
            conn: std::sync::Arc::new(Mutex::new(conn)),
            path,
        }
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

fn conn(db: &Db) -> std::sync::MutexGuard<'_, Connection> {
    db.conn.lock().unwrap_or_else(|e| e.into_inner())
}

fn top_pos(c: &Connection, done: bool, category: i64) -> i64 {
    c.query_row(
        "SELECT COALESCE(MIN(pos), 1) - 1 FROM tasks
         WHERE done=? AND deleted=0 AND category_id=?",
        rusqlite::params![done as i64, category],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

fn task_cat(c: &Connection, id: i64) -> i64 {
    c.query_row("SELECT category_id FROM tasks WHERE id=?", [id], |r| r.get(0))
        .unwrap_or(0)
}

#[tauri::command]
pub fn list_tasks(db: tauri::State<Db>, project_id: i64) -> R<Vec<Task>> {
    let c = conn(&db);
    let mut st = c
        .prepare(
            "SELECT id, text, done, deleted, pos, category_id, project_id FROM tasks
             WHERE project_id=? ORDER BY pos ASC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = st
        .query_map([project_id], |r| {
            Ok(Task {
                id: r.get(0)?,
                text: r.get(1)?,
                done: r.get::<_, i64>(2)? != 0,
                deleted: r.get::<_, i64>(3)? != 0,
                pos: r.get(4)?,
                category_id: r.get(5)?,
                project_id: r.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_categories(db: tauri::State<Db>, project_id: i64) -> R<Vec<Category>> {
    let c = conn(&db);
    let mut st = c
        .prepare(
            "SELECT id, name, COALESCE(color, ''), pos FROM categories
             WHERE project_id=? ORDER BY pos ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = st
        .query_map([project_id], |r| {
            Ok(Category {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                pos: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_projects(db: tauri::State<Db>) -> R<Vec<Project>> {
    let c = conn(&db);
    let mut st = c
        .prepare("SELECT id, name, description FROM projects ORDER BY name COLLATE NOCASE")
        .map_err(|e| e.to_string())?;
    let rows = st
        .query_map([], |r| {
            Ok(Project {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_project(db: tauri::State<Db>, name: String, description: String) -> R<i64> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Nama project kosong".into());
    }
    let description = description.trim();
    let mut c = conn(&db);
    let tx = c.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO projects(name, description) VALUES(?, ?)",
        rusqlite::params![name, description],
    )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                "Nama project sudah dipakai".to_string()
            } else {
                e.to_string()
            }
        })?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO categories(name, color, project_id) VALUES('Umum', ?, ?)",
        rusqlite::params![default_color(id), id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(id)
}

#[tauri::command]
pub fn update_project(
    db: tauri::State<Db>,
    id: i64,
    name: String,
    description: String,
) -> R<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Nama project kosong".into());
    }
    let description = description.trim();
    let c = conn(&db);
    c.execute(
        "UPDATE projects SET name=?, description=? WHERE id=?",
        rusqlite::params![name, description, id],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            "Nama project sudah dipakai".to_string()
        } else {
            e.to_string()
        }
    })?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn delete_project(db: tauri::State<Db>, id: i64) -> R<()> {
    let c = conn(&db);
    let total: i64 = c
        .query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if total <= 1 {
        return Err("Minimal harus ada satu project".into());
    }
    let n: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE project_id=?",
            [id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if n > 0 {
        return Err(format!("Project ini masih memiliki {n} todo."));
    }
    c.execute("DELETE FROM categories WHERE project_id=?", [id])
        .map_err(|e| e.to_string())?;
    c.execute("DELETE FROM projects WHERE id=?", [id])
        .map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn add_category(
    db: tauri::State<Db>,
    name: String,
    color: String,
    project_id: i64,
) -> R<i64> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Nama kategori kosong".into());
    }
    let color = color.trim();
    let color = if is_tone(color) { Some(color) } else { None };
    let c = conn(&db);
    if let Ok(id) = c.query_row(
        "SELECT id FROM categories WHERE name=? COLLATE NOCASE AND project_id=?",
        rusqlite::params![name, project_id],
        |r| r.get::<_, i64>(0),
    ) {
        return Ok(id);
    }
    let next_pos: i64 = c
        .query_row(
            "SELECT COALESCE(MAX(pos), -10) + 10 FROM categories WHERE project_id=?",
            [project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    c.execute(
        "INSERT INTO categories(name, color, project_id, pos) VALUES(?, ?, ?, ?)",
        rusqlite::params![name, color, project_id, next_pos],
    )
    .map_err(|e| e.to_string())?;
    let id = c.last_insert_rowid();
    if color.is_none() {
        c.execute(
            "UPDATE categories SET color=? WHERE id=?",
            rusqlite::params![default_color(id), id],
        )
        .ok();
    }
    save(&c, &db.path);
    Ok(id)
}

#[tauri::command]
pub fn set_category_color(db: tauri::State<Db>, id: i64, color: String) -> R<()> {
    let color = color.trim();
    if !is_tone(color) {
        return Err("Warna tidak valid".into());
    }
    conn(&db)
        .execute(
            "UPDATE categories SET color=? WHERE id=?",
            rusqlite::params![color, id],
        )
        .map_err(|e| e.to_string())?;
    save(&conn(&db), &db.path);
    Ok(())
}

#[tauri::command]
pub fn rename_category(
    db: tauri::State<Db>,
    id: i64,
    name: String,
    project_id: i64,
) -> R<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Nama kategori kosong".into());
    }
    let c = conn(&db);
    if c.query_row(
        "SELECT id FROM categories WHERE name=? COLLATE NOCASE AND id<>? AND project_id=?",
        rusqlite::params![name, id, project_id],
        |r| r.get::<_, i64>(0),
    )
    .is_ok()
    {
        return Err("Nama kategori sudah dipakai".into());
    }
    c.execute(
        "UPDATE categories SET name=? WHERE id=? AND project_id=?",
        rusqlite::params![name, id, project_id],
    )
    .map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn delete_category(db: tauri::State<Db>, id: i64, project_id: i64) -> R<()> {
    let c = conn(&db);
    let total: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM categories WHERE project_id=?",
            [project_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if total <= 1 {
        return Err("Minimal harus ada satu kategori".into());
    }
    let n: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE category_id=? AND project_id=?",
            rusqlite::params![id, project_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if n > 0 {
        return Err(format!(
            "Masih ada {n} tugas di kategori ini. Pindahkan/hapus dulu."
        ));
    }
    c.execute(
        "DELETE FROM categories WHERE id=? AND project_id=?",
        rusqlite::params![id, project_id],
    )
        .map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn reorder_categories(db: tauri::State<Db>, ids: Vec<i64>, project_id: i64) -> R<()> {
    let mut c = conn(&db);
    let tx = c.transaction().map_err(|e| e.to_string())?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE categories SET pos=? WHERE id=? AND project_id=?",
            rusqlite::params![(i as i64) * 10, id, project_id],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn add_task(
    db: tauri::State<Db>,
    text: String,
    category_id: i64,
    project_id: i64,
) -> R<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let c = conn(&db);
    let category_project: i64 = c
        .query_row(
            "SELECT project_id FROM categories WHERE id=?",
            [category_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if category_project != project_id {
        return Err("Kategori bukan bagian dari project aktif".into());
    }
    let pos = top_pos(&c, false, category_id);
    c.execute(
        "INSERT INTO tasks(text, done, deleted, created_at, pos, category_id, project_id)
         VALUES(?,0,0,?,?,?,?)",
        rusqlite::params![text, now(), pos, category_id, project_id],
    )
    .map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn update_task(
    db: tauri::State<Db>,
    id: i64,
    text: String,
    category_id: i64,
    project_id: i64,
) -> R<()> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Teks tugas kosong".into());
    }
    let c = conn(&db);
    let category_project: i64 = c
        .query_row(
            "SELECT project_id FROM categories WHERE id=?",
            [category_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if category_project != project_id {
        return Err("Kategori bukan bagian dari project aktif".into());
    }
    let old_cat = task_cat(&c, id);
    let pos = if old_cat == category_id {
        c.query_row("SELECT pos FROM tasks WHERE id=?", [id], |r| r.get(0))
            .unwrap_or(0)
    } else {
        top_pos(&c, false, category_id)
    };
    let changed = c.execute(
        "UPDATE tasks SET text=?, category_id=?, pos=? WHERE id=? AND project_id=?",
        rusqlite::params![text, category_id, pos, id, project_id],
    )
    .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Todo tidak ditemukan di project aktif".into());
    }
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn set_done(db: tauri::State<Db>, id: i64, done: bool) -> R<()> {
    let c = conn(&db);
    let pos = top_pos(&c, done, task_cat(&c, id));
    c.execute(
        "UPDATE tasks SET done=?, done_at=?, pos=? WHERE id=? AND deleted=0",
        rusqlite::params![done as i64, now(), pos, id],
    )
    .map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn soft_delete(db: tauri::State<Db>, id: i64) -> R<()> {
    let c = conn(&db);
    c.execute(
        "UPDATE tasks SET deleted=1, deleted_at=? WHERE id=? AND deleted=0",
        rusqlite::params![now(), id],
    )
    .map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn restore(db: tauri::State<Db>, id: i64) -> R<()> {
    let c = conn(&db);
    let (done, cat): (i64, i64) = c
        .query_row(
            "SELECT done, category_id FROM tasks WHERE id=? AND deleted=1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    let pos = top_pos(&c, done != 0, cat);
    c.execute(
        "UPDATE tasks SET deleted=0, deleted_at=NULL, pos=? WHERE id=? AND deleted=1",
        rusqlite::params![pos, id],
    )
    .map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn hard_delete(db: tauri::State<Db>, id: i64) -> R<()> {
    let c = conn(&db);
    c.execute("DELETE FROM tasks WHERE id=? AND deleted=1", [id])
        .map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn soft_delete_batch(db: tauri::State<Db>, ids: Vec<i64>) -> R<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut c = conn(&db);
    let tx = c.transaction().map_err(|e| e.to_string())?;
    let t = now();
    for id in ids {
        tx.execute(
            "UPDATE tasks SET deleted=1, deleted_at=? WHERE id=? AND deleted=0",
            rusqlite::params![t, id],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn restore_batch(db: tauri::State<Db>, ids: Vec<i64>) -> R<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut c = conn(&db);
    let tx = c.transaction().map_err(|e| e.to_string())?;
    for id in ids {
        if let Ok((done, cat)) = tx.query_row(
            "SELECT done, category_id FROM tasks WHERE id=? AND deleted=1",
            [id],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        ) {
            let pos = top_pos(&tx, done != 0, cat);
            tx.execute(
                "UPDATE tasks SET deleted=0, deleted_at=NULL, pos=? WHERE id=? AND deleted=1",
                rusqlite::params![pos, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn hard_delete_batch(db: tauri::State<Db>, ids: Vec<i64>) -> R<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut c = conn(&db);
    let tx = c.transaction().map_err(|e| e.to_string())?;
    for id in ids {
        tx.execute("DELETE FROM tasks WHERE id=? AND deleted=1", [id])
            .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn reorder(db: tauri::State<Db>, ids: Vec<i64>) -> R<()> {
    let mut c = conn(&db);
    let tx = c.transaction().map_err(|e| e.to_string())?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE tasks SET pos=? WHERE id=?",
            rusqlite::params![(i as i64) * 10, id],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    save(&c, &db.path);
    Ok(())
}

#[tauri::command]
pub fn export_csv(db: tauri::State<Db>, path: String) -> R<String> {
    if Path::new(&path).exists() {
        return Err("File sudah ada. Gunakan nama file lain agar file lama tidak tertimpa.".into());
    }
    let c = conn(&db);
    let mut st = c
        .prepare(
            "SELECT t.id, t.text, COALESCE(p.name,''), COALESCE(cat.name,'') AS category, t.done, t.deleted,
                    t.created_at, COALESCE(t.done_at,''), COALESCE(t.deleted_at,'')
             FROM tasks t
             LEFT JOIN projects p ON p.id=t.project_id
             LEFT JOIN categories cat ON cat.id = t.category_id
             ORDER BY p.name COLLATE NOCASE, t.pos ASC, t.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let mut out = String::from("id,text,project,category,done,deleted,created_at,done_at,deleted_at\r\n");
    let rows = st
        .query_map([], |r| {
            let id: i64 = r.get(0)?;
            let text: String = r.get(1)?;
            let project: String = r.get(2)?;
            let cat: String = r.get(3)?;
            let done: i64 = r.get(4)?;
            let deleted: i64 = r.get(5)?;
            let created: String = r.get(6)?;
            let done_at: String = r.get(7)?;
            let deleted_at: String = r.get(8)?;
            Ok((id, text, project, cat, done, deleted, created, done_at, deleted_at))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (id, text, project, cat, done, deleted, created, done_at, deleted_at) =
            row.map_err(|e| e.to_string())?;
        out.push_str(&format!(
            "{id},{},{},{},{},{},{},{},{}\r\n",
            csv_field(&text),
            csv_field(&project),
            csv_field(&cat),
            done != 0,
            deleted != 0,
            csv_field(&created),
            csv_field(&done_at),
            csv_field(&deleted_at),
        ));
    }

    write_csv(&path, &out)?;
    Ok(path)
}

#[tauri::command]
pub fn write_csv_template(path: String) -> R<String> {
    write_csv(
        &path,
        "id,text,project,category,done,deleted,created_at,done_at,deleted_at\r\n",
    )?;
    Ok(path)
}

#[tauri::command]
pub fn default_export_dir() -> String {
    std::env::var_os("USERPROFILE")
        .map(|p| PathBuf::from(p).join("Downloads"))
        .filter(|d| d.is_dir())
        .unwrap_or_else(db_dir)
        .to_string_lossy()
        .into_owned()
}

fn write_csv(path: &str, contents: &str) -> R<()> {
    let path = Path::new(path);
    if !path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
    {
        return Err("File harus menggunakan ekstensi .csv".into());
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("gagal membuat folder: {e}"))?;
    }
    std::fs::write(path, contents.as_bytes()).map_err(|e| format!("gagal tulis CSV: {e}"))
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImportProgress {
    processed: usize,
    total: usize,
}

struct ImportTask {
    text: String,
    project: String,
    category: String,
    done: bool,
    deleted: bool,
    created_at: String,
    done_at: Option<String>,
    deleted_at: Option<String>,
}

#[tauri::command]
pub async fn import_csv(
    app: tauri::AppHandle,
    db: tauri::State<'_, Db>,
    path: String,
) -> R<usize> {
    let db_conn = std::sync::Arc::clone(&db.conn);
    let db_path = db.path.clone();
    tauri::async_runtime::spawn_blocking(move || import_csv_inner(app, db_conn, db_path, path))
        .await
        .map_err(|e| e.to_string())?
}

fn import_csv_inner(
    app: tauri::AppHandle,
    db_conn: std::sync::Arc<Mutex<Connection>>,
    db_path: PathBuf,
    path: String,
) -> R<usize> {
    if !Path::new(&path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
    {
        return Err("File harus menggunakan ekstensi .csv".into());
    }
    let contents =
        std::fs::read_to_string(&path).map_err(|e| format!("gagal membaca CSV: {e}"))?;
    let records = parse_import_csv(&contents)?;
    let total = records.len();
    let mut c = db_conn.lock().unwrap_or_else(|e| e.into_inner());
    let tx = c.transaction().map_err(|e| e.to_string())?;

    for (index, task) in records.iter().enumerate() {
        let project_id: i64 = match tx.query_row(
            "SELECT id FROM projects WHERE name=? COLLATE NOCASE",
            [&task.project],
            |r| r.get(0),
        ) {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                tx.execute(
                    "INSERT INTO projects(name) VALUES(?)",
                    [&task.project],
                )
                .map_err(|e| e.to_string())?;
                tx.last_insert_rowid()
            }
            Err(e) => return Err(e.to_string()),
        };
        let category_id: i64 = match tx.query_row(
            "SELECT id FROM categories WHERE project_id=? AND name=? COLLATE NOCASE",
            rusqlite::params![project_id, task.category],
            |r| r.get(0),
        ) {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                tx.execute(
                    "INSERT INTO categories(name, color, project_id) VALUES(?, ?, ?)",
                    rusqlite::params![task.category, default_color(project_id), project_id],
                )
                .map_err(|e| e.to_string())?;
                tx.last_insert_rowid()
            }
            Err(e) => return Err(e.to_string()),
        };
        let pos: i64 = tx
            .query_row(
                "SELECT COALESCE(MIN(pos), 1) - 1 FROM tasks
                 WHERE done=? AND deleted=0 AND category_id=?",
                rusqlite::params![task.done as i64, category_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO tasks(text, done, deleted, created_at, done_at, deleted_at, pos, category_id, project_id)
             VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                task.text,
                task.done as i64,
                task.deleted as i64,
                task.created_at,
                task.done_at,
                task.deleted_at,
                pos,
                category_id,
                project_id,
            ],
        )
        .map_err(|e| e.to_string())?;
        app.emit(
            "import-progress",
            ImportProgress {
                processed: index + 1,
                total,
            },
        )
        .map_err(|e| format!("gagal mengirim progres import: {e}"))?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    save(&c, &db_path);
    Ok(total)
}

fn parse_import_csv(contents: &str) -> R<Vec<ImportTask>> {
    let rows = parse_csv(contents)?;
    let Some(header) = rows.first() else {
        return Err("CSV tidak valid: header tidak ditemukan".into());
    };
    let expected = [
        "id",
        "text",
        "project",
        "category",
        "done",
        "deleted",
        "created_at",
        "done_at",
        "deleted_at",
    ];
    if header.len() != expected.len()
        || !header
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.trim().eq_ignore_ascii_case(expected))
    {
        return Err(format!(
            "CSV tidak valid. Header harus: {}",
            expected.join(",")
        ));
    }

    rows.iter()
        .enumerate()
        .skip(1)
        .filter(|(_, row)| !(row.len() == 1 && row[0].trim().is_empty()))
        .map(|(index, row)| {
            if row.len() != expected.len() {
                return Err(format!("CSV tidak valid pada baris {}: jumlah kolom salah", index + 1));
            }
            let text = row[1].trim();
            let project = row[2].trim();
            let category = row[3].trim();
            if text.is_empty() || project.is_empty() || category.is_empty() {
                return Err(format!(
                    "CSV tidak valid pada baris {}: todo, project, dan kategori wajib diisi",
                    index + 1
                ));
            }
            let done = parse_csv_bool(&row[4], index + 1, "done")?;
            let deleted = parse_csv_bool(&row[5], index + 1, "deleted")?;
            Ok(ImportTask {
                text: text.to_string(),
                project: project.to_string(),
                category: category.to_string(),
                done,
                deleted,
                created_at: if row[6].trim().is_empty() {
                    now()
                } else {
                    row[6].clone()
                },
                done_at: optional_csv_value(&row[7]),
                deleted_at: optional_csv_value(&row[8]),
            })
        })
        .collect()
}

fn parse_csv_bool(value: &str, line: usize, field: &str) -> R<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(format!(
            "CSV tidak valid pada baris {line}: {field} harus true/false atau 1/0"
        )),
    }
}

fn optional_csv_value(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn parse_csv(contents: &str) -> R<Vec<Vec<String>>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = contents.chars().peekable();
    let mut quoted = false;
    let mut closed_quote = false;

    while let Some(ch) = chars.next() {
        if quoted {
            match ch {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next();
                    field.push('"');
                }
                '"' => {
                    quoted = false;
                    closed_quote = true;
                }
                _ => field.push(ch),
            }
        } else {
            match ch {
                '"' if field.is_empty() && !closed_quote => quoted = true,
                '"' => return Err("CSV tidak valid: tanda kutip di posisi yang salah".into()),
                ',' => {
                    row.push(std::mem::take(&mut field));
                    closed_quote = false;
                }
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    closed_quote = false;
                }
                '\r' if chars.peek() == Some(&'\n') => {
                    chars.next();
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    closed_quote = false;
                }
                '\r' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                    closed_quote = false;
                }
                _ if closed_quote => {
                    return Err("CSV tidak valid: ada karakter setelah tanda kutip penutup".into())
                }
                _ => field.push(ch),
            }
        }
    }
    if quoted {
        return Err("CSV tidak valid: tanda kutip tidak berpasangan".into());
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    Ok(rows)
}

// epoch detik -> YYYYMMDD (UTC). Civil-from-days, tanpa dependency chrono.
#[cfg(test)]
fn ymd(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}{m:02}{d:02}")
}

fn csv_field(v: &str) -> String {
    if v.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enc_roundtrip_and_magic() {
        let msg = b"SQLite format 3\0hello world payload";
        let enc = encrypt(msg);
        assert_eq!(&enc[..MAGIC.len()], MAGIC, "file harus berawalan MAGIC");
        assert_ne!(&enc[MAGIC.len()..], &msg[..], "body harus ter-obfuscate");
        assert_eq!(decrypt(&enc).unwrap(), msg, "dekripsi harus balik utuh");
    }

    #[test]
    fn plain_sqlite_detected_as_not_encrypted() {
        // file sqlite polos (bukan buatan kita) -> decrypt None, dipakai apa adanya
        assert!(decrypt(b"SQLite format 3\0").is_none());
    }

    #[test]
    fn csv_quoting() {
        assert_eq!(csv_field("aman"), "aman");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("kata \"penting\""), "\"kata \"\"penting\"\"\"");
    }

    #[test]
    fn import_csv_validates_and_parses_template_records() {
        let csv = "id,text,project,category,done,deleted,created_at,done_at,deleted_at\r\n\
            ,\"todo, penting\",Project A,Umum,false,false,123,,\r\n";
        let parsed = parse_import_csv(csv).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].text, "todo, penting");
        assert_eq!(parsed[0].project, "Project A");
        assert!(!parsed[0].done);
    }

    #[test]
    fn import_csv_rejects_invalid_headers_and_status_values() {
        assert!(parse_import_csv("text,project,category\n").is_err());
        assert!(parse_import_csv(
            "id,text,project,category,done,deleted,created_at,done_at,deleted_at\n\
            ,Todo,Project A,Umum,maybe,false,123,,\n"
        )
        .is_err());
        assert!(parse_csv("id,text\n1,\"Todo\"suffix\n").is_err());
    }

    #[test]
    fn ymd_epoch() {
        assert_eq!(ymd(0), "19700101");
        assert_eq!(ymd(1_000_000_000), "20010909");
        assert_eq!(ymd(1_729_382_400), "20241020");
    }
}
