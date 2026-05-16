use lettre::{
    transport::smtp::{authentication::Credentials, response::Response},
    Message, SmtpTransport, Transport,
};
use lettre::message::header::ContentType;
use crate::config::Config;

pub async fn send_password_reset_email(
    config: &Config,
    to_email: &str,
    token: &str,
) -> Result<Response, lettre::transport::smtp::Error> {
    let reset_link = format!("{}/auth/reset-password-form?token={}", config.base_url, token);

    let email_body = format!(
    r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background-color: #f8f9fa; padding: 20px; border-radius: 5px;">
        <h2 style="color: #2c3e50; margin-top: 0;">Password Reset Request</h2>
        
        <p>Hello,</p>
        
        <p>We received a request to reset the password for your account. Click the button below to create a new password:</p>
        
        <div style="text-align: center; margin: 30px 0;">
            <a href="{}" 
               style="background-color: #007bff; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block; font-weight: bold;">
                Reset Your Password
            </a>
        </div>
        
        <p style="font-size: 14px; color: #666;">
            Or copy and paste this link into your browser:<br>
            <a href="{}" style="color: #007bff; word-break: break-all;">{}</a>
        </p>
        
        <div style="background-color: #fff3cd; border-left: 4px solid #ffc107; padding: 12px; margin: 20px 0;">
            <p style="margin: 0; font-size: 14px;">
                <strong>⚠️ Security Notice:</strong><br>
                • This link will expire in <strong>12 hours</strong><br>
                • Do not share this link with anyone<br>
                • If you didn't request this reset, please ignore this email
            </p>
        </div>
        
        <p style="font-size: 13px; color: #666; margin-top: 30px;">
            If you're having trouble with the button above, you can also reset your password by visiting the password reset page directly and entering your email.
        </p>
        
        <div style="background-color: #e8f4f8; border-left: 4px solid #17a2b8; padding: 15px; margin: 25px 0; border-radius: 3px;">
            <p style="margin: 0 0 10px 0; font-size: 14px;">
                <strong>📝 Need Help?</strong>
            </p>
            <p style="margin: 0 0 10px 0; font-size: 13px; color: #555;">
                If you're experiencing issues with the password reset process, please report your problem using our support form:
            </p>
            <div style="text-align: center; margin-top: 15px;">
                <a href="https://docs.google.com/forms/d/e/1FAIpQLSegH-tD5UbHVlApETNhzHdUTJ1-dXKmHsgKwrKL9mt3mCFKWg/viewform" 
                   style="background-color: #17a2b8; color: white; padding: 10px 25px; text-decoration: none; border-radius: 5px; display: inline-block; font-size: 14px;">
                    Report an Issue
                </a>
            </div>
            <p style="margin: 10px 0 0 0; font-size: 12px; color: #666;">
                Our support team will assist you with manual password reset.
            </p>
        </div>
        
        <hr style="border: none; border-top: 1px solid #e0e0e0; margin: 30px 0;">
        
        <p style="font-size: 12px; color: #999;">
            Best regards,<br>
            <strong>Morty's Team</strong><br><br>
            This is an automated message, please do not reply to this email.
        </p>
    </div>
</body>
</html>
    "#,
    reset_link, reset_link, reset_link
);

    let email = Message::builder()
        .from(config.from_email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject("Password Reset Request")
        .header(ContentType::TEXT_HTML)
        .body(email_body)
        .unwrap();

    let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

    // FIXED: Use relay() which automatically handles STARTTLS
    let mailer = SmtpTransport::relay(&config.smtp_host)?
        .credentials(creds)
        .build();

    mailer.send(&email)
}
