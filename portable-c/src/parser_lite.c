#include "rexxlint_portable.h"

/* parser_lite keeps only block-balance checks for legacy targets. */
int rexx_portable_parser_reserved(void) {
    return 0;
}
