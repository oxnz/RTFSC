#ifndef CODEC_H
#define CODEC_H

#include <string>

template <typename T>
struct encoder {
		virtual std::string encode(const T& in) = 0;
};

template <typename T>
struct decoder {
		virtual T decode(const std::string& raw) = 0;
};

template <typename T, typename U>
struct codec : public encoder<T>, public decoder<U> {
};

#endif//CODEC_H
