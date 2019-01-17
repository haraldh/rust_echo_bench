use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!(
        r#"Echo benchmark.

Usage:
  {program} [ -a <address> ] [ -l <length> ] [ -c <number> ] [ -t <duration> ]
  {program} (-h | --help)
  {program} --version"#,
        program = program
    );
    print!("{}", opts.usage(&brief));
}

struct Count {
    inb: u64,
    outb: u64,
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Print this help.");
    opts.optopt(
        "a",
        "address",
        "Target echo server address. Default: 127.0.0.1:12345",
        "<address>",
    );
    opts.optopt(
        "l",
        "length",
        "Test message length. Default: 512",
        "<length>",
    );
    opts.optopt(
        "t",
        "duration",
        "Test duration in seconds. Default: 60",
        "<duration>",
    );
    opts.optopt(
        "c",
        "number",
        "Test connection number. Default: 50",
        "<number>",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}", f.to_string());
            print_usage(&program, &opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return;
    }

    let length = matches
        .opt_str("length")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap_or(512);
    let duration = matches
        .opt_str("duration")
        .unwrap_or_default()
        .parse::<u64>()
        .unwrap_or(60);
    let number = matches
        .opt_str("number")
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or(50);
    let address = matches
        .opt_str("address")
        .unwrap_or_else(|| "127.0.0.1:12345".to_string());

    let (tx, rx) = mpsc::channel();

    let stop = Arc::new(AtomicBool::new(false));
    let control = Arc::downgrade(&stop);

    for _ in 0..number {
        let tx = tx.clone();
        let address = address.clone();
        let stop = stop.clone();
        let length = length;

        thread::spawn(move || {
            let mut sum = Count { inb: 0, outb: 0 };
            let mut out_buf: Vec<u8> = vec![0; length];
            out_buf[length - 1] = b'\n';
            let mut in_buf: Vec<u8> = vec![0; length];
            let mut stream = TcpStream::connect(&*address).unwrap();

            loop {
                if (*stop).load(Ordering::Relaxed) {
                    break;
                }

                match stream.write_all(&out_buf) {
                    Err(_) => {
                        println!("Write error!");
                        break;
                    }
                    Ok(_) => sum.outb += 1,
                }

                if (*stop).load(Ordering::Relaxed) {
                    break;
                }

                match stream.read(&mut in_buf) {
                    Err(_) => break,
                    Ok(m) => {
                        if m == 0 || m != length {
                            println!("Read error! length={}", m);
                            break;
                        }
                    }
                };
                sum.inb += 1;
            }
            tx.send(sum).unwrap();
        });
    }

    thread::sleep(Duration::from_secs(duration));

    match control.upgrade() {
        Some(stop) => (*stop).store(true, Ordering::Relaxed),
        None => println!("Sorry, but all threads died already."),
    }

    let mut sum = Count { inb: 0, outb: 0 };
    for _ in 0..number {
        let c: Count = rx.recv().unwrap();
        sum.inb += c.inb;
        sum.outb += c.outb;
    }
    println!("Benchmarking: {}", address);
    println!(
        "{} clients, running {} bytes, {} sec.",
        number, length, duration
    );
    println!();
    println!(
        "Speed: {} request/sec, {} response/sec",
        sum.outb / duration,
        sum.inb / duration
    );
    println!("Requests: {}", sum.outb);
    println!("Responses: {}", sum.inb);
}
