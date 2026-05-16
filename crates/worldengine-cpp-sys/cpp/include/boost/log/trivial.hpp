#pragma once

#include <iostream>

namespace boost
{
namespace log
{
namespace trivial
{
enum severity_level
{
   trace,
   debug,
   info,
   warning,
   error,
   fatal
};
} // namespace trivial
} // namespace log
} // namespace boost

#define BOOST_LOG_TRIVIAL(level)                                               \
   if (true)                                                                   \
   {                                                                           \
   }                                                                           \
   else                                                                        \
      std::clog

