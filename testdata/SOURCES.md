# Snapshot fixture provenance

The C reference binary used to mint every `*.ok` file under this directory is
built from the C tree at `/Users/malcolm/dev/ccze/ccze/`, packaged into the
`ccze:reference` Docker image (see `rust/scripts/Dockerfile`). To regenerate
any baseline:

```
rust/scripts/build-c-ref.sh                  # one-time, after C source changes
rust/scripts/generate-baseline.sh <name> <plugins>
```

The image is currently built from the C tree at git HEAD `da40b19` (`Import`).

## bug-* fixtures (originals)

Copied verbatim from the C testsuite at `/Users/malcolm/dev/ccze/ccze/testsuite/`.
Authoritative source for what the original `make check` exercised.

| Name                | Plugins              | Origin                   |
|---------------------|----------------------|--------------------------|
| bug-sysrepeat       | syslog               | C testsuite              |
| bug-wnum            | syslog               | C testsuite              |
| bug-procmailsubj    | syslog,procmail      | C testsuite              |
| bug-procmailsubj2   | syslog,procmail      | C testsuite              |
| bug-httpd           | httpd                | C testsuite              |
| bug-dpkg            | dpkg                 | C testsuite              |

## snap-* fixtures (synthetic)

Hand-crafted inputs designed to match each plugin's regex; outputs minted from
the Docker C reference. Sources for each input:

| Name           | Plugins                | Input source                                                                  |
|----------------|------------------------|-------------------------------------------------------------------------------|
| snap-apm       | syslog,apm             | Constructed from `mod_apm.c` regex.                                           |
| snap-distcc    | distcc                 | Constructed from `mod_distcc.c` regex; representative distccd messages.       |
| snap-exim      | exim                   | Constructed from `mod_exim.c` regex; standard `<=` `=>` `==` action symbols.  |
| snap-fetchmail | syslog,fetchmail       | Constructed from `mod_fetchmail.c` regex (`reading message X@H:N of M`).      |
| snap-ftpstats  | ftpstats               | Constructed from `mod_ftpstats.c` regex (pure-ftpd stats line).               |
| snap-icecast   | icecast                | Constructed from both icecast regexes (regular + usage).                      |
| snap-oops      | oops                   | Constructed from `mod_oops.c` regex (`statistics()`).                         |
| snap-php       | php                    | Constructed from `mod_php.c` regex (`[date] PHP <severity>: ...`).            |
| snap-postfix   | syslog,postfix         | Constructed from `mod_postfix.c` regex; client/from/to events.                |
| snap-proftpd   | proftpd                | Constructed from both proftpd regexes (access + auth).                        |
| snap-squid     | squid                  | Constructed from `mod_squid.c` regexes (access + cache log).                  |
| snap-sulog     | sulog                  | Constructed from `mod_sulog.c` regex (`SU MM/DD HH:MM ± tty from-to`).        |
| snap-super     | super                  | Constructed from `mod_super.c` regex.                                         |
| snap-ulogd     | syslog,ulogd           | Constructed from `mod_ulogd.c` regex (kernel firewall log).                   |
| snap-vsftpd    | vsftpd                 | Constructed from `mod_vsftpd.c` regex.                                        |
| snap-xferlog   | xferlog                | Constructed from `mod_xferlog.c` regex (FTP transfer log line).               |

## Why some need syslog ahead of them

Plugins of type `Partial` (apm, fetchmail, postfix, ulogd) only run on the
residue left over after a `Full` plugin has consumed a line's prefix. Their
fixtures are syslog-formatted so that `syslog` matches first, peels off the
`<date> <host> <process>[<pid>]:` preamble, and hands the message body to the
Partial plugin.
