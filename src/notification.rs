use reqwest::Client;
use lettre::{Message, SmtpTransport, Transport};

pub async fn call_api(url: &str, payload: &str) {
    let client = Client::new();
    match client.post(url)
        .body(payload.to_string())
        .send()
        .await {
            Ok(response) => println!("API call successful: {:?}", response),
            Err(e) => println!("API call failed: {:?}", e),
        }
}

pub fn send_email(to: &str, subject: &str, body: &str) {
    let email = Message::builder()
        .from("your_email@example.com".parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(body.to_string())
        .unwrap();

    let mailer = SmtpTransport::builder_dangerous("smtp.example.com")
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => println!("Could not send email: {:?}", e),
    }
}
