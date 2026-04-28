//! Plugin registry. Each plugin is a unit struct implementing `Plugin`.
//!
//! In C, plugins were loaded via `dlopen` from `.so` files. The Rust port
//! drops dynamic loading entirely — every plugin is statically registered
//! here, selected at run time by name from the `-p` flag.

use crate::plugin::Plugin;

pub mod apm;
pub mod distcc;
pub mod dpkg;
pub mod exim;
pub mod fetchmail;
pub mod ftpstats;
pub mod httpd;
pub mod icecast;
pub mod oops;
pub mod php;
pub mod postfix;
pub mod procmail;
pub mod proftpd;
pub mod squid;
pub mod sulog;
#[path = "super_.rs"]
pub mod super_;
pub mod syslog;
pub mod ulogd;
pub mod vsftpd;
pub mod xferlog;

/// Build the list of all known plugins. The order here matters for dispatch:
/// for a given line, Full plugins are tried in registration order until one
/// matches. (The C `ccze_plugin_run` does the same — see ccze.c:706-720.)
/// Syslog goes first so it gets first shot at a `Mon DD HH:MM:SS host …` line;
/// procmail's regex is permissive enough to swallow nearly anything otherwise.
pub fn all_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(syslog::Syslog::new()),
        Box::new(httpd::Httpd::new()),
        Box::new(dpkg::Dpkg::new()),
        Box::new(php::Php::new()),
        Box::new(super_::Super::new()),
        Box::new(distcc::Distcc::new()),
        Box::new(vsftpd::Vsftpd::new()),
        Box::new(sulog::Sulog::new()),
        Box::new(ftpstats::Ftpstats::new()),
        Box::new(oops::Oops::new()),
        Box::new(exim::Exim::new()),
        Box::new(xferlog::Xferlog::new()),
        Box::new(icecast::Icecast::new()),
        Box::new(proftpd::Proftpd::new()),
        Box::new(squid::Squid::new()),
        Box::new(procmail::Procmail::new()),
        // Partial-type plugins (run on the residue of a Full match):
        Box::new(apm::Apm::new()),
        Box::new(fetchmail::Fetchmail::new()),
        Box::new(postfix::Postfix::new()),
        Box::new(ulogd::Ulogd::new()),
    ]
}

/// Filter the plugin set down to a user-supplied selection (mirrors the
/// behaviour of the `-p` flag in C ccze).
pub fn filter_by_name(all: Vec<Box<dyn Plugin>>, names: &[String]) -> Vec<Box<dyn Plugin>> {
    if names.is_empty() {
        return all;
    }
    all.into_iter()
        .filter(|p| names.iter().any(|n| n == p.name()))
        .collect()
}
