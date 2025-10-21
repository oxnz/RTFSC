#include <thread>
#include <queue>
#include <future>
#include <mutex>

int main() {

    std::queue<std::packaged_task<void()>> tasks;
    std::mutex mutex;
    std::thread background_worker([]{});
    while (true) {
        std::scoped_lock<std::mutex> lock(mutex);
        if (tasks.empty()) {
            continue;
        }
        tasks.pop();
    }
    background_worker.join();
    return 0;
}
