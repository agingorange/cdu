# Cloudflare DNS Updater

This Rust program is a command-line utility for updating the A record of a domain on Cloudflare to
match the current outside IP address.

## Why use it?

I have a home server that I want to access from the internet. My ISP assigns me a dynamic IP
address, so it could change any time. I don't want to have to keep track of the IP address and
manually update the DNS record. This program does that for me. The IP address really doesn't change
often, I just don't want to be cut off at an inoppurtune time.

## How to use it

In its simplest form, it can be used like this:

```sh
cdu
```

It probably won't work, because it needs some environment variables set, or commandline arguments.
Check the help for more information:

```sh
cdu --help
```

Let's try again, but this time with the required environment variables set:

```sh
CDU_API_KEY="my-api-key" CDU_ZONE_ID="my-zone-id" CDU_DOMAIN="test.com" CDU_DRY_RUN=true cdu
```

or using commandline arguments

```sh
cdu --api-key my-api-key --zone-id my-zone-id --domain test.com --dry-run
```

> NOTE: From now on, I will assume that you have the environment variables set.

A dry run means that it will not actually update the DNS record, but it will print what it would do.
It's a good idea to use this when you first start using the program, to make sure it's going to do
what you expect.

A file called `cdu.toml` is saved in the directory from where you run the program. Amongst other
things, it will save the last outside IP address that it saw, so that it can compare it to the
current one and only contact Cloudflare if it's different. This is useful if you're running the
program on a schedule, which is the most common use case.

The program makes use of the crate [tracing-subscriber](https://crates.io/crates/tracing-subscriber) for logging, so
you can set the `RUST_LOG` environment variable to `debug` to see more detailed information about
what the program is doing.

```sh
RUST_LOG=debug cdu
```

## How to get it?

You're on GitHub, so you probably already know how to get it. You can clone the repository and build
it yourself, or you can download the binary from the releases page. To build it yourself, you'll
need the Rust toolchain installed. I recommend using [rustup](https://rustup.rs/).

## How do I use it?

I've exported all arguments as environment variables, and I've scheduled it to run every five
minutes. Don't be afraid of rate limiting, because the program will only contact Cloudflare if the
outside IP address has changed.

Here's the command, which outputs to console and appends to a logfile.

```sh
RUST_LOG=info cdu 2>&1 | tee -a /var/log/cdu.log
```

If your log file is getting too big, you can use `logrotate` to manage it, or just truncate or
delete it from time to time, using cron or even manually.
