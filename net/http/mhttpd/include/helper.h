#ifndef HELPER_H
#define HELPER_H

#include <stdexcept>
#include <sstream>

#include <arpa/inet.h>
#include <fcntl.h>
#include <sys/socket.h>


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

template <typename T>
struct ReprHelper {
    ReprHelper(const T* p) : ReprHelper(typeid(T).name(), p) {}
    ReprHelper(const std::string& name, const T* p) {
        m_ss << name << "<" << p << ": ";
    }
    ReprHelper& with(const std::string& s) {
        m_ss << "[" << s << "]";
        return *this;
    }
    template <typename U>
    ReprHelper& with(const std::string& k, const U& v) {
        m_ss << "[" << k << "=" << v << "]";
        return *this;
    }

    std::string str() {
        m_ss << ">";
        return m_ss.str();
    }
private:
    std::stringstream m_ss;
};

template<typename T>
inline std::string repr(const T& val) {
    return std::to_string(val);
}

template<>
inline std::string repr(const struct sockaddr_in& sockaddr) {
    char addr_buf[INET_ADDRSTRLEN] = "misc";
    if (NULL == inet_ntop(AF_INET, &sockaddr.sin_addr, addr_buf, INET_ADDRSTRLEN)) {
        return strerror(errno);
    }
    return std::string(addr_buf) + ":" + std::to_string(ntohs(sockaddr.sin_port));
}

inline timespec to_timespec(std::chrono::milliseconds duration) {
    size_t ms = duration.count();
    return timespec{static_cast<int>(ms/1000), static_cast<int>((ms%1000)*1000*1000)};
}

inline int nonblock(int fd) {
    return fcntl(fd, F_SETFL, fcntl(fd, F_GETFL) | O_NONBLOCK);
}

inline int bitcnt(size_t v) {
    int n = 0;
    for(; v; ++n) v &= v - 1;
    return n;
}

#ifdef __linux__
int nodelay(int sockfd) {
    int optval = 1;
    return setsockopt(sockfd, SOL_TCP, TCP_NODELAY, &optval, sizeof(optval));
}
#endif

#endif//HELPER_H
