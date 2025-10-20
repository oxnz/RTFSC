#include <shared_mutex>
#include <map>
#include <string>
#include <optional>
#include <mutex>
#include <iostream>
#include <thread>

struct dns_entry {
    std::string name;
    std::uint32_t addr;

    friend std::ostream& operator<<(std::ostream & os, const dns_entry & value) {
        os << value.name << ":" << value.addr;
        return os;
    }
};

class dns_cache {
    std::map<std::string, dns_entry> m_store;
    mutable std::shared_mutex m_mtx;
public:
   std::optional<dns_entry> find_entry(std::string const & name) const {
        std::shared_lock slock(m_mtx);
        auto it = m_store.find(name);
        if (it != m_store.end()) {
           return std::optional(it->second);
        } else {
            return std::nullopt;
        }
    }

    void upsert_entry(std::string const & name, dns_entry const& entry) {
        std::scoped_lock<std::shared_mutex> xlock(m_mtx);
        m_store.insert_or_assign(name, dns_entry(entry));
    }
};

void populate(dns_cache& cache) {
    std::string name = "oxnz.github.io";
    for (std::uint32_t i = 1; i < 1000; ++i) {
        cache.upsert_entry(name, dns_entry(name, i));
    }
}

int main() {
    dns_cache cache;
    std::string name = "oxnz.github.io";
    std::thread worker(populate, std::ref(cache));
    worker.join();
    auto found = cache.find_entry(name);    
    if (found.has_value()) {
        std::cout << found.value() << std::endl;
    } else {
        std::cout << "not found" << std::endl;
    }
    return 0;
}
