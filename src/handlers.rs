use axum::{
    extract::{Path, Query, Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Form,
};
use sqlx::SqlitePool;
use uuid::Uuid;

// 🌟 NEW: Import LoginForm
use crate::models::{Comment, CommentQuery, CreateCommentForm, LoginForm};

/* --------------------------------------------------------
   PUBLIC API ROUTES (GET & POST COMMENTS)
-------------------------------------------------------- */

pub async fn get_comments(
    State(pool): State<SqlitePool>,
    Query(query): Query<CommentQuery>,
) -> impl IntoResponse {
    let comments: Vec<Comment> = sqlx::query_as!(
        Comment,
        "SELECT id, post_slug, author_name, author_email, content, is_approved, created_at FROM comments WHERE post_slug = ? AND is_approved = 1 ORDER BY created_at ASC",
        query.slug
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    if comments.is_empty() {
        // 🌟 UPDATED to use standard class
        return Html("<p class=\"no-comments\">No comments yet. Be the first to share your thoughts!</p>".to_string());
    }

    let mut html_output = String::new();
    for comment in comments {
        html_output.push_str(&render_comment_item(&comment));
    }

    Html(html_output)
}

pub async fn post_comment(
    State(pool): State<SqlitePool>,
    Form(payload): Form<CreateCommentForm>,
) -> impl IntoResponse {
    if let Some(bot_trap) = payload.honeypot {
        if !bot_trap.trim().is_empty() {
            return (StatusCode::OK, Html(String::new())).into_response();
        }
    }

    let clean_author = ammonia::clean(&payload.author_name);
    let clean_content = ammonia::clean(&payload.content);
    let id = Uuid::new_v4().to_string();

    if clean_content.trim().is_empty() || clean_author.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Html("<p style=\"color: red;\">Author name and comment cannot be empty.</p>".to_string()),
        )
            .into_response();
    }

    let res = sqlx::query!(
        "INSERT INTO comments (id, post_slug, author_name, author_email, content, is_approved) VALUES (?, ?, ?, ?, ?, 0)",
        id,
        payload.post_slug,
        clean_author,
        payload.author_email,
        clean_content
    )
    .execute(&pool)
    .await;

    match res {
        Ok(_) => {
            let created_comment = Comment {
                id,
                post_slug: payload.post_slug,
                author_name: clean_author,
                author_email: payload.author_email,
                content: clean_content,
                is_approved: false,
                created_at: chrono::Utc::now().naive_utc(),
            };
            
            let comment_clone = created_comment.clone();
            tokio::spawn(async move {
                crate::mailer::send_new_comment_alert(&comment_clone).await;
            });
            
            // 🌟 UPDATED to use standard class
            let success_msg = "<div class=\"comment-success-msg\">Thank you! Your comment has been submitted and is awaiting moderation.</div>".to_string();
            (StatusCode::CREATED, Html(success_msg)).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("<p style=\"color: red;\">Failed to save comment.</p>".to_string()),
        )
            .into_response(),
    }
}

fn render_comment_item(c: &Comment) -> String {
    // 🌟 UPDATED to use standard semantic HTML classes
    format!(
        "<div class=\"comment-item\">\
            <div class=\"comment-header\">\
                <span class=\"comment-author\">{name}</span>\
                <span class=\"comment-date\">{date}</span>\
            </div>\
            <p class=\"comment-content\">{content}</p>\
        </div>",
        name = c.author_name,
        date = c.created_at.format("%b %d, %Y at %H:%M"),
        content = c.content
    )
}

/* --------------------------------------------------------
   AUTHENTICATION LOGIC & LOGIN PAGE
-------------------------------------------------------- */

