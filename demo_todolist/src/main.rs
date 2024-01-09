use axum:: {
    extract:: {Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing:: {get,post},
    Router,
}
use bb8::Pool;
use bb8_postgres::PostgreyConnectionManager;
use serde::{Deserialize, Serialize};
use tokio_postgrey::NoTls;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost user=postgres dbname=todolist password=123456",NoTls,)
        .unwrap();

    let pool = Pool::builder().build(manager).await.unwrap();
    
    let app = Router::new()
        .route("/todos", get(todo_index))
        .route("/todo/new", post(todo_create))
        // .route("/todo/update", post(todo_update))
        // .route("/todo/delelte/:id", post(todo_delete))
        .layer(TraceLayer::new_for_http())
        .fallback(handler_404)
        .with_state(pool);  

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing:: debug!("listening on {}",listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: String,
    description: String,
    completed: bool
}

#[derive(Debug, Serialize, Clone)]
pub struct Todo {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

async fn todos_index(pagination: Option<Query<pagination>>, State(pool): State<ConnectionPool>)
    -> Result<Json<Vec<Todo>>, (StatusCode, String)>{

    let conn = pool.get().await.map_err(internal_error)?;
    let Query(pagination) = pagination.unwrap_or_default();
    let offset: i64 = pagination.offset.unwrap_or(0);
    let limit: i64 = pagination.limit.unwrap_or(100);
    
    let rows = conn.query(
        "select id, description, completed from todo offset $1 limit $2", &[&offset, &limit],
    )
    .await.map_err(internal_error)?;

    let mut todos: Vec<Todo> = Vec::new();
    for rows in rows {
        let id = row.get(0);
        let description = row.get(1);
        let compeleted = row.get(2);
        let todo = Todo {
            id,description,compeleted,
        }
        todos.push(todo);
    }
    Ok(Json(todos))
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    description: String,
}

async fn todo_create(State(pool): State<ConnectionPool>, Json(input): Json<CreateTodo>)
    -> Result<(StatusCode,Json<Todo>),(StatusCode, String)> {

    let todo = Todo {
        id: Uuid::new_v4().simple().to_string(),
        description: input.description,
        compeleted: false,
    }    

    let conn = pool.get().await.map_err(internal_error)?;

    let _ret = conn.execute(
        "insert into todo (id, description, completed) values ($1, $2, $3) returning id",
            &[&todo.id, &todo.description, &todo.completed],
    ).await.map_err(internal_error)?;

    Ok(StatusCode::CREATED, Json(todo))
}