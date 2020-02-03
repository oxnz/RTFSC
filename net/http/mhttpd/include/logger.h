#ifndef LOGGER_H
#define LOGGER_H

#include <syslog.h>
#include <string.h>
#include <errno.h>
#include <string>

struct logger {
    logger(const char* ident) : m_prefix(ident) {}
    template <typename ...Args>
    void debug(const char* fmt, Args... args) { log(LOG_DEBUG, fmt, args...); }

    template <typename ...Args>
    void info(const char* fmt, Args... args) { log(LOG_INFO, fmt, args...); }

    template <typename ...Args>
    void warn(const char* fmt, Args... args) { log(LOG_WARNING, fmt, args...); }

    template <typename ...Args>
    void error(const char* fmt, Args... args) { log(LOG_ERR, fmt, args...); }

private:
    template <typename ...Args>
    void log(int level, const char* fmt, Args... args) {
        syslog(level, fmt, args...);
    }
    const char* m_prefix;
};

#endif//LOGGER_H
