#include <atomic>
#include <thread>
#include <mutex>

class spinlock_mutex {
    std::atomic_flag m_flag = ATOMIC_FLAG_INIT;
public:
    spinlock_mutex() = default;
    void lock() noexcept {
        while (m_flag.test_and_set(std::memory_order_release)) {
            // spin
        }
    }
    void unlock() noexcept { m_flag.clear(std::memory_order_release); }
};

int main() {
    unsigned int value = 0;
    spinlock_mutex mutex;
    unsigned int concurrency = std::max(std::thread::hardware_concurrency(), 2u);
    int m = 1000;
    std::vector<std::thread> workers;
    for (unsigned int i = 0; i < concurrency; i++) {
        workers.emplace_back([&value, &mutex, m] {
            for (int j = 0; j < m; ++j) {
                std::scoped_lock<spinlock_mutex> lock(mutex);
                value += 1;
            }
        });
    }
    for (auto& t: workers) {
        t.join();
    }
    return !(value == concurrency*m);
}
