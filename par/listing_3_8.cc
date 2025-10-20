#include <mutex>
#include <thread>
#include <iostream>

class hierarchical_mutex {
    int const m_hierarchy_value;
    int m_prev_hierarchy_value;
    std::mutex m_mutex;
    static thread_local int this_thread_hierarchy_value;

    void check_for_hierarchy_violation() {
        if (this_thread_hierarchy_value <= m_hierarchy_value) {
            throw std::logic_error("hierachy value inverse");
        }
    }

    void update_hierarchy_value() {
        m_prev_hierarchy_value = this_thread_hierarchy_value;
        this_thread_hierarchy_value = m_hierarchy_value;
    }

    void restore_hierarchy_value() {
        this_thread_hierarchy_value = m_prev_hierarchy_value;
    }

public:
    explicit hierarchical_mutex(int hierarchy_value): m_hierarchy_value(hierarchy_value), m_prev_hierarchy_value(0) {}
    bool try_lock() {
        check_for_hierarchy_violation();
        if (!m_mutex.try_lock()) {
            return false;
        }
        update_hierarchy_value();
        return true;
    }

    void lock() {
        check_for_hierarchy_violation();
        m_mutex.lock();
        update_hierarchy_value();
    }

    void unlock() {
        if (this_thread_hierarchy_value != m_hierarchy_value) {
            throw std::logic_error("hierarchy value mismatch");
        }
        restore_hierarchy_value();
        m_mutex.unlock();
    }
};

thread_local int hierarchical_mutex::this_thread_hierarchy_value(INT_MAX);

hierarchical_mutex high_level_mutex(3);
hierarchical_mutex mid_level_mutex(2);
hierarchical_mutex low_level_mutex(1);

int do_low_level_stuff() {
    std::cout << __FUNCTION__ << std::endl;
    return 0;
}
int low_level_func() {
    std::scoped_lock<hierarchical_mutex> lock(low_level_mutex);
    return do_low_level_stuff();
}

void high_level_stuff(int some_param) {
    std::cout << __FUNCTION__ << some_param << std::endl;
}
void high_level_func() {
    std::scoped_lock<hierarchical_mutex> lock(high_level_mutex);
    high_level_stuff(low_level_func());
}

void thread_a() {
    high_level_func();
}

void do_mid_stuff() {
    std::cout << __FUNCTION__ << std::endl;
}
void mid_stuff() {
    std::scoped_lock<hierarchical_mutex> lock(mid_level_mutex);
    high_level_func();
    do_mid_stuff();
}

void thread_b() {
    mid_stuff();
}


int main() {
    std::thread a(thread_a);
    std::thread b(thread_b);

    a.join();
    b.join();
    return 0;
}
