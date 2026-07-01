# ccze screenshot gallery

Every image below is unedited `ccze` output. The caption over each terminal is the exact command
that produced it, so you can reproduce any of these against your own logs.

The point of the gallery is to show the same tool adapting to wildly different log formats: `ccze`
doesn't apply one generic highlight rule everywhere — each **format-aware plugin** understands the
structure of its log and colors the fields that carry meaning (queue IDs, status codes, cache
verdicts, severities), so scanning a log becomes reading a log.

> Run any of these yourself: `tail -f <logfile> | ccze -A`. On a TTY, `-A` (ANSI) is the default —
> pass `-h` instead to capture a self-contained HTML document you can share or archive.

---

## The pitch, in one frame

```sh
cat /var/log/*.log | ccze -A   # one colorizer, every format
```

Apache, syslog, squid, Postfix, apm, dpkg, PHP, and vsftpd lines — all in the same stream, each
recognized and colored by its own plugin, no configuration.

![ccze colorizing a mix of Apache, syslog, squid, Postfix, apm, dpkg, PHP, and vsftpd log lines in a single stream](images/rs-ccze-hero-mixed-logs.png)

---

## syslog — daemon messages

```sh
tail -f /var/log/syslog | ccze -A
```

The `syslog` plugin parses the `Mon DD HH:MM:SS host process[pid]:` preamble on every line, so
timestamps, hostnames, and PIDs are pulled out consistently. Keywords such as `started`, `Exit`,
`Started`, and `down` are highlighted so daemon state changes — xinetd restarts, pppd scripts
finishing, cron jobs, syslog-ng shutting down — stand out from routine chatter.

![ccze colorizing syslog output: xinetd, pppd, CRON, and syslog-ng messages with highlighted timestamps, hosts, and PIDs](images/rs-ccze-syslog-daemon-messages.png)

---

## Apache httpd — access log

```sh
tail -f /var/log/apache2/access.log | ccze -A
```

Combined-log-format access lines: client IPs, the quoted request line, the HTTP status code, and
the response size each get their own color. A `400` reads differently from a `200` at a glance, so
error responses surface immediately when you're scrolling a busy access log.

![ccze colorizing an Apache access log with colored IPs, request lines, and HTTP status codes](images/rs-ccze-apache-access-log.png)

---

## Postfix — mail log

```sh
tail -f /var/log/mail.log | ccze -A
```

The `postfix` plugin recognizes queue IDs and the `client=`, `from=`, `to=`, `relay=`, `size=`,
`delay=`, and `status=` fields across `smtpd`, `qmgr`, and `smtp`. Because the queue ID is colored
consistently, you can follow a single message through arrival, queueing, and delivery just by
tracking one token down the screen.

![ccze colorizing a Postfix mail log showing queue IDs, client/from/to fields, and delivery status](images/rs-ccze-postfix-mail-log.png)

---

## Exim — mail queue

```sh
tail -f /var/log/exim4/mainlog | ccze -A
```

Exim's terse mainlog becomes legible: the `<=` (arrival), `=>` (delivery), and `==` (defer) arrows
are colored, along with message IDs, sender/recipient addresses, and `Completed` / `Start queue
run` markers — the full lifecycle of a message at a glance.

![ccze colorizing an Exim mainlog with colored message IDs, delivery arrows, and addresses](images/rs-ccze-exim-mail-queue.png)

---

## squid — proxy cache

```sh
tail -f /var/log/squid/access.log | ccze -A
```

Cache verdicts are the story in a squid log, so the `squid` plugin color-codes `TCP_HIT`,
`TCP_MISS`, and `TCP_DENIED` alongside their HTTP status codes. Hits, misses, and denials become
instantly distinguishable, and the trailing `Listening`/`running` daemon lines are colored too.

![ccze colorizing a squid access log with colored cache verdicts and HTTP status codes](images/rs-ccze-squid-proxy-cache.png)

---

## PHP — error log

```sh
tail -f /var/log/php_errors.log | ccze -A
```

Severity escalates in color: `Notice` → `Warning` → `Fatal error` each get a distinct treatment,
and the file paths and line numbers embedded in the message are picked out so you can jump straight
to the offending source.

![ccze colorizing a PHP error log with severity-escalating colors and highlighted file paths](images/rs-ccze-php-error-log.png)

---

## dpkg — package log

```sh
tail -f /var/log/dpkg.log | ccze -A
```

The `dpkg` plugin colors the action (`upgrade`, `status`, `conffile`, `install`), the package name,
and the version numbers separately, turning a long wall of package history into a scannable
timeline of what changed and when.

![ccze colorizing a dpkg log with colored actions, package names, and version numbers](images/rs-ccze-dpkg-package-log.png)

---

## ProFTPD — transfer log

```sh
tail -f /var/log/proftpd/xferlog | ccze -A
```

xferlog transfer records (`RETR` downloads, `STOR` uploads) show colored client IPs, usernames,
paths, and byte counts, while FTP command lines (`USER`, `PASS`) are colored on the same screen —
handy for auditing exactly who moved which file.

![ccze colorizing a ProFTPD xferlog with colored transfer commands, paths, and byte counts](images/rs-ccze-proftpd-transfer-log.png)

---

## vsftpd — FTP log

```sh
tail -f /var/log/vsftpd.log | ccze -A
```

Login outcomes drive the color: `FAIL` lights up red while `OK` reads green, and `CONNECT` and
`DOWNLOAD` events show colored clients, paths, sizes, and transfer rates. A failed anonymous login
is impossible to miss.

![ccze colorizing a vsftpd log with colored login outcomes, connects, and download details](images/rs-ccze-vsftpd-ftp-log.png)

---

## And 11 more

The screenshots above cover 9 of the 20 ported plugins. The rest — `super`, `distcc`, `sulog`,
`ftpstats`, `oops`, `xferlog`, `icecast`, `procmail`, `apm`, `fetchmail`, and `ulogd` — follow the
same principle: recognize the format, color what matters. See the
[status table in the README](../README.md#status--port-complete) for the full list, and
`testdata/snap-<plugin>.in` for a sample input line per plugin.
