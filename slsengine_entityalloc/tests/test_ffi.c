
#include <stdio.h>

int
test_ffi_main()
{
#ifndef RUST_TARGET
  return 1;
#else
#define TARGET_STR ""##RUST_TARGET
  fprintf(stdout, "target is %s\n", RUST_TARGET);
  return 0;
#endif
}