#include "rexxlint_portable.h"

/* Intentionally lightweight: tokenization is line-oriented in portable mode. */
int rexx_portable_lexer_reserved(void) {
    return 0;
}
