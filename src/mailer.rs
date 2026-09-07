use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncTransport, Message,
};
use crate::models::Comment;

pub async fn send_new_comment_alert(comment: &Comment) {
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_default();
    let smtp_user = std::env::var("SMTP_USER").unwrap_or_default();
    let smtp_pass = std::env::var("SMTP_PASS").unwrap_or_default();
    let admin_email = std::env::var("ADMIN_EMAIL").unwrap_or_default();

    if smtp_host.is_empty() || admin_email.is_empty() {
        tracing::warn!("SMTP credentials missing; skipping email notification.");
        return;
    }

    let email_body = format!(
        "A new comment was posted on your blog!\n\n\
        Post: {}\n\
        Author: {} ({})\n\
        Date: {}\n\n\
        Comment:\n{}\n\n\
        Review or approve it here: https://comments.yourdomain.com/admin",
        comment.post_slug,
        comment.author_name,
        comment.author_email.as_deref().unwrap_or("No email provided"),
        comment.created_at.format("%Y-%m-%d %H:%M"),
        comment.content
    );

    let email = Message::builder()
        .from(smtp_user.parse().unwrap())
        .to(admin_email.parse().unwrap())
        .subject(format!("New Comment on {}", comment.post_slug))
        .header(ContentType::TEXT_PLAIN)
        .body(email_body)
        .unwrap();

    let creds = Credentials::new(smtp_user.clone(), smtp_pass);
    
    let mailer = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&smtp_host)
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(email).await {
        Ok(_) => tracing::info!("Notification email sent to {}", admin_email),
        Err(e) => tracing::error!("Could not send email: {:?}", e),
    }
}