// 🌟 UPDATED: Redirects to /admin/login instead of throwing a blank 401 error
pub async fn require_admin_auth(req: Request, next: Next) -> Result<Response, Response> {
    let expected_token = std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| "secret-admin-key".to_string());
    
    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|val| val.to_str().ok()).map(|val| val.trim_start_matches("Bearer ").to_string());
    let cookie_token = req.headers().get(header::COOKIE).and_then(|val| val.to_str().ok()).and_then(|cookie_str| {
        cookie_str.split(';').find_map(|c| {
            let mut parts = c.trim().split('=');
            if parts.next()? == "admin_token" { Some(parts.next()?.to_string()) } else { None }
        })
    });

    if auth_header.as_deref() == Some(&expected_token) || cookie_token.as_deref() == Some(&expected_token) {
        Ok(next.run(req).await)
    } else {
        // Redirect unauthorized users
        Err((StatusCode::SEE_OTHER, [(header::LOCATION, "/admin/login")]).into_response())
    }
}

// 🌟 NEW: Renders the Login UI
pub async fn admin_login_form() -> impl IntoResponse {
    Html(render_login_page(None))
}

// 🌟 FIX 1: Added .into_response() to the else block
pub async fn admin_login_submit(Form(payload): Form<LoginForm>) -> impl IntoResponse {
    let expected = std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| "secret-admin-key".to_string());
    
    if payload.token == expected {
        let cookie = format!("admin_token={}; Path=/; HttpOnly; Max-Age=31536000", payload.token);
        (StatusCode::SEE_OTHER, [(header::SET_COOKIE, cookie), (header::LOCATION, "/admin".to_string())]).into_response()
    } else {
        Html(render_login_page(Some("Invalid Token. Please try again."))).into_response()
    }
}

// 🌟 FIX 2: Removed .to_string() from the location header
pub async fn admin_logout() -> impl IntoResponse {
    let cookie = "admin_token=; Path=/; HttpOnly; Max-Age=0"; // Kills the cookie
    (StatusCode::SEE_OTHER, [(header::SET_COOKIE, cookie), (header::LOCATION, "/admin/login")]).into_response()
}

fn render_login_page(error: Option<&str>) -> String {
    let err_msg = match error {
        Some(msg) => format!("<div class=\"p-3 mb-4 rounded bg-rose-950/50 border border-rose-900 text-rose-400 text-sm\">{}</div>", msg),
        None => "".to_string(),
    };

    format!("\
    <!DOCTYPE html>\
    <html lang=\"en\">\
    <head>\
        <meta charset=\"UTF-8\">\
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
        <title>Admin Login</title>\
        <script src=\"https://cdn.tailwindcss.com\"></script>\
    </head>\
    <body class=\"bg-neutral-950 text-neutral-100 min-h-screen flex items-center justify-center font-sans p-6\">\
        <div class=\"bg-neutral-900 border border-neutral-800 p-8 rounded-xl shadow-lg w-full max-w-md\">\
            <h1 class=\"text-2xl font-bold text-emerald-400 mb-6\">Admin Login</h1>\
            {error_html}\
            <form method=\"POST\" action=\"/admin/login\" class=\"space-y-4\">\
                <div>\
                    <label class=\"block text-xs font-semibold mb-1 text-neutral-400\">Authentication Token</label>\
                    <input type=\"password\" name=\"token\" required class=\"w-full px-3.5 py-2 bg-neutral-950 border border-neutral-800 rounded-lg text-sm outline-none focus:ring-2 focus:ring-emerald-500 text-white\" />\
                </div>\
                <button type=\"submit\" class=\"w-full px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg text-sm transition-colors shadow-sm\">\
                    Secure Login\
                </button>\
            </form>\
        </div>\
    </body>\
    </html>\
    ", error_html = err_msg)
}

/* --------------------------------------------------------
   ADMIN DASHBOARD ROUTES
-------------------------------------------------------- */

