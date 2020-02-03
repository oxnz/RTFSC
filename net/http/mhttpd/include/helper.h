#ifndef HELPER_H
#define HELPER_H

#include <stdexcept>
#include "config.h"
#include <arpa/inet.h>


inline void ensure(bool cond, const std::exception& ex) {
    if (!cond) throw ex;
}

inline void ensure(bool cond, const char* msg) {
    ensure(cond, std::runtime_error(msg));
}

inline void require(bool cond, const std::exception& ex) {
    ensure(cond, ex);
}

inline void require(bool cond, const char* msg) {
    require(cond, std::invalid_argument(msg));
}

template<>
std::string to_string(const struct sockaddr_in& sockaddr) {
    char addr_buf[INET_ADDRSTRLEN] = "misc";
    if (NULL == inet_ntop(AF_INET, &sockaddr.sin_addr, addr_buf, INET_ADDRSTRLEN)) {
        syslog(LOG_ERR, "[inet_ntop]: %s", strerror(errno));
    }
    return std::string(addr_buf) + ":" + ntohs(sockaddr.sin_port);
}

#endif//HELPER_H
