#include <mutex>
#include <utility>

class some_big_object
{
    int m_value;
public:
    some_big_object(int value): m_value(value) {}

    friend void swap(some_big_object& lhs, some_big_object &rhs) {
        using std::swap;
        swap(lhs.m_value, rhs.m_value);
    }
};

class X {
    some_big_object m_obj;
    mutable std::mutex m_mtx;
public:
    explicit X(some_big_object const& obj) noexcept: m_obj(obj) {}
    friend void swap(X& lhs, X& rhs) noexcept;

};

inline void swap(X& lhs, X& rhs) noexcept {
        if (&lhs == &rhs) {
            return;
        }
        std::unique_lock<std::mutex> lock_l(lhs.m_mtx, std::defer_lock);
        std::unique_lock<std::mutex> lock_r(rhs.m_mtx, std::defer_lock);
        std::lock(lock_l, lock_r);
        using std::swap;
        swap(lhs.m_obj, rhs.m_obj);
}

// Specialize std::swap for X (optional, for direct std::swap() call)
namespace std {
inline void swap(X& lhs, X& rhs) noexcept {
    ::swap(lhs, rhs);  // call global friend via ADL
}
}

int main() {
    some_big_object a(1), b(2);
    X x_1(a), x_2(b);
    std::swap(x_1, x_2);
}

