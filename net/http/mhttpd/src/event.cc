#include "event.h"

#ifdef __APPLE__

event_loop::event_loop(size_t n, const struct timespec& interval)
    :
    m_changes(new class event[n]),
    m_events(new class event[n]),
    size(0), capacity(n),
    m_interval(interval),
#ifdef __APPLE__
    m_fd(kqueue())
#elif __linux__
    m_fd(epoll_create(1))
#endif
{
    syslog(LOG_DEBUG, "[event_loop] startup");
}

event_loop::~event_loop() {
    syslog(LOG_DEBUG, "[event_loop] shutdown");
    close(m_fd);
    delete[] m_changes;
    delete[] m_events;
}

void event_loop::operator()(int fd, size_t flags, size_t data) {
    syslog(LOG_DEBUG, "[event_loop] register fd: %d on flags: %ld with data: %lu", fd, flags, data);
    require(size + bitcnt(flags) <= capacity, std::runtime_error("event queue would overflow"));
    if (EF_READABLE & flags) EV_SET((struct kevent *)&m_changes[size++], fd, EVFILT_READ, EV_ADD, 0, 0, (void*)data);
    if (EF_WRITABLE & flags) EV_SET((struct kevent *)&m_changes[size++], fd, EVFILT_READ, EV_ADD, 0, 0, (void*)data);
}

#endif
