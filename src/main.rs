use clap::Parser;
use clokwerk::{Scheduler, TimeUnits};
use lettre::message::MessageBuilder;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const RSS_URL: &str = "https://gg.deals/au/news/feed/";

#[derive(Parser)]
#[clap(version = "0.1", author = "Samit Shaikh <shaikhsamit@live.com>")]
struct Opts {
    #[clap(
        short,
        long,
        value_name = "ADDRESS",
        about = "Email address to send freebies to."
    )]
    to: String,
    #[clap(
        short,
        long,
        value_name = "ADDRESS",
        about = "Email address to send freebies from."
    )]
    from: String,
    #[clap(short = 's', long, value_name = "URL", about = "Sets SMTP server URL.")]
    smtp_server: String,
    #[clap(
        short = 'u',
        long,
        value_name = "USERNAME",
        about = "Set SMTP server username."
    )]
    smtp_username: String,
    #[clap(
        short = 'p',
        long,
        value_name = "PASSWORD",
        about = "Set SMTP server password."
    )]
    smtp_password: String,
    #[clap(
        short = 'm',
        long,
        value_name = "NUMBER",
        about = "Maximum number of sent freebies to remember.",
        default_value = "25"
    )]
    max: usize,
}

struct Sent {
    list: VecDeque<String>,
    max: usize,
}

impl Sent {
    pub fn new(max: usize) -> Sent {
        Sent {
            list: VecDeque::new(),
            max,
        }
    }

    fn add(&mut self, item: String) {
        if self.list.len() == self.max {
            self.list.pop_back();
        }

        self.list.push_front(item);
    }

    fn contains(&self, item: String) -> bool {
        self.list.contains(&item)
    }
}

struct Mailer {
    transport: SmtpTransport,
    base_mail: MessageBuilder,
}

impl Mailer {
    pub fn new(server: String, user: String, pass: String, from: String, to: String) -> Mailer {
        Mailer {
            transport: SmtpTransport::relay(&server)
                .unwrap()
                .credentials(Credentials::new(user, pass))
                .build(),
            base_mail: Message::builder()
                .from(format!("Freebies <{}>", from).parse().unwrap())
                .to(to.parse().unwrap()),
        }
    }

    fn send_mail(&self, subject: &str, body: String) {
        let mail = &self.base_mail.clone().subject(subject).body(body).unwrap();

        match &self.transport.send(&mail) {
            Ok(r) => {
                if r.is_positive() {
                    println!(
                        "\"{}\" sent successfully! Response code: {}.",
                        subject,
                        r.code()
                    );
                } else {
                    println!("\"{}\" sent with negative response. Response code: {}. Server response message:", subject, r.code());
                    for line in r.message() {
                        println!("{}", line);
                    }
                }
            }
            Err(e) => panic!("Could not send email: {:?}", e),
        }
    }
}

fn check(sent: Arc<Mutex<Sent>>, mailer: Arc<Mailer>) {
    fn get_channel() -> Result<rss::Channel, Box<dyn Error>> {
        let data = reqwest::blocking::get(RSS_URL)?.bytes()?;
        let channel = rss::Channel::read_from(&data[..])?;

        println!("Got RSS channel.");

        Ok(channel)
    }

    fn parse_desc(desc: &str) -> String {
        let mut out: String = String::new();
        let mut started = false;
        for char in desc.chars() {
            if started {
                if char == '<' {
                    return out;
                }
                out.push(char);
            } else if char == '>' {
                started = true;
            }
        }

        out
    }

    println!("Starting check.");

    match get_channel() {
        Ok(channel) => {
            for item in channel.items {
                let title = item.title().unwrap();
                if title.contains("FREE") && !sent.lock().unwrap().contains(title.to_string()) {
                    println!("Sending: {}", title);
                    sent.lock().unwrap().add(title.to_string());
                    let desc = parse_desc(item.description().unwrap());
                    let link = item.link().unwrap();
                    let published = item.pub_date().unwrap();
                    let author = item.author().unwrap();
                    mailer.send_mail(
                        title,
                        format!(
                            "Description: {}\nLink: {}\nPublished: {}\nAuthor: {}",
                            desc, link, published, author
                        ),
                    )
                }
            }
        }
        Err(e) => {
            println!(
                "Error retrieving RSS channel from \"{}\". Err: {}",
                RSS_URL, e
            )
        }
    };

    // print!("{:#?}", sent.lock().unwrap().list);
    println!("finished check");
}

fn main() {
    let opts = Opts::parse();
    let sent = Arc::new(Mutex::new(Sent::new(opts.max)));
    let mailer = Arc::new(Mailer::new(
        opts.smtp_server,
        opts.smtp_username,
        opts.smtp_password,
        opts.from,
        opts.to,
    ));
    let task = move || {
        check(sent.clone(), mailer.clone());
    };

    println!("Running task.");
    task();

    println!("Starting loop.");
    let mut scheduler = Scheduler::new();
    scheduler.every(1.hour()).run(task);
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(100));
    }
}
