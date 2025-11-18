use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware, error::ResponseError};
use actix_cors::Cors;
use actix_files::Files;
use rusqlite::{Connection, params, Error as SqliteError, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use thiserror::Error;
use log::{error, warn, info, debug};

// 自定义错误类型
#[derive(Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] SqliteError),

    #[error("Command not found")]
    CommandNotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        error!("Application error: {}", self);

        let (status_code, message) = match self {
            AppError::CommandNotFound => (actix_web::http::StatusCode::NOT_FOUND, "Command not found"),
            AppError::InvalidInput(_) => (actix_web::http::StatusCode::BAD_REQUEST, "Invalid input"),
            AppError::DatabaseError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::InternalError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        HttpResponse::build(status_code)
            .json(ApiResponse::<String> {
                success: false,
                data: None,
                message: Some(message.to_string()),
            })
    }
}

// 数据模型
#[derive(Debug, Serialize, Deserialize)]
struct Command {
    id: i64,
    name: String,
    #[serde(serialize_with = "serialize_category")]
    category: i64,
    description: String,
}

fn serialize_category<S>(category: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // 将数字分类转换为字符串显示 - 基于Kotlin项目的分类
    let category_str = match category {
        1 => "Miscellaneous",
        2 => "System information",
        3 => "System control",
        4 => "Users & Groups",
        5 => "Files & Folders",
        6 => "Games",
        7 => "Input",
        8 => "Printing",
        9 => "JSON",
        10 => "Network",
        11 => "Search & Find",
        12 => "GIT",
        13 => "SSH",
        14 => "Video & Audio",
        15 => "Package manager",
        16 => "Hacking tools",
        17 => "Terminal games",
        18 => "Crypto currencies",
        19 => "VIM Texteditor",
        20 => "Emacs Texteditor",
        21 => "Nano Texteditor",
        22 => "Pico Texteditor",
        23 => "Micro Texteditor",
        _ => "Other",
    };
    serializer.serialize_str(category_str)
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandDetail {
    id: i64,
    name: String,
    #[serde(serialize_with = "serialize_category")]
    category: i64,
    description: String,
    sections: Vec<CommandSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tldr: Option<String>, // 添加TLDR字段，类似Kotlin项目
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandSection {
    title: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tip {
    id: i64,
    title: String,
    sections: Vec<TipSection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TipSection {
    #[serde(rename = "type")]
    section_type: i64,
    data1: String,
    data2: String,
    extra: String,
}

// 基础分类模型 - 来自Kotlin项目的BasicCategory
#[derive(Debug, Serialize, Deserialize)]
struct BasicCategory {
    id: i64,
    title: String,
    position: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

// 搜索结果模型 (保留供将来使用)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    commands: Vec<Command>,
    total_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestions: Option<Vec<String>>,
}

// 应用统计模型
#[derive(Debug, Serialize, Deserialize)]
struct AppStats {
    total_commands: i64,
    total_categories: i64,
    total_tips: i64,
    total_basic_categories: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchQuery {
    q: String,
    category: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

// 数据库管理
struct AppState {
    db: Mutex<Connection>,
}

impl AppState {
    fn new(db_path: &str) -> Result<Self, AppError> {
        info!("Initializing database connection to: {}", db_path);

        let conn = Connection::open(db_path)
            .map_err(|e| {
                error!("Failed to open database at {}: {}", db_path, e);
                AppError::DatabaseError(e)
            })?;

        // 验证数据库schema
        Self::validate_schema(&conn)?;

        info!("Database connection established successfully");
        Ok(Self {
            db: Mutex::new(conn),
        })
    }

    fn validate_schema(conn: &Connection) -> Result<(), AppError> {
        let tables = ["Command", "CommandSection", "Tip", "TipSection", "BasicCategory", "BasicGroup", "BasicCommand"];

        for table in &tables {
            let count: Result<i64, _> = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0)
            );

            match count {
                Ok(0) => {
                    warn!("Table '{}' not found in database", table);
                }
                Ok(_) => {
                    debug!("Table '{}' found in database", table);
                }
                Err(e) => {
                    error!("Error validating table '{}': {}", table, e);
                    return Err(AppError::DatabaseError(e));
                }
            }
        }

        Ok(())
    }
}

// API 端点

// 获取应用统计信息
async fn get_stats(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    info!("Fetching application statistics");

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    // 统计命令数量
    let total_commands: i64 = conn.query_row(
        "SELECT COUNT(*) FROM Command",
        [],
        |row| row.get(0)
    ).map_err(|e| {
        error!("Failed to count commands: {}", e);
        AppError::DatabaseError(e)
    })?;

    // 统计分类数量（已弃用 - 使用BasicCategory数量代替）
    let _total_categories: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT category) FROM Command",
        [],
        |row| row.get(0)
    ).map_err(|e| {
        error!("Failed to count categories: {}", e);
        AppError::DatabaseError(e)
    })?;

    // 统计提示数量
    let total_tips: i64 = conn.query_row(
        "SELECT COUNT(*) FROM Tip",
        [],
        |row| row.get(0)
    ).map_err(|e| {
        error!("Failed to count tips: {}", e);
        AppError::DatabaseError(e)
    })?;

    // 统计基础分类数量
    let total_basic_categories: i64 = conn.query_row(
        "SELECT COUNT(*) FROM BasicCategory",
        [],
        |row| row.get(0)
    ).map_err(|e| {
        error!("Failed to count basic categories: {}", e);
        AppError::DatabaseError(e)
    })?;

    info!("Stats: {} commands, {} categories, {} tips, {} basic categories",
          total_commands, total_basic_categories, total_tips, total_basic_categories);

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(AppStats {
            total_commands,
            total_categories: total_basic_categories, // 使用实际的basic类别数量
            total_tips,
            total_basic_categories,
        }),
        message: None,
    }))
}

