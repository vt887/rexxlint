CC ?= cc
CFLAGS ?= -std=c99 -Wall -Wextra -pedantic -O2
LDFLAGS ?=

BIN := rexxlint-portable
SRC := src/main.c src/diagnostics.c src/lexer.c src/parser_lite.c src/formatter_lite.c src/rules.c
OBJ := $(SRC:.c=.o)

.PHONY: all clean test

all: $(BIN)

$(BIN): $(OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) $(LDFLAGS)

%.o: %.c include/rexxlint_portable.h
	$(CC) $(CFLAGS) -Iinclude -c $< -o $@

clean:
	rm -f $(OBJ) $(BIN)

test: $(BIN)
	./tests/run.sh ./$(BIN)
