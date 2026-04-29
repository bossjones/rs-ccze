# Builds the C ccze reference inside Linux. The resulting image is the
# canonical oracle for snapshot baseline generation — see
# rust/scripts/generate-baseline.sh.
#
# Build from the C-tree root:
#   docker build -f rust/scripts/Dockerfile -t ccze:reference .
#
# Run in the typical "produce a debug-mode snapshot" shape:
#   docker run --rm -i ccze:reference -F /dev/null -d -o nolookups -p syslog \
#     < testdata/bug-sysrepeat.in > testdata/bug-sysrepeat.ok
FROM debian:12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        libpcre3-dev \
        libncurses-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . /src

# All plugins linked statically. The default Makefile builds plugins as .so
# and dlopen()s them at runtime, which forces the binary to be installed at
# its compile-time PKGLIBDIR — way more cycles than we want for a baseline
# tool.
RUN make distclean 2>/dev/null || true \
 && ./configure --build="$(uname -m)-unknown-linux-gnu" \
 && sed -i 's|^S_PLUGINS\s*=.*|S_PLUGINS = apm distcc dpkg exim fetchmail ftpstats httpd icecast oops php postfix procmail proftpd squid sulog super syslog ulogd vsftpd xferlog|' src/Makefile \
 && make \
 && cp src/ccze /usr/local/bin/ccze \
 && /usr/local/bin/ccze -V

ENTRYPOINT ["/usr/local/bin/ccze"]