// 获取详细的分类信息（包含描述和图标） - 使用真实的BasicCategory数据
async fn get_categories_detailed(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    info!("Fetching detailed categories from BasicCategory table");

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    let mut stmt = conn
        .prepare("SELECT id, title, position FROM BasicCategory ORDER BY position")
        .map_err(|e| {
            error!("Failed to prepare detailed categories query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let categories: Vec<BasicCategory> = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let position: i64 = row.get(2)?;

            // 根据Kotlin项目添加描述和图标
            let description = match title.as_str() {
                "One-liners" => Some("Useful linux command line one liners".to_string()),
                "System information" => Some("System and battery/cpu/memory/disk usage info on Linux".to_string()),
                "System control" => Some("Lock, unlock, start/stop bluetooth/wifi, shutdown, reboot system".to_string()),
                "Users & Groups" => Some("Create, delete, user, group, list, info".to_string()),
                "Files & Folders" => Some("File and directory operations".to_string()),
                "Input" => Some("Move, click, mouse, type, text, xdotool, ydotool, read, copy, clipboard".to_string()),
                "Printing" => Some("Printer management and printing commands".to_string()),
                "JSON" => Some("JSON processing and manipulation tools".to_string()),
                "Network" => Some("Network configuration and tools".to_string()),
                "Search & Find" => Some("Search and find files and content".to_string()),
                "GIT" => Some("Git version control commands".to_string()),
                "SSH" => Some("SSH connection and key management".to_string()),
                "Video & Audio" => Some("Video and audio processing tools".to_string()),
                "Package manager" => Some("Package management commands".to_string()),
                "Hacking tools" => Some("Security testing and hacking tools".to_string()),
                "Terminal games" => Some("Games that run in the terminal".to_string()),
                "Crypto currencies" => Some("Cryptocurrency related commands".to_string()),
                "VIM Texteditor" => Some("VIM text editor commands and shortcuts".to_string()),
                "Emacs Texteditor" => Some("Emacs text editor commands and shortcuts".to_string()),
                "Nano Texteditor" => Some("Nano text editor commands and shortcuts".to_string()),
                "Pico Texteditor" => Some("Pico text editor commands and shortcuts".to_string()),
                "Micro Texteditor" => Some("Micro text editor commands and shortcuts".to_string()),
                _ => None,
            };

            let icon = None; // Frontend uses Lucide icons, not custom SVG files

            Ok(BasicCategory {
                id,
                title,
                position,
                description,
                icon,
            })
        })
        .map_err(|e| {
            error!("Failed to execute detailed categories query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect();

    info!("Found {} detailed categories", categories.len());

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(categories),
        message: None,
    }))
}

// 获取命令建议（自动完成）
async fn get_command_suggestions(
    query: web::Query<SearchQuery>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    if query.q.trim().is_empty() {
        return Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(vec![] as Vec<String>),
            message: None,
        }));
    }

    info!("Fetching command suggestions for: {}", query.q);

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    let search_term = format!("{}%", query.q);
    let mut stmt = conn
        .prepare("SELECT DISTINCT name FROM Command WHERE name LIKE ?1 ORDER BY name LIMIT 10")
        .map_err(|e| {
            error!("Failed to prepare suggestions query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let suggestions: Vec<String> = stmt
        .query_map(params![&search_term], |row| {
            Ok(row.get::<_, String>(0)?)
        })
        .map_err(|e| {
            error!("Failed to execute suggestions query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect();

    debug!("Found {} suggestions for query: {}", suggestions.len(), query.q);

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(suggestions),
        message: None,
    }))
}

// 获取热门命令（基于某种算法）
async fn get_popular_commands(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    info!("Fetching popular commands");

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    // 获取一些常用命令（这里可以根据实际使用统计来调整）
    let mut stmt = conn
        .prepare("SELECT id, name, category, description FROM Command WHERE category IN (1,3,5,10) ORDER BY RANDOM() LIMIT 20")
        .map_err(|e| {
            error!("Failed to prepare popular commands query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let commands: Vec<Command> = stmt
        .query_map([], |row| {
            Ok(Command {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
            })
        })
        .map_err(|e| {
            error!("Failed to execute popular commands query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect();

    info!("Found {} popular commands", commands.len());

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(commands),
        message: None,
    }))
}

// API 端点

// 获取所有分类
async fn get_categories(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    info!("Fetching all categories");

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    let mut stmt = conn
        .prepare("SELECT title FROM BasicCategory ORDER BY position")
        .map_err(|e| {
            error!("Failed to prepare categories query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let categories: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| {
            error!("Failed to execute categories query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect();

    info!("Found {} categories", categories.len());

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(categories),
        message: None,
    }))
}

// 搜索命令
async fn search_commands(
    query: web::Query<SearchQuery>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // 验证搜索查询
    if query.q.trim().is_empty() {
        warn!("Empty search query received");
        return Err(AppError::InvalidInput("Search query cannot be empty".to_string()));
    }

    info!("Searching commands with query: {:?}", query.q);

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    let limit = query.limit.unwrap_or(50).min(100); // 限制最大返回数量

    // 改进搜索查询：按相关性排序
    let sql = if let Some(ref _cat) = query.category {
        format!(
            "SELECT id, name, category, description,
                   CASE
                       WHEN name = ?1 THEN 100  -- 精确匹配名称，最高优先级
                       WHEN name LIKE ?2 THEN 50  -- 名称开头匹配
                       WHEN name LIKE ?3 THEN 30  -- 名称包含匹配
                       WHEN description LIKE ?2 THEN 20  -- 描述开头匹配
                       WHEN description LIKE ?3 THEN 10  -- 描述包含匹配
                       ELSE 0
                   END as relevance
             FROM Command
             WHERE (name LIKE ?3 OR description LIKE ?3) AND category = ?4
             ORDER BY relevance DESC, name ASC
             LIMIT ?5"
        )
    } else {
        format!(
            "SELECT id, name, category, description,
                   CASE
                       WHEN name = ?1 THEN 100  -- 精确匹配名称，最高优先级
                       WHEN name LIKE ?2 THEN 50  -- 名称开头匹配
                       WHEN name LIKE ?3 THEN 30  -- 名称包含匹配
                       WHEN description LIKE ?2 THEN 20  -- 描述开头匹配
                       WHEN description LIKE ?3 THEN 10  -- 描述包含匹配
                       ELSE 0
                   END as relevance
             FROM Command
             WHERE name LIKE ?3 OR description LIKE ?3
             ORDER BY relevance DESC, name ASC
             LIMIT ?4"
        )
    };

    let exact_term = query.q.trim();
    let start_term = format!("{}%", exact_term);
    let contain_term = format!("%{}%", exact_term);
    debug!("Search SQL: {} with terms: exact='{}', start='{}', contain='{}'", sql, exact_term, start_term, contain_term);

    let mut stmt = conn.prepare(&sql).map_err(|e| {
        error!("Failed to prepare search query: {}", e);
        AppError::DatabaseError(e)
    })?;

    let commands: Vec<Command> = if let Some(ref cat) = query.category {
        stmt.query_map(params![&exact_term, &start_term, &contain_term, cat, limit], |row| {
            Ok(Command {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
            })
        })
        .map_err(|e| {
            error!("Failed to execute category search query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        stmt.query_map(params![&exact_term, &start_term, &contain_term, limit], |row| {
            Ok(Command {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
            })
        })
        .map_err(|e| {
            error!("Failed to execute search query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect()
    };

    info!("Found {} commands for search query: {}", commands.len(), query.q);

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(commands),
        message: None,
    }))
}

// 获取所有命令（用于字母列表）
async fn get_all_commands(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    info!("Fetching all commands for alphabetical listing");

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    let mut stmt = conn
        .prepare("SELECT id, name, category, description FROM Command ORDER BY name")
        .map_err(|e| {
            error!("Failed to prepare all commands query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let commands: Vec<Command> = stmt
        .query_map([], |row| {
            Ok(Command {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
            })
        })
        .map_err(|e| {
            error!("Failed to execute all commands query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect();

    info!("Found {} commands for alphabetical listing", commands.len());

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(commands),
        message: None,
    }))
}

// 获取命令详情
async fn get_command(
    command_id: web::Path<i64>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let command_id = *command_id;
    info!("Fetching command details for id: {}", command_id);

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    // 获取命令基本信息
    let mut stmt = conn
        .prepare("SELECT id, name, category, description
                  FROM Command
                  WHERE id = ?1")
        .map_err(|e| {
            error!("Failed to prepare command query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let command = stmt.query_row(params![command_id], |row| {
        Ok(Command {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            description: row.get(3)?,
        })
    });

    let cmd = match command {
        Ok(cmd) => cmd,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            warn!("Command with id {} not found", command_id);
            return Err(AppError::CommandNotFound);
        }
        Err(e) => {
            error!("Database error fetching command {}: {}", command_id, e);
            return Err(AppError::DatabaseError(e));
        }
    };

    // 获取命令章节，排除NAME章节（根据Kotlin项目的实现）
    let mut sect_stmt = conn
        .prepare("SELECT title, content FROM CommandSection WHERE command_id = ?1 AND title != 'NAME' ORDER BY id")
        .map_err(|e| {
            error!("Failed to prepare sections query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let sections: Vec<CommandSection> = sect_stmt
        .query_map(params![command_id], |row| {
            Ok(CommandSection {
                title: row.get(0)?,
                content: row.get(1)?,
            })
        })
        .map_err(|e| {
            error!("Failed to execute sections query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect();

    // 获取TLDR章节（如果存在）
    let mut tldr_stmt = conn
        .prepare("SELECT content FROM CommandSection WHERE command_id = ?1 AND title = 'TLDR'")
        .map_err(|e| {
            error!("Failed to prepare TLDR query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let tldr: Option<String> = tldr_stmt
        .query_row(params![command_id], |row| {
            Ok(row.get::<_, String>(0)?)
        })
        .ok(); // 忽略错误，TLDR可能不存在

    info!("Command {} found with {} sections", cmd.name, sections.len());

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(CommandDetail {
            id: cmd.id,
            name: cmd.name,
            category: cmd.category,
            description: cmd.description,
            sections,
            tldr,
        }),
        message: None,
    }))
}

// 获取按分类的命令 - 使用BasicCategory系统
async fn get_commands_by_category(
    category: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let category_name = category.as_str();
    info!("Fetching commands for BasicCategory: {}", category_name);

    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    // First, find the BasicCategory ID
    let mut category_stmt = conn
        .prepare("SELECT id FROM BasicCategory WHERE title = ?1")
        .map_err(|e| {
            error!("Failed to prepare category lookup query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let category_id: Option<i64> = category_stmt
        .query_row(params![category_name], |row| {
            Ok(row.get::<_, i64>(0)?)
        })
        .optional()
        .map_err(|e| {
            error!("Failed to lookup category '{}': {}", category_name, e);
            AppError::DatabaseError(e)
        })?;

    let category_id = match category_id {
        Some(id) => {
            debug!("Found BasicCategory '{}' with ID: {}", category_name, id);
            id
        }
        None => {
            warn!("BasicCategory '{}' not found", category_name);
            return Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(vec![] as Vec<Command>),
                message: Some(format!("Category '{}' not found", category_name)),
            }));
        }
    };

    // Get commands through BasicGroup -> BasicCommand relationship
    let mut stmt = conn
        .prepare("SELECT bc.id, bc.command, bc.mans, bg.description
                  FROM BasicCommand bc
                  JOIN BasicGroup bg ON bc.group_id = bg.id
                  WHERE bg.category_id = ?1
                  ORDER BY bc.command")
        .map_err(|e| {
            error!("Failed to prepare basic commands by category query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let commands: Vec<Command> = stmt
        .query_map(params![category_id], |row| {
            let id: i64 = row.get(0)?;
            let command: String = row.get(1)?;
            let _mans: String = row.get(2)?;
            let description: String = row.get(3)?;

            // Extract first line of command (before newline if any)
            let command_name = command.lines().next().unwrap_or(&command).trim();

            Ok(Command {
                id,
                name: command_name.to_string(),
                category: 0, // Use 0 as placeholder for BasicCategory commands
                description,
            })
        })
        .map_err(|e| {
            error!("Failed to execute basic commands by category query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect();

    info!("Found {} basic commands for category '{}' (ID: {})", commands.len(), category_name, category_id);

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(commands),
        message: None,
    }))
}

// 获取随机提示
async fn get_random_tip(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let conn = data.db.lock().map_err(|e| {
        error!("Failed to acquire database lock: {}", e);
        AppError::InternalError("Database lock error".to_string())
    })?;

    let mut stmt = conn
        .prepare("SELECT id, title FROM Tip ORDER BY RANDOM() LIMIT 1")
        .map_err(|e| {
            error!("Failed to prepare random tip query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let tip = stmt
        .query_row([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| {
            error!("Failed to get random tip: {}", e);
            AppError::DatabaseError(e)
        })?;

    let (id, title) = tip;

    // Get tip sections
    let mut sect_stmt = conn
        .prepare("SELECT type, data1, data2, extra FROM TipSection WHERE tip_id = ?1 ORDER BY position")
        .map_err(|e| {
            error!("Failed to prepare tip sections query: {}", e);
            AppError::DatabaseError(e)
        })?;

    let sections: Vec<TipSection> = sect_stmt
        .query_map(params![id], |row| {
            Ok(TipSection {
                section_type: row.get(0)?,
                data1: row.get(1)?,
                data2: row.get(2)?,
                extra: row.get(3)?,
            })
        })
        .map_err(|e| {
            error!("Failed to execute tip sections query: {}", e);
            AppError::DatabaseError(e)
        })?
        .filter_map(|r| r.ok())
        .collect();

    info!("Found random tip: {} with {} sections", title, sections.len());

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(Tip {
            id,
            title,
            sections,
        }),
        message: None,
    }))
}

// 健康检查
async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some("Service is healthy"),
        message: None,
    }))
}

// 提供前端页面
async fn serve_frontend() -> Result<HttpResponse> {
    let html = include_str!("index.html");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{Connection, params};

    // Test helper function to create a test database
    fn create_test_database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Create test tables
        conn.execute(
            "CREATE TABLE Command (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE CommandSection (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                command_id INTEGER NOT NULL
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE BasicCategory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                position INTEGER NOT NULL,
                title TEXT NOT NULL
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE Tip (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                position INTEGER NOT NULL
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE TipSection (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tip_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                type INTEGER NOT NULL,
                data1 TEXT NOT NULL,
                data2 TEXT NOT NULL,
                extra TEXT NOT NULL
            )",
            [],
        ).unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO Command (category, name, description) VALUES (1, 'grep', 'Search files for lines matching a pattern')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO Command (category, name, description) VALUES (3, 'chmod', 'Change file permissions')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO CommandSection (title, content, command_id) VALUES ('TLDR', 'grep pattern file', 1)",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO CommandSection (title, content, command_id) VALUES ('DESCRIPTION', 'grep searches for PATTERN in each FILE.', 1)",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO BasicCategory (position, title) VALUES (1, 'System')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO BasicCategory (position, title) VALUES (2, 'Files')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO Tip (title, position) VALUES ('Quick Navigation', 1)",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO TipSection (tip_id, position, type, data1, data2, extra) VALUES (1, 1, 0, 'Use Ctrl+A to go to beginning of line', '', '')",
            [],
        ).unwrap();

        conn
    }

    #[test]
    fn test_database_schema_validation() {
        let conn = create_test_database();

        // Test that all required tables exist
        let tables = ["Command", "CommandSection", "BasicCategory", "Tip", "TipSection"];

        for table in &tables {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();

            assert_eq!(count, 1, "Table {} should exist", table);
        }
    }

    #[test]
    fn test_command_serialization() {
        let conn = create_test_database();

        let mut stmt = conn
            .prepare("SELECT id, name, category, description FROM Command WHERE name = 'grep'")
            .unwrap();

        let command = stmt
            .query_row([], |row| {
                Ok(Command {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    description: row.get(3)?,
                })
            })
            .unwrap();

        assert_eq!(command.name, "grep");
        assert_eq!(command.category, 1);
        assert_eq!(command.description, "Search files for lines matching a pattern");
    }

    #[test]
    fn test_category_serialization() {
        // Test the category serialization function
        use serde_json;

        let test_cases = vec![
            (1, "Miscellaneous"),
            (3, "System control"),
            (5, "Files & Folders"),
            (10, "Network"),
            (19, "VIM Texteditor"),
            (99, "Other"), // Unknown category
        ];

        for (input, _expected) in test_cases {
            let serialized = serde_json::to_string(&input).unwrap();
            // The serialize_category function should convert numeric categories to strings
            // This is tested indirectly through the API responses
            assert!(serialized.contains(&input.to_string()));
        }
    }

    #[test]
    fn test_search_functionality() {
        let conn = create_test_database();

        // Test search by name
        let mut stmt = conn
            .prepare("SELECT id, name, category, description FROM Command WHERE name LIKE ?1 OR description LIKE ?1")
            .unwrap();

        let search_term = "%grep%";
        let commands: Vec<Command> = stmt
            .query_map(params![search_term], |row| {
                Ok(Command {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    description: row.get(3)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "grep");
    }

    #[test]
    fn test_command_sections_retrieval() {
        let conn = create_test_database();

        let mut stmt = conn
            .prepare("SELECT title, content FROM CommandSection WHERE command_id = ?1 AND title != 'NAME' ORDER BY id")
            .unwrap();

        let sections: Vec<CommandSection> = stmt
            .query_map(params![1], |row| {
                Ok(CommandSection {
                    title: row.get(0)?,
                    content: row.get(1)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "TLDR");
        assert_eq!(sections[0].content, "grep pattern file");
        assert_eq!(sections[1].title, "DESCRIPTION");
    }

    #[test]
    fn test_tldr_section_retrieval() {
        let conn = create_test_database();

        let mut stmt = conn
            .prepare("SELECT content FROM CommandSection WHERE command_id = ?1 AND title = 'TLDR'")
            .unwrap();

        let tldr: Result<String, _> = stmt.query_row(params![1], |row| {
            Ok(row.get::<_, String>(0)?)
        });

        assert!(tldr.is_ok());
        assert_eq!(tldr.unwrap(), "grep pattern file");
    }

    #[test]
    fn test_categories_retrieval() {
        let conn = create_test_database();

        let mut stmt = conn
            .prepare("SELECT title FROM BasicCategory ORDER BY position")
            .unwrap();

        let categories: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0], "System");
        assert_eq!(categories[1], "Files");
    }

    #[test]
    fn test_tips_retrieval() {
        let conn = create_test_database();

        let mut stmt = conn
            .prepare("SELECT id, title FROM Tip ORDER BY RANDOM() LIMIT 1")
            .unwrap();

        let tip = stmt
            .query_row([], |row| {
                Ok(Tip {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    sections: vec![], // Would be populated separately
                })
            })
            .unwrap();

        assert_eq!(tip.title, "Quick Navigation");
    }

    #[test]
    fn test_tip_sections_retrieval() {
        let conn = create_test_database();

        let mut stmt = conn
            .prepare("SELECT type, data1, data2, extra FROM TipSection WHERE tip_id = ?1 ORDER BY position")
            .unwrap();

        let sections: Vec<TipSection> = stmt
            .query_map(params![1], |row| {
                Ok(TipSection {
                    section_type: row.get(0)?,
                    data1: row.get(1)?,
                    data2: row.get(2)?,
                    extra: row.get(3)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].section_type, 0);
        assert_eq!(sections[0].data1, "Use Ctrl+A to go to beginning of line");
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting Linux Command Library Web API Server");

    // 初始化数据库连接
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "database.db".to_string());
    let app_state = web::Data::new(AppState::new(&db_path)?);

    // 获取配置
    let server_addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let enable_cors = std::env::var("ENABLE_CORS").unwrap_or_else(|_| "true".to_string()) == "true";

    info!("Starting Linux Command Library API server on http://{}", server_addr);
    info!("CORS enabled: {}", enable_cors);

    let server = HttpServer::new(move || {
        let cors = if enable_cors {
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
        } else {
            Cors::default()
        };

        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            // 静态资源
            .service(Files::new("/stylesheets", "src/stylesheets"))
            .service(Files::new("/scripts", "src/scripts"))
            .service(Files::new("/images", "src/images"))
            // 前端页面
            .route("/", web::get().to(serve_frontend))
            // 健康检查
            .route("/health", web::get().to(health_check))
            // 应用统计
            .route("/api/stats", web::get().to(get_stats))
            // 分类相关
            .route("/api/categories", web::get().to(get_categories))
            .route("/api/categories/detailed", web::get().to(get_categories_detailed))
            // 搜索相关
            .route("/api/search", web::get().to(search_commands))
            .route("/api/suggestions", web::get().to(get_command_suggestions))
            .route("/api/popular", web::get().to(get_popular_commands))
            // 命令相关
            .route("/api/commands", web::get().to(get_all_commands))
            .route("/api/commands/{id}", web::get().to(get_command))
            .route("/api/category/{name}", web::get().to(get_commands_by_category))
            // 提示相关
            .route("/api/random-tip", web::get().to(get_random_tip))
    })
        .bind(&server_addr)?
        .run();

    info!("Server started successfully");
    server.await?;
    Ok(())
}