pub async fn admin_dashboard(State(_pool): State<SqlitePool>) -> impl IntoResponse {
    // 🌟 UPDATED: Added a Logout button to the header
    let html = "\
    <!DOCTYPE html>\
    <html lang=\"en\">\
    <head>\
        <meta charset=\"UTF-8\">\
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
        <title>Comment Moderation</title>\
        <script src=\"https://cdn.tailwindcss.com\"></script>\
        <script src=\"https://unpkg.com/htmx.org@1.9.12\"></script>\
    </head>\
    <body class=\"bg-neutral-950 text-neutral-100 min-h-screen font-sans p-6\">\
        <div class=\"max-w-6xl mx-auto space-y-6\">\
            <header class=\"flex items-center justify-between border-b border-neutral-800 pb-4\">\
                <div>\
                    <h1 class=\"text-2xl font-bold text-emerald-400\">Comment Moderation</h1>\
                    <p class=\"text-xs text-neutral-400 mt-1\">Review, approve, and purge blog comments</p>\
                </div>\
                <div class=\"flex gap-2\">\
                    <button hx-get=\"/admin/api/comments\" hx-target=\"#comment-table-body\" class=\"px-3 py-1.5 bg-neutral-800 hover:bg-neutral-700 text-xs font-semibold rounded-lg transition-colors\">Refresh</button>\
                    <a href=\"/admin/logout\" class=\"px-3 py-1.5 bg-rose-950 hover:bg-rose-900 text-rose-300 border border-rose-800 text-xs font-semibold rounded-lg transition-colors\">Logout</a>\
                </div>\
            </header>\
            <div class=\"bg-neutral-900 border border-neutral-800 rounded-xl overflow-hidden shadow-lg\">\
                <div class=\"overflow-x-auto\">\
                    <table class=\"w-full text-left text-sm\">\
                        <thead class=\"bg-neutral-800/60 text-xs uppercase text-neutral-400 border-b border-neutral-800\">\
                            <tr>\
                                <th class=\"px-4 py-3\">Status</th>\
                                <th class=\"px-4 py-3\">Post Slug</th>\
                                <th class=\"px-4 py-3\">Author</th>\
                                <th class=\"px-4 py-3\">Comment</th>\
                                <th class=\"px-4 py-3\">Date</th>\
                                <th class=\"px-4 py-3 text-right\">Actions</th>\
                            </tr>\
                        </thead>\
                        <tbody id=\"comment-table-body\" hx-get=\"/admin/api/comments\" hx-trigger=\"load\" class=\"divide-y divide-neutral-800\">\
                            <tr>\
                                <td colspan=\"6\" class=\"p-6 text-center text-neutral-500 animate-pulse\">Loading comments...</td>\
                            </tr>\
                        </tbody>\
                    </table>\
                </div>\
            </div>\
        </div>\
    </body>\
    </html>\
    ";

    Html(html.to_string())
}

pub async fn list_admin_comments(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let comments: Vec<Comment> = sqlx::query_as!(
        Comment,
        "SELECT id, post_slug, author_name, author_email, content, is_approved, created_at FROM comments ORDER BY created_at DESC LIMIT 100"
    ).fetch_all(&pool).await.unwrap_or_default();

    if comments.is_empty() {
        return Html("<tr><td colspan=\"6\" class=\"p-6 text-center text-neutral-500\">No comments found.</td></tr>".to_string());
    }

    let mut rows = String::new();
    for comment in comments {
        rows.push_str(&render_admin_row(&comment));
    }
    Html(rows)
}

pub async fn toggle_approve_comment(State(pool): State<SqlitePool>, Path(id): Path<String>) -> impl IntoResponse {
    let _ = sqlx::query!("UPDATE comments SET is_approved = NOT is_approved WHERE id = ?", id).execute(&pool).await;
    if let Ok(comment) = sqlx::query_as!(Comment, "SELECT * FROM comments WHERE id = ?", id).fetch_one(&pool).await {
        Html(render_admin_row(&comment))
    } else { Html("".to_string()) }
}

pub async fn delete_comment(State(pool): State<SqlitePool>, Path(id): Path<String>) -> impl IntoResponse {
    let _ = sqlx::query!("DELETE FROM comments WHERE id = ?", id).execute(&pool).await;
    Html("".to_string())
}

