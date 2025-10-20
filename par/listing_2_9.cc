#include <numeric>
#include <vector>
#include <iostream>
#include <thread>

template<typename T, typename InputIter>
struct Chunk {
    void operator()(InputIter first, InputIter last, T& result) {
        result = std::accumulate(first, last, result);
    }
};

template <typename T, typename InputIter>
T par_accumulate(InputIter first, InputIter last, T initial_value) {
    size_t n = std::distance(first, last);
    if (n == 0) {
        return initial_value;
    }
    const size_t min_elems_per_thread = 127;
    size_t max_concurrency = (n + min_elems_per_thread - 1)/min_elems_per_thread;
    size_t hardware_concurrency = std::thread::hardware_concurrency();
    size_t concurrency = std::min(max_concurrency, hardware_concurrency == 0 ? 2 : hardware_concurrency);
    size_t chunk_size = (n + concurrency - 1)/concurrency;
    std::vector<std::thread> threads;
    std::vector<T> results(concurrency);
    for (size_t i = 0; i < concurrency-1; ++i) {
        auto chunk_end = first;
        std::advance(chunk_end, chunk_size);
        threads.emplace_back(Chunk<T, InputIter>(), first, chunk_end, std::ref(results[i]));
        first = chunk_end;
    }
    Chunk<T, InputIter>()(first, last, results[concurrency-1]);
    for (auto &t: threads) {
        t.join();
    }
    return std::accumulate(results.begin(), results.end(), initial_value);
}

int main() {
    std::cout << "hello world!" << std::endl;
    std::vector<size_t> v;
    for (size_t i = 0; i < 1*1000*1000; ++i) {
        v.push_back(i);
    }
    size_t r = par_accumulate(v.begin(), v.end(), 0);
    std::cout << "sum([0..1000,000]) = " << r << std::endl;
}
