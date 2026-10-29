/* Symbols DuckDB references that wasi-libc does not provide or where fallback
   stubs are needed. Each call fails closed or acts as safe no-op. */
#include <errno.h>
#include <stdarg.h>
#include <stddef.h>

int mlock(const void *addr, size_t len) {
    (void)addr;
    (void)len;
    errno = ENOSYS;
    return -1;
}

int munlock(const void *addr, size_t len) {
    (void)addr;
    (void)len;
    errno = ENOSYS;
    return -1;
}

int msync(void *addr, size_t len, int flags) {
    (void)addr;
    (void)len;
    (void)flags;
    errno = ENOSYS;
    return -1;
}

int madvise(void *addr, size_t len, int advice) {
    (void)addr;
    (void)len;
    (void)advice;
    errno = ENOSYS;
    return -1;
}

void *mmap(void *addr, size_t length, int prot, int flags, int fd, long long offset) {
    (void)addr;
    (void)length;
    (void)prot;
    (void)flags;
    (void)fd;
    (void)offset;
    errno = ENOSYS;
    return (void *)-1;
}

int munmap(void *addr, size_t length) {
    (void)addr;
    (void)length;
    errno = ENOSYS;
    return -1;
}

int getrlimit(int resource, void *rlim) {
    (void)resource;
    (void)rlim;
    errno = ENOSYS;
    return -1;
}

int getpid(void) { return 1; }

int kill(int pid, int sig) {
    (void)pid;
    (void)sig;
    errno = ENOSYS;
    return -1;
}

int sched_getcpu(void) { return 0; }

int flock(int fd, int operation) {
    (void)fd;
    (void)operation;
    return 0;
}
