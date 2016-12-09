use std::thread;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

extern crate rustc_serialize;
extern crate docopt;
use docopt::Docopt;

const USAGE: &'static str = "
Echo benchmark.

Usage:
  echo_bench [ -a <address> ] [ -l <length> ] [ -c <number> ] [ -t <duration> ]
  echo_bench (-h | --help)
  echo_bench --version

Options:
  -h, --help                 Show this screen.
  -a, --address <address>    Target echo server address.
  -l, --length <length>      Test message length.
  -t, --duration <duration>  Test duration in seconds.
  -c, --number <number>      Test connection number.
";

#[derive(RustcDecodable,Debug)]
struct Args {
    flag_address: Option<String>,
    flag_length: Option<usize>,
    flag_duration: Option<u64>,
    flag_number: Option<u32>,
}

struct Count {
    inb: u64,
    outb: u64,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let length = args.flag_length.unwrap_or(26);
    let address = args.flag_address.unwrap_or("127.0.0.1:12345".to_string());
    let duration = args.flag_duration.unwrap_or(60);
    let number = args.flag_number.unwrap_or(50);

    let (tx, rx) = mpsc::channel();

    let stop = Arc::new(AtomicBool::new(false));
    let control = Arc::downgrade(&stop);

    for _ in 0..number {
        let tx = tx.clone();
        let address = address.clone();
        let stop = stop.clone();
        let length = length.clone();

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
                    Err(_) => break,
                    Ok(_) => sum.outb += 1,
                }

                if (*stop).load(Ordering::Relaxed) {
                    break;
                }

                match stream.read(&mut in_buf) {
                    Err(_) => break,
                    Ok(m) => {
                        if m == 0 || m != length {
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
    println!("{} clients, running {} bytes, {} sec.",
             number,
             length,
             duration);
    println!("");
    println!("Speed: {} request/sec, {} response/sec",
             sum.outb / duration,
             sum.inb / duration);
    println!("Requests: {}", sum.outb);
    println!("Responses: {}", sum.inb);
}