fn render_admin_row(c: &Comment) -> String {
    let status_badge = if c.is_approved {
        "<span class=\"inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold bg-emerald-950 text-emerald-400 border border-emerald-800\">Approved</span>"
    } else {
        "<span class=\"inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold bg-amber-950 text-amber-400 border border-amber-800\">Pending</span>"
    };

    let toggle_label = if c.is_approved { "Unapprove" } else { "Approve" };

    format!(
        "<tr id=\"comment-row-{id}\" class=\"hover:bg-neutral-800/40 transition-colors\">\
            <td class=\"px-4 py-3\">{badge}</td>\
            <td class=\"px-4 py-3 font-mono text-xs text-neutral-300\">{slug}</td>\
            <td class=\"px-4 py-3\">\
                <div class=\"font-bold text-neutral-200\">{author}</div>\
                <div class=\"text-xs text-neutral-500\">{email}</div>\
            </td>\
            <td class=\"px-4 py-3 text-neutral-300 max-w-md truncate\">{content}</td>\
            <td class=\"px-4 py-3 text-xs text-neutral-400 whitespace-nowrap\">{date}</td>\
            <td class=\"px-4 py-3 text-right space-x-2 whitespace-nowrap\">\
                <button hx-post=\"/admin/api/comments/{id}/toggle\" hx-target=\"#comment-row-{id}\" hx-swap=\"outerHTML\" class=\"px-2.5 py-1 text-xs font-semibold bg-neutral-800 hover:bg-neutral-700 text-neutral-200 rounded border border-neutral-700 transition-colors\">{toggle}</button>\
                <button hx-delete=\"/admin/api/comments/{id}\" hx-target=\"#comment-row-{id}\" hx-swap=\"outerHTML\" hx-confirm=\"Are you sure you want to permanently delete this comment?\" class=\"px-2.5 py-1 text-xs font-semibold bg-rose-950 hover:bg-rose-900 text-rose-300 rounded border border-rose-800 transition-colors\">Delete</button>\
            </td>\
        </tr>",
        id = c.id, badge = status_badge, slug = c.post_slug, author = c.author_name,
        email = c.author_email.as_deref().unwrap_or("-"), content = c.content,
        date = c.created_at.format("%Y-%m-%d %H:%M"), toggle = toggle_label
    )
}

