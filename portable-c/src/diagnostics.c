#include "rexxlint_portable.h"

#include <stdio.h>
#include <string.h>

void rexx_diagnostics_init(rexx_diagnostics_t *out) {
    if (out == NULL) {
        return;
    }
    out->count = 0;
}

int rexx_add_diagnostic(
    rexx_diagnostics_t *out,
    const char *rule_id,
    rexx_severity_t severity,
    size_t line,
    size_t column,
    const char *message
) {
    rexx_diagnostic_t *item;

    if (out == NULL || rule_id == NULL || message == NULL) {
        return 1;
    }
    if (out->count >= REXXLINT_MAX_DIAGNOSTICS) {
        return 2;
    }

    item = &out->items[out->count++];
    item->rule_id = rule_id;
    item->severity = severity;
    item->line = line;
    item->column = column;
    (void)snprintf(item->message, sizeof(item->message), "%s", message);
    item->message[sizeof(item->message) - 1] = '\0';
    return 0;
}
