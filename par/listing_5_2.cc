#include <atomic>
#include <thread>
#include <iostream>

std::atomic_flag data_ready = ATOMIC_FLAG_INIT;
std::uint32_t value = 0;

void read_thread() {
    while (!data_ready.test()) {
        // spin
    }
    std::cout << "data: " << value << std::endl;
}

void write_thread() {
    value = 42;
    data_ready.test_and_set();
}

int main() {
    std::thread r(read_thread);
    std::thread w(write_thread);

    r.join();
    w.join();

    return 0;
}
