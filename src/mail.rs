use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::ConnectionReuseParameters;
use lettre::{SmtpClient, Transport};
use lettre_email::Email;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ContactMail {
    pub sender_address: String,
    pub message: String,
}

pub struct Config {
    pub password: String,
}

pub fn send_contact_mail(config: Config, mail_data: ContactMail) {
    let email = Email::builder()
        // Addresses can be specified by the tuple (email, alias)
        .to(("kontakt@marcelkoch.net", "Marcel Koch"))
        .from(mail_data.sender_address)
        .subject("Kontaktformular")
        .text(mail_data.message)
        .build()
        .unwrap();

    // Open a local connection on port 25
    let mut mailer = SmtpClient::new_simple("koch.kasserver.com")
        .unwrap()
        .smtp_utf8(true)
        .credentials(Credentials::new(
            "kontakt@marcelkoch.net".to_string(),
            config.password,
        ))
        .authentication_mechanism(Mechanism::Login)
        .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
        .transport();
    // Send the email
    let result = mailer.send(email.into());

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    mailer.close();
}
