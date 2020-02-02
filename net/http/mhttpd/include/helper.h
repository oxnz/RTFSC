#ifndef HELPER_H
#define HELPER_H

#include <stdexcept>

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

#endif//HELPER_H
