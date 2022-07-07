#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class RegisterStatus : uint8_t {
  Registered,
  NotSupported,
  BroadcastWriterSetupErrored,
  ServerErrored,
};

struct RegisterResponse {
  RegisterStatus status;
  int32_t fd;
};

extern "C" {

RegisterResponse xbase_register(const uint32_t *root);

} // extern "C"