pub async fn serve_css() -> impl axum::response::IntoResponse {
    let css = r###"
    :root {
        --mr-primary: #059669; /* Emerald 600 */
        --mr-bg: #f9fafb;
        --mr-border: #e5e7eb;
        --mr-text: #171717;
        --mr-muted: #737373;
    }
    .mr-container { font-family: system-ui, sans-serif; color: var(--mr-text); margin-top: 3rem; padding-top: 2rem; border-top: 1px solid var(--mr-border); }
    .mr-title { font-size: 1.5rem; font-weight: bold; margin-bottom: 1.5rem; }
    .mr-input { width: 100%; padding: 0.5rem 0.75rem; border: 1px solid var(--mr-border); border-radius: 0.5rem; background: var(--mr-bg); margin-top: 0.25rem; font-family: inherit; }
    .mr-btn { background: var(--mr-primary); color: white; padding: 0.5rem 1.25rem; border-radius: 0.5rem; border: none; font-weight: bold; cursor: pointer; }
    .mr-btn:hover { opacity: 0.9; }
    .mr-grid { display: grid; grid-template-columns: 1fr; gap: 1rem; margin-bottom: 1rem; }
    @media (min-width: 640px) { .mr-grid { grid-template-columns: 1fr 1fr; } }
    .comment-item { padding: 1rem; background: var(--mr-bg); border: 1px solid var(--mr-border); border-radius: 0.5rem; margin-bottom: 1rem; }
    .comment-header { display: flex; justify-content: space-between; margin-bottom: 0.5rem; }
    .comment-author { font-weight: bold; }
    .comment-date { font-size: 0.75rem; color: var(--mr-muted); }
    .comment-content { font-size: 0.875rem; line-height: 1.5; }
    .comment-success-msg { padding: 1rem; background-color: #ecfdf5; border: 1px solid #a7f3d0; color: #065f46; border-radius: 0.5rem; font-size: 0.875rem; }
    "###;
    (axum::http::StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")], css).into_response()
}

pub async fn serve_js() -> impl axum::response::IntoResponse {
    let js = r###"
(function() {
    const scriptTag = document.currentScript;
    const backendOrigin = new URL(scriptTag ? scriptTag.src : window.location.href).origin;
    
    // Look for the container and its data-slug attribute
    const container = document.getElementById("markreply-comments");
    if (!container) return;
    
    const postSlug = container.getAttribute("data-slug") || window.location.pathname;

    // Inject CSS
    if (!document.querySelector(`link[href="${backendOrigin}/widget.css"]`)) {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = `${backendOrigin}/widget.css`;
        document.head.appendChild(link);
    }

    container.innerHTML = `
        <div class="mr-container">
            <h3 class="mr-title">Discussion</h3>
            <div id="mr-list" style="margin-bottom: 2rem;"><p style="color: var(--mr-muted); font-size: 0.875rem;">Loading comments...</p></div>
            
            <form id="mr-form" style="max-width: 36rem;">
                <input type="hidden" id="mr-slug" value="${postSlug}">
                <input type="text" id="mr-honeypot" style="display:none" tabindex="-1" autocomplete="off">
                
                <div class="mr-grid">
                    <div>
                        <label style="font-size: 0.75rem; font-weight: 600;">Name *</label>
                        <input type="text" id="mr-name" required class="mr-input" placeholder="Jane Doe">
                    </div>
                    <div>
                        <label style="font-size: 0.75rem; font-weight: 600;">Email (Optional)</label>
                        <input type="email" id="mr-email" class="mr-input" placeholder="jane@example.com">
                    </div>
                </div>
                <div style="margin-bottom: 1rem;">
                    <label style="font-size: 0.75rem; font-weight: 600;">Comment *</label>
                    <textarea id="mr-content" rows="3" required class="mr-input" placeholder="Write a comment..."></textarea>
                </div>
                <div id="mr-status" style="margin-bottom: 1rem;"></div>
                <button type="submit" class="mr-btn" id="mr-submit">Post Comment</button>
            </form>
        </div>
    `;

    const listEl = document.getElementById("mr-list");
    const formEl = document.getElementById("mr-form");
    const statusEl = document.getElementById("mr-status");

    async function fetchComments() {
        try {
            const res = await fetch(`${backendOrigin}/api/comments?slug=${encodeURIComponent(postSlug)}`);
            listEl.innerHTML = await res.text();
        } catch (err) {
            listEl.innerHTML = '<p>Could not load comments.</p>';
        }
    }

    formEl.addEventListener("submit", async (e) => {
        e.preventDefault();
        document.getElementById("mr-submit").disabled = true;
        
        const formData = new URLSearchParams();
        formData.append("post_slug", postSlug);
        formData.append("author_name", document.getElementById("mr-name").value);
        formData.append("author_email", document.getElementById("mr-email").value);
        formData.append("content", document.getElementById("mr-content").value);
        formData.append("honeypot", document.getElementById("mr-honeypot").value);

        try {
            const res = await fetch(`${backendOrigin}/api/comments`, { method: "POST", body: formData });
            statusEl.innerHTML = await res.text();
            if (res.ok) {
                formEl.reset();
                fetchComments();
            }
        } finally {
            document.getElementById("mr-submit").disabled = false;
        }
    });

    fetchComments();
})();
    "###;
    (axum::http::StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "application/javascript; charset=utf-8")], js).into_response()
}